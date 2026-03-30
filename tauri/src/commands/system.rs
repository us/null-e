use crate::dto::{DiskInfoDto, FdaStatusDto};
use crate::state::AppState;
use std::fs;
use std::io::ErrorKind;
use std::process::Command;

#[tauri::command]
pub async fn get_disk_info(_state: tauri::State<'_, AppState>) -> Result<DiskInfoDto, String> {
    // Use df command to get disk info for the root mount
    let output = Command::new("df")
        .args(["-k", "/"])
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    if lines.len() < 2 {
        return Err("Could not parse df output".to_string());
    }

    let parts: Vec<&str> = lines[1].split_whitespace().collect();
    if parts.len() < 6 {
        return Err("Could not parse df output".to_string());
    }

    let total = parts[1].parse::<u64>().unwrap_or(0) * 1024;
    let used = parts[2].parse::<u64>().unwrap_or(0) * 1024;
    let available = parts[3].parse::<u64>().unwrap_or(0) * 1024;
    let mount_point = parts[5].to_string();

    Ok(DiskInfoDto {
        total,
        used,
        available,
        mount_point,
    })
}

#[tauri::command]
pub async fn get_app_version(_state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(null_e_core::VERSION.to_string())
}

#[tauri::command]
pub async fn check_fda_status(_state: tauri::State<'_, AppState>) -> Result<FdaStatusDto, String> {
    Ok(check_fda_status_inner())
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

    let tcc_dir = home.join("Library/Application Support/com.apple.TCC");
    let status = match fs::read_dir(tcc_dir) {
        Ok(_) => "granted",
        Err(err) if err.kind() == ErrorKind::PermissionDenied || err.raw_os_error() == Some(1) => {
            "not_granted"
        }
        Err(_) => "unknown",
    };

    FdaStatusDto {
        status: status.to_string(),
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
