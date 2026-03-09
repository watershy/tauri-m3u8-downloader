// use serde::Serialize;
// use tauri::{AppHandle, Manager};

// pub fn emit_event<T: Serialize>(event_name: &str, payload: &T, app: &AppHandle) -> Result<(), String> {
//     let _ = app.emit_all(event_name, payload).map_err(|e| e.to_string())?;
//     Ok(())
// }