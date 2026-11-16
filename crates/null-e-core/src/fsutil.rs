//! Filesystem accounting primitives — honest sizes and free-space measurement.
//!
//! The cleaner's trust story depends on telling the truth about disk space:
//! - **Apparent size** (`st_size`) is what most tools sum; on APFS it diverges from reality
//!   for compressed and sparse files.
//! - **Allocated size** (`st_blocks * 512`) is the on-disk footprint — the better predictor of
//!   how much space deleting a file *might* return. It still overstates for APFS clones and for
//!   blocks pinned by snapshots, so it is labeled "size on disk", never "guaranteed freed".
//! - The only authoritative freed figure is the **free-space delta** from `statvfs`, which this
//!   module also exposes.

use std::fs::Metadata;
use std::path::{Path, PathBuf};

/// On-disk allocated bytes for a single file's metadata (`st_blocks * 512` on Unix).
///
/// Falls back to apparent length on platforms without block info.
#[cfg(unix)]
pub fn allocated_len(meta: &Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    // st_blocks is always in 512-byte units per POSIX, regardless of st_blksize.
    meta.blocks() * 512
}

/// On-disk allocated bytes for a single file's metadata.
#[cfg(not(unix))]
pub fn allocated_len(meta: &Metadata) -> u64 {
    meta.len()
}

/// `(device, inode)` identity for hardlink/clone dedup. `None` on non-Unix.
#[cfg(unix)]
pub fn file_id(meta: &Metadata) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    Some((meta.dev(), meta.ino()))
}

/// `(device, inode)` identity. `None` on non-Unix.
#[cfg(not(unix))]
pub fn file_id(_meta: &Metadata) -> Option<(u64, u64)> {
    None
}

/// Hardlink count (`st_nlink`). Returns 1 where unavailable.
#[cfg(unix)]
pub fn link_count(meta: &Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    meta.nlink()
}

/// Hardlink count. Returns 1 where unavailable.
#[cfg(not(unix))]
pub fn link_count(_meta: &Metadata) -> u64 {
    1
}

/// Free / total space for the filesystem containing `path`, from `statvfs`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DiskSpace {
    /// Total capacity in bytes (`f_blocks * f_frsize`).
    pub total: u64,
    /// Space available to an unprivileged process (`f_bavail * f_frsize`) — the honest "free".
    pub available: u64,
    /// Raw free space including root-reserved blocks (`f_bfree * f_frsize`).
    pub free_raw: u64,
}

/// Query free/total space for the filesystem containing `path` via `statvfs(3)`.
///
/// Returns `None` if the syscall fails or the platform is unsupported.
#[cfg(unix)]
pub fn disk_space(path: &Path) -> Option<DiskSpace> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let c_path = CString::new(path.as_os_str().as_bytes()).ok()?;
    // SAFETY: zeroed statvfs is a valid initial state; we only read fields after a success.
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
    if rc != 0 {
        return None;
    }
    // f_frsize is the fundamental block size; fall back to f_bsize if zero.
    let unit = if stat.f_frsize != 0 {
        stat.f_frsize as u64
    } else {
        stat.f_bsize as u64
    };
    Some(DiskSpace {
        total: stat.f_blocks as u64 * unit,
        available: stat.f_bavail as u64 * unit,
        free_raw: stat.f_bfree as u64 * unit,
    })
}

/// Query free/total space. `None` on unsupported platforms.
#[cfg(not(unix))]
pub fn disk_space(_path: &Path) -> Option<DiskSpace> {
    None
}

/// Bytes available to the user on the filesystem containing `path`.
pub fn available_bytes(path: &Path) -> Option<u64> {
    disk_space(path).map(|s| s.available)
}

/// Measure a path subtree, returning `(apparent_bytes, allocated_bytes)`.
///
/// - `apparent` sums `st_size` (matches what Finder/most tools show).
/// - `allocated` sums `st_blocks * 512` with **hardlink dedup** (`nlink > 1` counted once via
///   `(dev, ino)`), so repeated hardlinks aren't multiply-counted. APFS clones (distinct inodes
///   sharing extents) are *not* deduped here — that reclaim is only knowable from the free-space
///   delta, which the deletion engine measures separately.
///
/// Does not follow symlinks. Unreadable entries are skipped.
pub fn measure_tree(path: &Path) -> (u64, u64) {
    let m = measure_tree_counted(path);
    (m.apparent, m.allocated)
}

