// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)]
// #![windows_subsystem = "windows"]

mod commands;
mod constants;
mod services;
mod types;
mod utils;

//use tauri::Manager;
use commands::*;
use types::*;

use crate::utils::{fs_utils, logging_utils};
use crate::services::settings_services;

#[tokio::main]
async fn main() {
    let _guard = logging_utils::init_logging();
    tracing::info!("Application started!");

    let saved_jobs = fs_utils::load_jobs_from_disk();
    let settings = settings_services::load_settings_from_disk().unwrap_or_default();
    let default_download_path = tauri::api::path::download_dir()
        .map(|p| p.to_string_lossy().to_string())
        .expect("Critical Error: Could not resolve Downloads directory. Check App Sandbox permissions.");
    let app_state = AppState::new(saved_jobs, settings, default_download_path);
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            check_media,
            check_file_status,
            check_files_exist,
            check_active_downloads,
            create_job,
            pause_job,
            resume_job,
            get_jobs,
            delete_job,
            delete_completed_jobs,
            get_job_logs,
            open_folder,
            load_settings,
            save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}