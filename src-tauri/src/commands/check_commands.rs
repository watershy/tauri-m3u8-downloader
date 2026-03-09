use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::State;

use crate::types::*;
use crate::services::*;
use crate::utils::logging_utils::LogErr;
use crate::utils::path_utils;

#[tauri::command]
pub async fn check_media(
    video_url: String,
    http_headers: std::collections::HashMap<String, String>,
    state: tauri::State<'_, AppState>,
) -> Result<CheckMediaResult, String> {
    let custom_path = state.with_settings(|s| s.download_path.clone()).log_err()?;
    let save_folder = if !custom_path.is_empty() {
        custom_path
    } else {
        state.default_download_path.clone()
    };

    let active_paths = state.with_jobs(|jobs| {
        jobs.iter()
            .filter(|job| job.status.is_ongoing())
            .map(|job: &DownloadJob| path_utils::join(&job.save_folder, &job.file_name))
            .collect::<Vec<PathBuf>>()
    }).unwrap_or_default();

    media_services::check_media(&video_url, &http_headers, save_folder, &active_paths)
    .await
    .map_err(|e| format!("Failed to download video: {}", e))
    .log_err()
}

#[tauri::command]
pub fn check_file_status(
    folder: String,
    filename: String,
    state: State<AppState>,
) -> Result<TargetFileStatus, String> {
    let full_path = Path::new(&folder).join(&filename);
    let is_busy = state.with_jobs(|jobs| {
        jobs.iter().any(|j| j.save_folder == folder && j.file_name == filename && j.status.is_ongoing())
    }).log_err()?;

    if is_busy {
        return Ok(TargetFileStatus::Busy);
    }

    if full_path.exists() {
        return Ok(TargetFileStatus::Exists);
    }

    Ok(TargetFileStatus::Ready)
}

#[tauri::command]
pub fn check_active_downloads(state: State<Arc<AppState>>) -> bool {
    state.with_jobs(|jobs| {
        jobs.iter().any(|j| j.status == DownloadStatus::Downloading)
    }).unwrap_or(false)
}

#[tauri::command]
pub fn check_files_exist(files: Vec<(String, Option<u64>)>) -> Vec<bool> {
    files.into_iter().map(|(path, expected_size)| {
        let p = std::path::Path::new(&path);
        if !p.exists() {
            return false;
        }

        if let Some(size) = expected_size {
            if let Ok(meta) = p.metadata() {
                return meta.len() == size;
            }
        }

        true
    }).collect()
}