/// Result of measuring a path subtree.
#[derive(Debug, Clone, Copy, Default)]
pub struct TreeMeasure {
    /// Sum of `st_size` over regular files.
    pub apparent: u64,
    /// Sum of `st_blocks * 512` with hardlink dedup.
    pub allocated: u64,
    /// Number of regular files counted.
    pub file_count: u64,
}

/// Measure a path subtree returning apparent + allocated bytes and a file count.
///
/// See [`measure_tree`] for the apparent-vs-allocated and dedup semantics.
pub fn measure_tree_counted(path: &Path) -> TreeMeasure {
    use std::collections::HashSet;

    let meta = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return TreeMeasure::default(),
    };
    if !meta.is_dir() {
        let apparent = if meta.is_file() { meta.len() } else { 0 };
        return TreeMeasure {
            apparent,
            allocated: allocated_len(&meta),
            file_count: if meta.is_file() { 1 } else { 0 },
        };
    }

    let mut seen: HashSet<(u64, u64)> = HashSet::new();
    let mut out = TreeMeasure::default();
    for entry in walkdir::WalkDir::new(path).into_iter().flatten() {
        // Skip symlinks for free: walkdir's d_type (from readdir, no extra syscall) already tells
        // us the entry is a symlink, so we avoid even the lstat below. We never count a symlink's
        // target — deleting the entry only removes the link.
        if entry.file_type().is_symlink() {
            continue;
        }
        // lstat, NOT entry.metadata(): the latter follows symlinks for the final component, so a
        // symlink to a large file outside the tree would be counted at the target's size even
        // though deleting the entry only removes the link. With symlink_metadata a symlink is not
        // a regular file, so it falls through the is_file() check and contributes nothing.
        let m = match std::fs::symlink_metadata(entry.path()) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if !m.is_file() {
            continue;
        }
        out.apparent += m.len();
        out.file_count += 1;
        // Dedup hardlinks for the allocated figure.
        if link_count(&m) > 1 {
            if let Some(id) = file_id(&m) {
                if !seen.insert(id) {
                    continue;
                }
            }
        }
        out.allocated += allocated_len(&m);
    }
    out
}

/// Whether `path` is owned by root (uid 0) and not by the current user.
///
/// Used to classify items that need admin rights to delete. Returns false on non-Unix or if the
/// path can't be stat'd.
#[cfg(unix)]
pub fn is_root_owned(path: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match std::fs::symlink_metadata(path) {
        Ok(m) => {
            let uid = m.uid();
            // SAFETY: geteuid is always safe and never fails.
            let euid = unsafe { libc::geteuid() };
            uid == 0 && euid != 0
        }
        Err(_) => false,
    }
}

/// Whether `path` is owned by root. Always false on non-Unix.
#[cfg(not(unix))]
pub fn is_root_owned(_path: &Path) -> bool {
    false
}

/// Whether a path is the root volume or a dangerous aggregate location that must never be
/// handed to a generic recursive delete (used by the deletion engine's refuse-list).
pub fn is_protected_aggregate(path: &Path) -> bool {
    let p = path.to_string_lossy();
    if path == Path::new("/") {
        return true;
    }
    let home = dirs::home_dir();
    if let Some(home) = &home {
        if path == home.as_path() {
            return true;
        }
        // ~/.Trash, ~/Library, ~/Documents, ~/Downloads, ~/Desktop — never bulk-delete these roots.
        for danger in [
            "Library",
            "Documents",
            "Downloads",
            "Desktop",
            "Movies",
            "Music",
            "Pictures",
            ".Trash",
        ] {
            if path == home.join(danger) {
                return true;
            }
        }
    }
    // System roots.
    matches!(
        p.as_ref(),
        "/System"
            | "/Library"
            | "/Applications"
            | "/Users"
            | "/private"
            | "/usr"
            | "/bin"
            | "/etc"
            | "/var"
    )
}

/// Why a delete failed — drives honest, actionable failure reporting instead of raw errno strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteFailureClass {
    /// TCC denial — needs Full Disk Access (user-owned but access blocked).
    NeedsFda,
    /// Root-owned, or system-immutable (`schg`) — needs admin rights (v1 does not auto-elevate).
    NeedsAdmin,
    /// SIP-protected / sealed system volume (`SF_RESTRICTED`, `/System`) — unremovable by any app.
    SipProtected,
    /// On a read-only volume (`EROFS`).
    ReadOnlyVolume,
    /// User immutable flag (`uchg`) we could not clear.
    Immutable,
    /// Already gone (`ENOENT`) — callers should treat as success.
    NotFound,
    /// Busy / not-empty (`EBUSY`/`ENOTEMPTY`), often a daemon repopulating a cache.
    Busy,
    /// Anything else.
    Other,
}

