use crate::types::AppSettings;
use crate::utils::fs_utils;

pub fn load_settings_from_disk() -> Result<AppSettings, String> {
    let path = fs_utils::get_settings_file_path()?;
    if path.exists() {
        let content = fs_utils::read_text_from_file(path)?;
        let settings = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(settings)
    } else {
        Ok(AppSettings::default())
    }
}