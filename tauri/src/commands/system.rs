use crate::dto::{DiskInfoDto, FdaStatusDto};
use crate::state::AppState;
use std::fs;
use std::io::ErrorKind;

#[tauri::command]
pub async fn get_disk_info(_state: tauri::State<'_, AppState>) -> Result<DiskInfoDto, String> {
    // Read real filesystem stats via statvfs (honest free space) instead of parsing `df`.
    // On APFS, `available` is space available to the user (f_bavail); `used = total - available`
    // is the honest "occupied from the user's point of view" (purgeable space counts as used
    // until the OS reclaims it — which is exactly why "59 GB shows but won't free").
    let root = std::path::Path::new("/");
    let space = null_e_core::fsutil::disk_space(root)
        .ok_or_else(|| "Could not read filesystem stats (statvfs failed)".to_string())?;

    let total = space.total;
    let available = space.available;
    let used = total.saturating_sub(available);

    Ok(DiskInfoDto {
        total,
        used,
        available,
        mount_point: "/".to_string(),
    })
}

#[tauri::command]
pub async fn get_app_version(_state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(null_e_core::VERSION.to_string())
}

/// Open the macOS Full Disk Access settings pane.
///
/// Uses `open` with the Privacy_AllFiles anchor, which resolves to the FDA list on
/// Ventura/Sonoma/Sequoia when launched from a GUI app context (as Tauri is). If a future macOS
/// drops the anchor and lands on the top-level pane, the upgrade path is `NSWorkspace.open` via
/// objc2 — localized to this one function. The frontend also shows copy-paste manual steps as a
/// guaranteed fallback.
#[tauri::command]
pub async fn open_privacy_settings(_state: tauri::State<'_, AppState>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let url = "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles";
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(())
    }
}

#[tauri::command]
pub async fn check_fda_status(_state: tauri::State<'_, AppState>) -> Result<FdaStatusDto, String> {
    Ok(check_fda_status_inner())
}

/// Empty the user's Trash, returning bytes actually freed.
///
/// Powers the "Empty Trash now" action after a Trash-mode clean — so the user can actually reclaim
/// the space they just moved to the Trash (Trash mode itself frees 0 until this happens).
#[tauri::command]
pub async fn empty_trash(_state: tauri::State<'_, AppState>) -> Result<u64, String> {
    tokio::task::spawn_blocking(|| {
        let home = dirs::home_dir().ok_or_else(|| "No home directory".to_string())?;
        let trash = home.join(".Trash");
        if !trash.exists() {
            return Ok(0u64);
        }
        let mut freed = 0u64;
        let entries = fs::read_dir(&trash).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            // Each top-level item in ~/.Trash is permanently deleted; sum the real freed bytes.
            if let Ok(outcome) = null_e_core::trash::delete_path_detailed(
                &entry.path(),
                null_e_core::trash::DeleteMethod::Permanent,
            ) {
                freed += outcome.freed;
            }
        }
        Ok(freed)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(target_os = "macos")]
fn check_fda_status_inner() -> FdaStatusDto {
    let platform = std::env::consts::OS.to_string();
    let Some(home) = dirs::home_dir() else {
        return FdaStatusDto {
            status: "unknown".to_string(),
            platform,
        };
    };

    // Probe directories that are genuinely TCC-protected (require Full Disk Access to read).
    // NOTE: ~/Library/Caches is user-readable WITHOUT FDA — using it would false-positive — so we
    // probe ~/Library/Safari then ~/Library/Mail. A FDA denial returns EPERM(1) (sometimes
    // EACCES(13)); ENOENT means the dir doesn't exist on this machine, so we fall through.
    let probes = [
        home.join("Library/Safari"),
        home.join("Library/Mail"),
        home.join("Library/Application Support/com.apple.TCC"),
    ];

    for probe in &probes {
        match fs::read_dir(probe) {
            Ok(_) => {
                return FdaStatusDto {
                    status: "granted".to_string(),
                    platform,
                };
            }
            // EPERM(1)/EACCES(13) ⇒ the dir exists but we're blocked ⇒ FDA not granted.
            Err(err)
                if err.kind() == ErrorKind::PermissionDenied
                    || err.raw_os_error() == Some(1)
                    || err.raw_os_error() == Some(13) =>
            {
                return FdaStatusDto {
                    status: "not_granted".to_string(),
                    platform,
                };
            }
            // NotFound / other ⇒ inconclusive; try the next probe.
            Err(_) => {}
        }
    }

    FdaStatusDto {
        status: "unknown".to_string(),
        platform,
    }
}

#[cfg(not(target_os = "macos"))]
fn check_fda_status_inner() -> FdaStatusDto {
    FdaStatusDto {
        status: "granted".to_string(),
        platform: std::env::consts::OS.to_string(),
    }
}
