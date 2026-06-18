//! Trash support - safe deletion with recovery
//!
//! Provides cross-platform trash functionality so users can recover
//! accidentally deleted files.

mod platform;
mod record;

pub use platform::*;
pub use record::*;

use crate::core::{Artifact, CleanResult};
use crate::error::{NullEError, Result};
use std::path::Path;

/// Delete method for cleanup operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeleteMethod {
    /// Move to system trash (recoverable)
    #[default]
    Trash,
    /// Permanently delete (not recoverable!)
    Permanent,
    /// Just log what would be deleted (dry run)
    DryRun,
}

impl DeleteMethod {
    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "trash" => Some(Self::Trash),
            "permanent" | "delete" | "rm" => Some(Self::Permanent),
            "dry-run" | "dryrun" | "dry_run" => Some(Self::DryRun),
            _ => None,
        }
    }
}

/// Outcome of a delete operation, with honest freed-vs-pending accounting.
///
/// The headline "freed" number was historically a lie: Trash mode reported the file size as
/// "freed" even though trashing frees **0 bytes** until the Trash is emptied, and successful
/// deletes reported the *apparent* size rather than what actually returned to free space.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeleteOutcome {
    /// Apparent size of the target (`st_size` sum) — for display continuity with the scan totals.
    pub attributed: u64,
    /// Bytes returned to free space **now** (Permanent: on-disk allocated of removed files;
    /// Trash & DryRun: 0).
    pub freed: u64,
    /// Bytes that will be reclaimed **later** (Trash: on-disk allocated moved to Trash; else 0).
    pub pending: u64,
}

/// Delete a path; returns the apparent size acted upon (backward-compatible scalar).
///
/// For honest freed-vs-pending accounting (GUI summaries), use [`delete_path_detailed`].
pub fn delete_path(path: &Path, method: DeleteMethod) -> Result<u64> {
    delete_path_detailed(path, method).map(|o| o.attributed)
}

/// Delete a path, returning a full [`DeleteOutcome`] with real freed/pending bytes.
pub fn delete_path_detailed(path: &Path, method: DeleteMethod) -> Result<DeleteOutcome> {
    // Use symlink_metadata (lstat) — NOT exists()/is_dir(), which follow symlinks. A symlinked
    // directory must be treated as a single link to delete, never recursed into: otherwise the
    // best-effort walker would root at the link and delete the *target's* contents outside the
    // intended location (e.g. a symlink sitting in the Trash pointing at ~/Documents).
    let lmeta = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        // Already gone (incl. broken symlink that errors) → idempotent success, nothing to free.
        Err(_) => return Ok(DeleteOutcome::default()),
    };
    let is_symlink = lmeta.file_type().is_symlink();
    let is_real_dir = lmeta.is_dir(); // false for symlinks (lstat-based)

    let (apparent, allocated) = if is_real_dir {
        crate::fsutil::measure_tree(path)
    } else {
        // A symlink or regular file is a single entry; never measure a symlink's target.
        let apparent = if lmeta.is_file() { lmeta.len() } else { 0 };
        (apparent, crate::fsutil::allocated_len(&lmeta))
    };

    match method {
        DeleteMethod::DryRun => Ok(DeleteOutcome {
            attributed: apparent,
            freed: 0,
            pending: 0,
        }),
        DeleteMethod::Trash => match trash::delete(path) {
            // Trashing frees nothing now — the bytes are pending until the Trash is emptied.
            Ok(()) => Ok(DeleteOutcome {
                attributed: apparent,
                freed: 0,
                pending: allocated,
            }),
            // Only recurse for a REAL directory, never a symlink.
            Err(_) if is_real_dir && !is_symlink => delete_dir_contents_best_effort(path, method),
            Err(e) => Err(NullEError::Trash(format!("Failed to move to trash: {}", e))),
        },
        DeleteMethod::Permanent => {
            if is_real_dir && !is_symlink {
                match std::fs::remove_dir_all(path) {
                    Ok(()) => Ok(DeleteOutcome {
                        attributed: apparent,
                        freed: allocated,
                        pending: 0,
                    }),
                    Err(_) => delete_dir_contents_best_effort(path, method),
                }
            } else {
                // Regular file OR symlink → remove the entry itself (removes the link, not target).
                crate::fsutil::remove_file_robust(path)?;
                Ok(DeleteOutcome {
                    attributed: apparent,
                    freed: allocated,
                    pending: 0,
                })
            }
        }
    }
}

/// Best-effort deletion: walk directory bottom-up, delete what we can, skip locked/busy files.
/// Returns a [`DeleteOutcome`] reflecting only what was actually removed.
fn delete_dir_contents_best_effort(dir: &Path, method: DeleteMethod) -> Result<DeleteOutcome> {
    let mut moved_or_freed = 0u64; // on-disk allocated of items we successfully acted on
    let mut attributed = 0u64; // apparent size of items we successfully acted on

    // Collect entries bottom-up (files first, then dirs). WalkDir defaults to NOT following
    // symlinks, so we never descend through a symlinked directory.
    let mut files = Vec::new();
    let mut dirs = Vec::new();

    for entry in walkdir::WalkDir::new(dir)
        .contents_first(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path() == dir {
            continue; // Skip root dir itself
        }
        if entry.file_type().is_dir() {
            dirs.push(entry.path().to_path_buf());
        } else {
            files.push(entry.path().to_path_buf());
        }
    }

    // Delete files first
    for file in &files {
        let (apparent, allocated) = std::fs::symlink_metadata(file)
            .map(|m| (m.len(), crate::fsutil::allocated_len(&m)))
            .unwrap_or((0, 0));
        let ok = match method {
            DeleteMethod::Trash => trash::delete(file).is_ok(),
            _ => crate::fsutil::remove_file_robust(file).is_ok(),
        };
        if ok {
            moved_or_freed += allocated;
            attributed += apparent;
        }
    }

    // Then try to remove empty dirs (bottom-up order from walkdir)
    for d in &dirs {
        let _ = crate::fsutil::remove_dir_robust(d); // Only succeeds if empty
    }

    // Try to remove the root dir itself if now empty
    let _ = crate::fsutil::remove_dir_robust(dir);

    if moved_or_freed == 0 && attributed == 0 {
        return Err(NullEError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!("Could not delete any files in {}", dir.display()),
        )));
    }

    // Trash mode: items were moved, not freed → report as pending. Otherwise truly freed.
    let (freed, pending) = match method {
        DeleteMethod::Trash => (0, moved_or_freed),
        _ => (moved_or_freed, 0),
    };
    Ok(DeleteOutcome {
        attributed,
        freed,
        pending,
    })
}