impl DeleteFailureClass {
    /// Stable id for the DTO / UI grouping.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NeedsFda => "fda",
            Self::NeedsAdmin => "needs_admin",
            Self::SipProtected => "sip_protected",
            Self::ReadOnlyVolume => "read_only",
            Self::Immutable => "immutable",
            Self::NotFound => "not_found",
            Self::Busy => "busy",
            Self::Other => "other",
        }
    }

    /// Whether this is a Full Disk Access (TCC) failure — kept for back-compat with `is_tcc`.
    pub fn is_fda(&self) -> bool {
        matches!(self, Self::NeedsFda)
    }

    /// A human-readable, actionable explanation.
    pub fn explain(&self) -> &'static str {
        match self {
            Self::NeedsFda => "Needs Full Disk Access",
            Self::NeedsAdmin => "Needs administrator rights",
            Self::SipProtected => "Protected by macOS (cannot be removed by any app)",
            Self::ReadOnlyVolume => "On a read-only system volume",
            Self::Immutable => "File is locked (immutable flag)",
            Self::NotFound => "Already removed",
            Self::Busy => "In use or being rewritten",
            Self::Other => "Could not be removed",
        }
    }
}

/// macOS BSD file flags (`st_flags`) for `path`, via `lstat`. `None` elsewhere or on error.
#[cfg(target_os = "macos")]
pub fn file_flags(path: &Path) -> Option<u32> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let c = CString::new(path.as_os_str().as_bytes()).ok()?;
    // SAFETY: zeroed stat is valid; we only read st_flags after a successful lstat.
    let mut st: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::lstat(c.as_ptr(), &mut st) } != 0 {
        return None;
    }
    Some(st.st_flags)
}

/// BSD file flags. `None` on non-macOS.
#[cfg(not(target_os = "macos"))]
pub fn file_flags(_path: &Path) -> Option<u32> {
    None
}

/// If `path` has a *user* immutable/append flag (`uchg`/`uappnd`) and we own it, clear only those
/// bits (read-modify-write — never `chflags(0)`, which would clobber system flags). Returns whether
/// a clear was performed. No-op for root-owned or system-flagged files.
#[cfg(target_os = "macos")]
pub fn try_clear_user_immutable(path: &Path) -> bool {
    let Some(flags) = file_flags(path) else {
        return false;
    };
    let user_lock = libc::UF_IMMUTABLE | libc::UF_APPEND;
    if flags & user_lock == 0 {
        return false; // no user lock to clear
    }
    if is_root_owned(path) {
        return false; // we can't chflags a root-owned file as a normal user
    }
    let new_flags = flags & !user_lock;
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let Ok(c) = CString::new(path.as_os_str().as_bytes()) else {
        return false;
    };
    // lchflags, NOT chflags: chflags follows symlinks, so on a locked symlink it would clear flags
    // on the link's external target instead of the link itself. file_flags() already classified via
    // lstat, so we must mutate the same (link) inode here. The libc crate doesn't expose lchflags,
    // so bind it directly — it's a stable macOS/BSD syscall.
    extern "C" {
        fn lchflags(path: *const libc::c_char, flags: libc::c_uint) -> libc::c_int;
    }
    // SAFETY: c outlives the call; lchflags has the same contract as chflags but never follows links.
    unsafe { lchflags(c.as_ptr(), new_flags as libc::c_uint) == 0 }
}

/// No-op on non-macOS.
#[cfg(not(target_os = "macos"))]
pub fn try_clear_user_immutable(_path: &Path) -> bool {
    false
}

