use crate::dto::{ScanConfigDto, ScanProgressDto, ScanResultDto};
use crate::state::AppState;
use null_e_core::core::{ScanConfig, Scanner};
use null_e_core::scanner::ParallelScanner;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

#[tauri::command]
pub async fn start_scan(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    config: ScanConfigDto,
) -> Result<String, String> {
    let mut scan_config: ScanConfig = config.into();

    // Fall back to config default_paths when no roots specified
    if scan_config.roots.is_empty() {
        let app_config = null_e_core::config::load_default_config().map_err(|e| e.to_string())?;
        scan_config.roots = app_config.general.default_paths;
    }

    // Filter to only existing directories
    scan_config.roots.retain(|p| p.exists() && p.is_dir());

    // If still empty, scan home directory as last resort
    if scan_config.roots.is_empty() {
        let home = std::env::var("HOME")
            .map(std::path::PathBuf::from)
            .ok()
            .filter(|p| p.exists());
        if let Some(home) = home {
            scan_config.roots.push(home);
        } else {
            return Err("No valid scan paths found. Add paths in Settings.".into());
        }
    }
    let registry = Arc::clone(&state.registry);
    let scanner = ParallelScanner::new(registry);
    let progress = scanner.progress();

    // Store progress for polling
    *state.scan_progress.lock().unwrap() = Some(Arc::clone(&progress));

    let scan_id = uuid::Uuid::new_v4().to_string();
    let scan_id_clone = scan_id.clone();

    // Spawn progress emitter
    let app_handle = app.clone();
    let progress_clone = Arc::clone(&progress);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let snapshot = progress_clone.snapshot();
            let dto = ScanProgressDto {
                directories_scanned: snapshot.directories_scanned,
                projects_found: snapshot.projects_found,
                total_size_found: snapshot.total_size_found,
                current_path: snapshot.current_path.to_string_lossy().to_string(),
                is_complete: snapshot.is_complete,
            };
            let _ = app_handle.emit("scan:progress", &dto);
            if snapshot.is_complete {
                break;
            }
        }
    });

    // Spawn scan in blocking thread
    let app_handle = app.clone();
    tokio::task::spawn_blocking(move || {
        match scanner.scan(&scan_config) {
            Ok(result) => {
                let dto = ScanResultDto::from(result);
                let _ = app_handle.emit("scan:complete", &dto);
                // Store result
                if let Some(state) = app_handle.try_state::<AppState>() {
                    *state.last_scan_result.lock().unwrap() = Some(dto);
                }
            }
            Err(e) => {
                let _ = app_handle.emit("scan:error", e.to_string());
            }
        }
    });

    Ok(scan_id_clone)
}

#[tauri::command]
pub async fn cancel_scan(state: tauri::State<'_, AppState>) -> Result<(), String> {
    if let Some(progress) = state.scan_progress.lock().unwrap().as_ref() {
        progress.cancel();
    }
    Ok(())
}

#[tauri::command]
pub async fn get_scan_result(
    state: tauri::State<'_, AppState>,
) -> Result<Option<ScanResultDto>, String> {
    Ok(state.last_scan_result.lock().unwrap().clone())
}
