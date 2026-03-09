use tauri::State;
use crate::AppState;
use crate::types::AppSettings;
use crate::utils::{LogErr, fs_utils, json_utils};

#[tauri::command]
pub fn save_settings(
    settings: AppSettings,
    state: State<AppState>,
) -> Result<(), String> {
    state.with_settings_mut(|s| {
        *s = settings.clone();
    }).log_err()?;

    let path = fs_utils::get_settings_file_path()?;
    let json = json_utils::serialize(&settings)?;
    fs_utils::write_file_sync(path, &json).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn load_settings(state: State<AppState>) -> Result<AppSettings, String> {
    state.with_settings(|settings| settings.clone())
}