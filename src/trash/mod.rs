//! Trash support - safe deletion with recovery
//!
//! Provides cross-platform trash functionality so users can recover
//! accidentally deleted files.

mod platform;
mod record;

pub use platform::*;
pub use record::*;

use crate::core::{Artifact, CleanResult};
use crate::error::{DevSweepError, Result};
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
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "trash" => Some(Self::Trash),
            "permanent" | "delete" | "rm" => Some(Self::Permanent),
            "dry-run" | "dryrun" | "dry_run" => Some(Self::DryRun),
            _ => None,
        }
    }
}

/// Delete a path using the specified method
pub fn delete_path(path: &Path, method: DeleteMethod) -> Result<u64> {
    if !path.exists() {
        return Ok(0);
    }

    match method {
        DeleteMethod::DryRun => {
            // Just calculate size, don't delete
            let size = calculate_size(path)?;
            Ok(size)
        }
        DeleteMethod::Trash => {
            let size = calculate_size(path)?;
            trash::delete(path).map_err(|e| {
                DevSweepError::Trash(format!("Failed to move to trash: {}", e))
            })?;
            Ok(size)
        }
        DeleteMethod::Permanent => {
            let size = calculate_size(path)?;
            if path.is_dir() {
                std::fs::remove_dir_all(path)?;
            } else {
                std::fs::remove_file(path)?;
            }
            Ok(size)
        }
    }
}

/// Delete an artifact
pub fn delete_artifact(artifact: &Artifact, method: DeleteMethod) -> CleanResult {
    match delete_path(&artifact.path, method) {
        Ok(_bytes) => CleanResult::success(artifact.clone(), method == DeleteMethod::Trash),
        Err(e) => CleanResult::failure(artifact.clone(), e.to_string()),
    }
}

/// Calculate size of a path
fn calculate_size(path: &Path) -> Result<u64> {
    if path.is_file() {
        return Ok(path.metadata()?.len());
    }

    let mut size = 0u64;
    for entry in walkdir::WalkDir::new(path) {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                if let Ok(meta) = entry.metadata() {
                    size += meta.len();
                }
            }
        }
    }
    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_delete_method_from_str() {
        assert_eq!(DeleteMethod::from_str("trash"), Some(DeleteMethod::Trash));
        assert_eq!(DeleteMethod::from_str("permanent"), Some(DeleteMethod::Permanent));
        assert_eq!(DeleteMethod::from_str("dry-run"), Some(DeleteMethod::DryRun));
        assert_eq!(DeleteMethod::from_str("invalid"), None);
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
}
