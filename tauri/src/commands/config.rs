use crate::state::AppState;

#[tauri::command]
pub async fn get_config(_state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let config = null_e_core::config::load_default_config().map_err(|e| e.to_string())?;
    serde_json::to_value(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_config(
    _state: tauri::State<'_, AppState>,
    config: serde_json::Value,
) -> Result<(), String> {
    let config: null_e_core::config::Config =
        serde_json::from_value(config).map_err(|e| e.to_string())?;
    let path = null_e_core::config::default_config_path().map_err(|e| e.to_string())?;
    null_e_core::config::save_config(&config, &path).map_err(|e| e.to_string())
}
