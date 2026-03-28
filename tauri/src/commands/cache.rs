use crate::dto::GlobalCacheDto;
use crate::state::AppState;

#[tauri::command]
pub async fn detect_caches(
    _state: tauri::State<'_, AppState>,
) -> Result<Vec<GlobalCacheDto>, String> {
    let result = tokio::task::spawn_blocking(|| {
        let mut caches = null_e_core::caches::detect_caches().map_err(|e| e.to_string())?;
        null_e_core::caches::calculate_all_sizes(&mut caches).map_err(|e| e.to_string())?;
        Ok::<Vec<GlobalCacheDto>, String>(caches.into_iter().map(GlobalCacheDto::from).collect())
    })
    .await
    .map_err(|e| e.to_string())?;

    result
}

#[tauri::command]
pub async fn clean_cache(_state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let caches = null_e_core::caches::detect_caches().map_err(|e| e.to_string())?;
        let cache = caches
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| format!("Cache '{}' not found", id))?;
        null_e_core::caches::clean_cache(cache, true).map_err(|e| e.to_string())?;
        Ok::<(), String>(())
    })
    .await
    .map_err(|e| e.to_string())?
}
