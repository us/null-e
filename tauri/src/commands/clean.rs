use crate::dto::{CleanConfigDto, CleanFailureDto, CleanProgressDto, CleanSummaryDto};
use crate::state::AppState;
use null_e_core::error::NullEError;
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
        let mut failures = Vec::new();

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
                Err(err) => {
                    failed += 1;
                    let (reason, is_tcc) = classify_delete_error(&err);
                    failures.push(CleanFailureDto {
                        path: target.clone(),
                        reason,
                        is_tcc,
                    });
                }
            }
        }

        let summary = CleanSummaryDto {
            total_items: total,
            succeeded,
            failed,
            bytes_freed,
            used_trash: method == DeleteMethod::Trash,
            method_label: match method {
                DeleteMethod::Trash => "Trash".to_string(),
                DeleteMethod::Permanent | DeleteMethod::DryRun => "Deleted".to_string(),
            },
            failures,
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

fn classify_delete_error(err: &NullEError) -> (String, bool) {
    match err {
        NullEError::Io(io_err) => {
            let is_tcc = io_err.raw_os_error() == Some(1);
            (io_err.to_string(), is_tcc)
        }
        NullEError::PermissionDenied(path) => {
            (format!("Permission denied: {}", path.display()), false)
        }
        NullEError::Trash(message) => {
            let lower = message.to_lowercase();
            let is_tcc = lower.contains("operation not permitted")
                || lower.contains("eperm")
                || lower.contains("os error 1");
            (message.clone(), is_tcc)
        }
        _ => (err.to_string(), false),
    }
}
