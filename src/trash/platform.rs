//! Platform-specific trash implementation

use crate::error::{DevSweepError, Result};
use std::path::PathBuf;

/// Get the trash directory for the current platform
pub fn get_trash_dir() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or_else(|| {
            DevSweepError::Trash("Cannot determine home directory".into())
        })?;
        Ok(home.join(".Trash"))
    }

    #[cfg(target_os = "linux")]
    {
        // XDG trash directory
        if let Some(data_home) = dirs::data_dir() {
            return Ok(data_home.join("Trash/files"));
        }

        let home = dirs::home_dir().ok_or_else(|| {
            DevSweepError::Trash("Cannot determine home directory".into())
        })?;
        Ok(home.join(".local/share/Trash/files"))
    }

    #[cfg(target_os = "windows")]
    {
        // Windows Recycle Bin is not directly accessible as a path
        // The trash crate handles this correctly
        Err(DevSweepError::Trash(
            "Direct trash path access not supported on Windows".into(),
        ))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(DevSweepError::Trash("Unsupported platform".into()))
    }
}

/// Check if trash is available on this system
pub fn is_trash_available() -> bool {
    // Try to get trash directory
    get_trash_dir().is_ok()
}

/// Get the size of items in trash
pub fn get_trash_size() -> Result<u64> {
    let trash_dir = get_trash_dir()?;

    if !trash_dir.exists() {
        return Ok(0);
    }

    let mut size = 0u64;
    for entry in walkdir::WalkDir::new(&trash_dir) {
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

    #[test]
    fn test_trash_available() {
        // Should be available on macOS and Linux
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        assert!(is_trash_available());
    }
}
