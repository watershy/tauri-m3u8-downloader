// use tauri::{AppHandle, Manager};
// use crate::types::front_end_state::FrontEndState;

pub mod download_services;
pub mod media_services;
pub mod settings_services;
pub mod job_services;

// pub use download_services::*;
// pub use media_services::*;
// pub use settings_services::*;

// pub fn set_frontend_state(state: &FrontEndState, app: &AppHandle) -> Result<(), String> {
//     let _ = app.emit_all("set-frontend-state", &state).map_err(|e| e.to_string())?;
//     Ok(())
// }