/// Delete an artifact
pub fn delete_artifact(artifact: &Artifact, method: DeleteMethod) -> CleanResult {
    match delete_path_detailed(&artifact.path, method) {
        Ok(_outcome) => CleanResult::success(artifact.clone(), method == DeleteMethod::Trash),
        Err(e) => CleanResult::failure(artifact.clone(), e.to_string()),
    }
}

/// Calculate apparent size of a path (kept for callers/tests that want the simple scalar).
#[allow(dead_code)]
fn calculate_size(path: &Path) -> Result<u64> {
    Ok(crate::fsutil::measure_tree(path).0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_delete_method_from_str() {
        assert_eq!(DeleteMethod::parse("trash"), Some(DeleteMethod::Trash));
        assert_eq!(
            DeleteMethod::parse("permanent"),
            Some(DeleteMethod::Permanent)
        );
        assert_eq!(DeleteMethod::parse("dry-run"), Some(DeleteMethod::DryRun));
        assert_eq!(DeleteMethod::parse("invalid"), None);
    }

    #[test]
    fn test_dry_run_doesnt_delete() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");
        std::fs::write(&file, "hello world").unwrap();

        let size = delete_path(&file, DeleteMethod::DryRun).unwrap();
        assert!(size > 0);
        assert!(file.exists()); // File should still exist
    }

    #[test]
    fn test_permanent_delete() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");
        std::fs::write(&file, "hello world").unwrap();

        let size = delete_path(&file, DeleteMethod::Permanent).unwrap();
        assert!(size > 0);
        assert!(!file.exists()); // File should be gone
    }

    #[test]
    fn test_delete_nonexistent() {
        let path = Path::new("/nonexistent/path/that/doesnt/exist");
        let size = delete_path(path, DeleteMethod::DryRun).unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_delete_directory() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("subdir");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("file1.txt"), "content1").unwrap();
        std::fs::write(dir.join("file2.txt"), "content2").unwrap();

        let size = delete_path(&dir, DeleteMethod::Permanent).unwrap();
        assert!(size > 0);
        assert!(!dir.exists());
    }

    #[test]
    fn test_calculate_size() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");
        std::fs::write(&file, "0123456789").unwrap(); // 10 bytes

        let size = calculate_size(&file).unwrap();
        assert_eq!(size, 10);
    }

    #[test]
    fn test_permanent_reports_real_freed() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("d");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("a.bin"), vec![0u8; 4096]).unwrap();

        let outcome = delete_path_detailed(&dir, DeleteMethod::Permanent).unwrap();
        assert!(outcome.freed > 0, "permanent delete should free real bytes");
        assert_eq!(outcome.pending, 0);
        assert!(!dir.exists());
    }

    #[test]
    fn test_trash_reports_zero_freed_with_pending() {
        // trash::delete may not work in all CI sandboxes; only assert the accounting contract
        // when the move actually succeeds.
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("trash_me.bin");
        std::fs::write(&file, vec![0u8; 4096]).unwrap();

        if let Ok(outcome) = delete_path_detailed(&file, DeleteMethod::Trash) {
            if !file.exists() {
                assert_eq!(outcome.freed, 0, "trashing frees 0 bytes until emptied");
                assert!(outcome.pending > 0, "trashed bytes are pending");
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_permanent_delete_of_symlinked_dir_spares_target() {
        // A symlink pointing at a directory must be removed as a link — the target's contents
        // must survive (regression guard for the best-effort walker rooting at a symlink).
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("real_target");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("keep.txt"), b"precious").unwrap();

        let link = temp.path().join("link_to_target");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let outcome = delete_path_detailed(&link, DeleteMethod::Permanent).unwrap();
        assert!(!link.exists(), "the symlink itself should be gone");
        assert!(target.exists(), "the symlink target dir must survive");
        assert!(
            target.join("keep.txt").exists(),
            "target contents must NOT be deleted through the symlink"
        );
        // The link's own footprint is tiny; the target's 8 bytes must not be attributed as freed.
        assert!(outcome.freed < 4096 * 4);
    }

    #[test]
    fn test_dry_run_outcome_has_no_freed() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("x.bin");
        std::fs::write(&file, vec![0u8; 4096]).unwrap();

        let outcome = delete_path_detailed(&file, DeleteMethod::DryRun).unwrap();
        assert!(outcome.attributed > 0);
        assert_eq!(outcome.freed, 0);
        assert_eq!(outcome.pending, 0);
        assert!(file.exists());
    }
}