/// Directories to search for developer tools: `PATH`, then the usual install locations.
///
/// A GUI app launched from Finder inherits a minimal `PATH` (`/usr/bin:/bin:/usr/sbin:/sbin`), so
/// tools in `~/.cargo/bin`, Homebrew or `~/.bun/bin` are not found by name.
pub fn tool_search_dirs() -> Vec<PathBuf> {
    let home = dirs::home_dir();
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    let extra = [
        std::env::var_os("CARGO_HOME").map(|c| PathBuf::from(c).join("bin")),
        home.as_ref().map(|h| h.join(".cargo/bin")),
        Some(PathBuf::from("/opt/homebrew/bin")),
        Some(PathBuf::from("/usr/local/bin")),
        home.as_ref().map(|h| h.join(".bun/bin")),
        home.as_ref().map(|h| h.join("go/bin")),
        Some(PathBuf::from("/usr/local/go/bin")),
    ];
    for dir in extra.into_iter().flatten() {
        if !dirs.contains(&dir) {
            dirs.push(dir);
        }
    }
    dirs
}

/// `PATH` value for a child process: [`tool_search_dirs`] joined, so `#!/usr/bin/env node`
/// scripts such as npm also find their interpreter.
pub fn tool_search_path() -> std::ffi::OsString {
    std::env::join_paths(tool_search_dirs()).unwrap_or_default()
}

/// Resolve a developer tool binary by name; falls back to the bare name so errors stay readable.
pub fn resolve_tool(name: &str) -> PathBuf {
    find_tool_in(&tool_search_dirs(), name).unwrap_or_else(|| PathBuf::from(name))
}

/// First executable file called `name` in `dirs`.
fn find_tool_in(dirs: &[PathBuf], name: &str) -> Option<PathBuf> {
    dirs.iter()
        .map(|d| d.join(name))
        .find(|p| is_executable_file(p))
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

/// Retry `op` with the parent directory of `path` given its owner write bit, then put the old mode
/// back.
///
/// Unlinking needs write permission on the parent, not on the entry. Tools such as the Go module
/// cache leave their trees read-only (0555), so every delete inside them fails with `EACCES`.
/// Returns `None` (and changes nothing) unless we own the parent and it lacks the bit.
#[cfg(unix)]
fn retry_with_parent_writable(
    path: &Path,
    op: impl Fn(&Path) -> std::io::Result<()>,
) -> Option<std::io::Result<()>> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let parent = path.parent()?;
    let meta = std::fs::symlink_metadata(parent).ok()?;
    // SAFETY: geteuid is always safe and never fails.
    let euid = unsafe { libc::geteuid() };
    let mode = meta.mode() & 0o7777;
    if !meta.is_dir() || meta.uid() != euid || mode & 0o200 != 0 {
        return None;
    }
    std::fs::set_permissions(parent, std::fs::Permissions::from_mode(mode | 0o200)).ok()?;
    let result = op(path);
    let _ = std::fs::set_permissions(parent, std::fs::Permissions::from_mode(mode));
    Some(result)
}

#[cfg(not(unix))]
fn retry_with_parent_writable(
    _path: &Path,
    _op: impl Fn(&Path) -> std::io::Result<()>,
) -> Option<std::io::Result<()>> {
    None
}

/// Remove a single file, retrying once after clearing a user immutable flag (`EPERM`) or with the
/// parent's owner write bit restored (`EACCES`).
pub fn remove_file_robust(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) => {
            #[cfg(target_os = "macos")]
            {
                if e.raw_os_error() == Some(libc::EPERM) && try_clear_user_immutable(path) {
                    return std::fs::remove_file(path);
                }
            }
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                if let Some(result) = retry_with_parent_writable(path, |p| std::fs::remove_file(p))
                {
                    return result;
                }
            }
            Err(e)
        }
    }
}

/// Remove an (empty) directory, retrying once after clearing a user immutable flag (`EPERM`) or
/// with the parent's owner write bit restored (`EACCES`).
pub fn remove_dir_robust(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_dir(path) {
        Ok(()) => Ok(()),
        Err(e) => {
            #[cfg(target_os = "macos")]
            {
                if e.raw_os_error() == Some(libc::EPERM) && try_clear_user_immutable(path) {
                    return std::fs::remove_dir(path);
                }
            }
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                if let Some(result) = retry_with_parent_writable(path, |p| std::fs::remove_dir(p)) {
                    return result;
                }
            }
            Err(e)
        }
    }
}

