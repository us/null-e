use crate::dto::{CleanConfigDto, CleanFailureDto, CleanProgressDto, CleanSummaryDto};
use crate::state::AppState;
use null_e_core::core::CleanProgress;
use null_e_core::fsutil;
use null_e_core::trash::{delete_path_detailed, DeleteMethod};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn start_clean(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
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

    // Wire cancellation: create the progress handle, store it in shared state BEFORE spawning so
    // `cancel_clean` can reach it (previously this slot was never set → cancel was a no-op).
    let progress = CleanProgress::new(total);
    *state.clean_progress.lock().unwrap() = Some(progress.clone());

    tokio::task::spawn_blocking(move || {
        let mut succeeded = 0usize;
        let mut failed = 0usize;
        let mut bytes_freed = 0u64;
        let mut bytes_pending = 0u64;
        let mut failures = Vec::new();

        for (i, target) in targets.iter().enumerate() {
            if progress.is_cancelled() {
                break;
            }
            let path = PathBuf::from(target);

            let emit = CleanProgressDto {
                total_items: total,
                completed_items: i,
                bytes_cleaned: bytes_freed + bytes_pending,
                current_item: target.clone(),
                is_complete: false,
            };
            let _ = app_handle.emit("clean:progress", &emit);

            // Safety net: never hand a root volume or aggregate location (`/`, `~`, `~/.Trash`,
            // `~/Downloads`, `/Library`, …) to a recursive delete, regardless of how it got here.
            if fsutil::is_protected_aggregate(&path) {
                failed += 1;
                failures.push(CleanFailureDto {
                    path: target.clone(),
                    reason: "Refused: this is a protected location and was not deleted".to_string(),
                    is_tcc: false,
                    category: "refused".to_string(),
                });
                progress.complete_item(0);
                continue;
            }

            match delete_path_detailed(&path, method) {
                Ok(outcome) => {
                    succeeded += 1;
                    bytes_freed += outcome.freed;
                    bytes_pending += outcome.pending;
                    progress.complete_item(outcome.freed);
                }
                Err(err) => {
                    failed += 1;
                    failures.push(classify_failure(&path, &err));
                    progress.complete_item(0);
                }
            }
        }

        let cancelled = progress.is_cancelled();
        progress.mark_complete();

        let summary = CleanSummaryDto {
            total_items: total,
            succeeded,
            failed,
            bytes_freed,
            bytes_pending,
            used_trash: method == DeleteMethod::Trash,
            method_label: if cancelled {
                "Cancelled".to_string()
            } else {
                match method {
                    DeleteMethod::Trash => "Trash".to_string(),
                    DeleteMethod::Permanent | DeleteMethod::DryRun => "Deleted".to_string(),
                }
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

/// Map a delete error to an actionable, grouped failure DTO using the path + OS error.
fn classify_failure(path: &Path, err: &null_e_core::error::NullEError) -> CleanFailureDto {
    use null_e_core::error::NullEError;

    let class = match err {
        NullEError::Io(io_err) => fsutil::classify_delete_failure(path, Some(io_err)),
        NullEError::PermissionDenied(_) => fsutil::classify_delete_failure(path, None),
        // Trash errors are stringy; classify from the path (ownership/flags/location).
        NullEError::Trash(_) => fsutil::classify_delete_failure(path, None),
        _ => fsutil::DeleteFailureClass::Other,
    };

    let raw = err.to_string();
    let reason = format!("{} ({})", class.explain(), raw);

    CleanFailureDto {
        path: path.to_string_lossy().to_string(),
        reason,
        is_tcc: class.is_fda(),
        category: class.as_str().to_string(),
    }
}
