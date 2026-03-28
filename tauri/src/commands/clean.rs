use crate::dto::{CleanConfigDto, CleanProgressDto, CleanSummaryDto};
use crate::state::AppState;
use null_e_core::trash::{delete_path, DeleteMethod};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn start_clean(
    app: AppHandle,
    _state: tauri::State<'_, AppState>,
    targets: Vec<String>,
    config: CleanConfigDto,
) -> Result<String, String> {
    let clean_id = uuid::Uuid::new_v4().to_string();
    let clean_id_clone = clean_id.clone();

    let method = if config.dry_run {
        DeleteMethod::DryRun
    } else if config.use_trash {
        DeleteMethod::Trash
    } else {
        DeleteMethod::Permanent
    };

    let app_handle = app.clone();
    let total = targets.len();

    tokio::task::spawn_blocking(move || {
        let mut succeeded = 0usize;
        let mut failed = 0usize;
        let mut bytes_freed = 0u64;

        for (i, target) in targets.iter().enumerate() {
            let path = PathBuf::from(target);
            let progress = CleanProgressDto {
                total_items: total,
                completed_items: i,
                bytes_cleaned: bytes_freed,
                current_item: target.clone(),
                is_complete: false,
            };
            let _ = app_handle.emit("clean:progress", &progress);

            match delete_path(&path, method) {
                Ok(freed) => {
                    succeeded += 1;
                    bytes_freed += freed;
                }
                Err(_) => {
                    failed += 1;
                }
            }
        }

        let summary = CleanSummaryDto {
            total_items: total,
            succeeded,
            failed,
            bytes_freed,
            used_trash: method == DeleteMethod::Trash,
        };
        let _ = app_handle.emit("clean:complete", &summary);
    });

    Ok(clean_id_clone)
}

#[tauri::command]
pub async fn cancel_clean(state: tauri::State<'_, AppState>) -> Result<(), String> {
    if let Some(progress) = state.clean_progress.lock().unwrap().as_ref() {
        progress.cancel();
    }
    Ok(())
}