/// Classify why a delete failed, using the path and (optionally) the OS error.
///
/// Crucially distinguishes **NeedsFda** (TCC, user-owned but blocked — admin won't help) from
/// **NeedsAdmin** (root-owned — admin helps) — errno alone cannot tell these apart, so we consult
/// ownership and BSD flags. This is what prevents the "nonsensical permission error" the user hit.
pub fn classify_delete_failure(path: &Path, err: Option<&std::io::Error>) -> DeleteFailureClass {
    use std::io::ErrorKind;

    if let Some(e) = err {
        if e.kind() == ErrorKind::NotFound {
            return DeleteFailureClass::NotFound;
        }
    }
    let errno = err.and_then(|e| e.raw_os_error());

    #[cfg(unix)]
    {
        match errno {
            Some(e) if e == libc::EROFS => return DeleteFailureClass::ReadOnlyVolume,
            Some(e) if e == libc::ENOTEMPTY || e == libc::EBUSY => return DeleteFailureClass::Busy,
            Some(e) if e == libc::ENOENT => return DeleteFailureClass::NotFound,
            _ => {}
        }
    }

    // SIP / sealed system volume by path.
    if path.starts_with("/System") {
        return DeleteFailureClass::SipProtected;
    }

    // BSD flags give the most precise classification on macOS.
    #[cfg(target_os = "macos")]
    {
        // SF_RESTRICTED (the SIP flag) is not exported by the `libc` crate; define it.
        const SF_RESTRICTED: u32 = 0x0008_0000;
        if let Some(flags) = file_flags(path) {
            if flags & SF_RESTRICTED != 0 {
                return DeleteFailureClass::SipProtected;
            }
            if flags & libc::SF_IMMUTABLE != 0 {
                return DeleteFailureClass::NeedsAdmin; // schg: needs root
            }
            if flags & (libc::UF_IMMUTABLE | libc::UF_APPEND) != 0 {
                return DeleteFailureClass::Immutable; // uchg we couldn't clear
            }
        }
    }

    // Permission denied: ownership decides FDA vs admin.
    #[cfg(unix)]
    {
        let denied = matches!(errno, Some(e) if e == libc::EPERM || e == libc::EACCES);
        // When we have no errno (e.g. a stringy trash error), still try ownership heuristics.
        if denied || errno.is_none() {
            if is_root_owned(path) {
                return DeleteFailureClass::NeedsAdmin;
            }
            if denied {
                // User-owned but blocked → almost always TCC/Full Disk Access for this app's targets.
                return DeleteFailureClass::NeedsFda;
            }
        }
    }

    DeleteFailureClass::Other
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[cfg(unix)]
    #[test]
    fn find_tool_skips_non_executable_and_searches_in_order() {
        use std::os::unix::fs::PermissionsExt;
        let temp = TempDir::new().unwrap();
        let (first, second) = (temp.path().join("a"), temp.path().join("b"));
        for (dir, mode) in [(&first, 0o644), (&second, 0o755)] {
            std::fs::create_dir_all(dir).unwrap();
            let tool = dir.join("tool");
            std::fs::write(&tool, b"#!/bin/sh\n").unwrap();
            std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(mode)).unwrap();
        }
        let dirs = vec![first, second.clone()];

        assert_eq!(find_tool_in(&dirs, "tool"), Some(second.join("tool")));
        assert_eq!(find_tool_in(&dirs, "missing"), None);
    }

    #[test]
    fn tool_search_dirs_include_cargo_bin() {
        // A Finder-launched GUI has no ~/.cargo/bin on PATH; it must still be searched.
        let cargo_bin = dirs::home_dir().unwrap().join(".cargo/bin");
        assert!(tool_search_dirs().contains(&cargo_bin));
    }

    #[cfg(unix)]
    #[test]
    fn robust_removes_work_inside_read_only_dirs() {
        use std::os::unix::fs::PermissionsExt;
        let temp = TempDir::new().unwrap();
        // Go module cache shape: read-only module dir holding a read-only bin dir.
        let module = temp.path().join("mod@v1.0.0");
        let bin = module.join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        let file = bin.join("tool");
        std::fs::write(&file, b"x").unwrap();
        for d in [&bin, &module] {
            std::fs::set_permissions(d, std::fs::Permissions::from_mode(0o555)).unwrap();
        }

        remove_file_robust(&file).expect("file in a 0555 dir should be removed");
        remove_dir_robust(&bin).expect("empty dir in a 0555 dir should be removed");
        assert!(!bin.exists());
        // The parent keeps its read-only mode: the write bit is put back after the retry.
        let mode = std::fs::metadata(&module).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o555);
    }

    #[test]
    fn disk_space_returns_something_for_temp() {
        let temp = TempDir::new().unwrap();
        let space = disk_space(temp.path());
        // On supported platforms we should get a value with non-zero capacity.
        #[cfg(unix)]
        {
            let space = space.expect("statvfs should succeed on unix temp dir");
            assert!(space.total > 0);
            assert!(space.available <= space.total);
        }
        #[cfg(not(unix))]
        let _ = space;
    }

    #[test]
    fn allocated_len_nonzero_for_real_file() {
        let temp = TempDir::new().unwrap();
        let f = temp.path().join("x.bin");
        std::fs::write(&f, vec![0u8; 8192]).unwrap();
        let meta = std::fs::metadata(&f).unwrap();
        assert!(allocated_len(&meta) > 0);
    }

    #[test]
    fn measure_tree_counts_files_and_dedups_hardlinks() {
        let temp = TempDir::new().unwrap();
        let a = temp.path().join("a.bin");
        std::fs::write(&a, vec![0u8; 4096]).unwrap();
        let m1 = measure_tree_counted(temp.path());
        assert_eq!(m1.file_count, 1);
        let alloc_one = m1.allocated;
        assert!(alloc_one > 0);

        // Hardlink b -> a: apparent doubles, allocated stays (dedup), count is 2.
        let b = temp.path().join("b.bin");
        #[cfg(unix)]
        {
            std::fs::hard_link(&a, &b).unwrap();
            let m2 = measure_tree_counted(temp.path());
            assert_eq!(m2.file_count, 2);
            assert_eq!(m2.allocated, alloc_one, "hardlinked data counted once");
            assert!(
                m2.apparent > m1.apparent,
                "apparent double-counts hardlinks"
            );
        }
        let _ = b;
    }

    #[cfg(unix)]
    #[test]
    fn measure_tree_ignores_symlink_to_external_file() {
        // A symlink inside the tree pointing at a large file OUTSIDE the tree must not be counted
        // at the target's size — deleting the entry only removes the link. Regression guard for
        // entry.metadata() (which follows symlinks) vs symlink_metadata.
        let outside = TempDir::new().unwrap();
        let big = outside.path().join("big.bin");
        std::fs::write(&big, vec![0u8; 1024 * 1024]).unwrap(); // 1 MiB target

        let tree = TempDir::new().unwrap();
        std::os::unix::fs::symlink(&big, tree.path().join("link_to_big")).unwrap();

        let m = measure_tree_counted(tree.path());
        assert_eq!(m.file_count, 0, "the symlink must not count as a file");
        assert_eq!(m.apparent, 0, "the target's 1 MiB must not be attributed");
        assert!(
            m.allocated < 4096 * 4,
            "only negligible link footprint, if any"
        );
    }

    #[test]
    fn classify_not_found_and_robust_remove() {
        let temp = TempDir::new().unwrap();
        let f = temp.path().join("gone.bin");
        std::fs::write(&f, b"x").unwrap();

        // robust remove succeeds on a normal file
        remove_file_robust(&f).unwrap();
        assert!(!f.exists());

        // a NotFound io error classifies as NotFound (idempotent success upstream)
        let err = std::fs::remove_file(&f).unwrap_err();
        assert_eq!(
            classify_delete_failure(&f, Some(&err)),
            DeleteFailureClass::NotFound
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn uchg_flag_is_cleared_and_file_deletes() {
        let temp = TempDir::new().unwrap();
        let f = temp.path().join("locked.bin");
        std::fs::write(&f, vec![0u8; 1024]).unwrap();

        // Set the user-immutable (uchg) flag via chflags.
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        let c = CString::new(f.as_os_str().as_bytes()).unwrap();
        let rc = unsafe { libc::chflags(c.as_ptr(), libc::UF_IMMUTABLE) };
        assert_eq!(rc, 0, "should be able to set uchg on our own file");

        // Plain remove fails with EPERM; robust remove clears uchg and succeeds.
        assert!(std::fs::remove_file(&f).is_err());
        remove_file_robust(&f).expect("robust remove should clear uchg and delete");
        assert!(!f.exists());
    }

    #[test]
    fn protected_aggregates_are_flagged() {
        assert!(is_protected_aggregate(Path::new("/")));
        assert!(is_protected_aggregate(Path::new("/System")));
        if let Some(home) = dirs::home_dir() {
            assert!(is_protected_aggregate(&home));
            assert!(is_protected_aggregate(&home.join(".Trash")));
        }
        assert!(!is_protected_aggregate(Path::new("/tmp/some/cache/dir")));
    }
}
