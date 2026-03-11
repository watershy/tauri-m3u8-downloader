use std::collections::HashMap;
use tauri::State;
use crate::services::download_services;
use crate::types::*;
use crate::utils::*;

pub async fn create_job(
    url: String,
    headers: HashMap<String, String>,
    save_folder: String,
    file_name: String,
    state: AppState
) -> Result<String, String> { // Returns Job ID on success

    // 1. Fetch & Parse M3U8
    let m3u8_content = network_utils::fetch_http_content(&url, &headers).await?;
    let media_data: M3U8MediaPlaylist = match m3u8_utils::parse_m3u8_content(&m3u8_content, &url).await? {
        PlaylistData::Master(_) => return Err("Cannot download Master playlist directly".to_string()),
        PlaylistData::Media(d) => d,
    };

    // 2. Initialize Job State (Memory & Disk)
    let total_segments = media_data.total_segments;
    let job_id = init_job_state(&state, &url, &headers, &save_folder, &file_name, total_segments).await?;

    // 3. Setup Filesystem
    let download_folder = fs_utils::get_download_dir(&job_id)?; // Ensure folder exists
    fs_utils::ensure_directory_exists_async(&download_folder).await?;
    let save_to_file = path_utils::join(save_folder,file_name);

    let m3u8_backup_file_path = fs_utils::save_m3u8_content(&job_id, &m3u8_content)
        .await
        .map_err(|e| format!("Failed to save M3U8 backup: {}", e))?;

    // 4. Write Initial Logs
    write_job_log(&job_id, LogCategory::General, LogLevel::Info, "Job initialized.").await;
    write_job_log(&job_id, LogCategory::General, LogLevel::Info, &format!("Downloading from \"{}\"", url)).await;
    write_job_log(&job_id, LogCategory::General, LogLevel::Info, &format!("M3U8 file saved to \"{}\"", m3u8_backup_file_path)).await;
    write_job_log(&job_id, LogCategory::General, LogLevel::Info, &format!("Total Segments: {}", total_segments)).await;
    write_job_log(&job_id, LogCategory::General, LogLevel::Info, &format!("Video file to be downloaded to: \"{}\"", save_to_file.to_string())).await;

    // 5. Hand off to the Engine
    download_services::start_download_task(
        job_id.clone(),
        headers,
        save_to_file,
        media_data,
        state
    ).await?;

    Ok(job_id)
}

pub async fn resume_job(
    job_id: String, 
    original_url: String, 
    save_folder: String, 
    file_name: String, 
    headers: HashMap<String, String>,
    state: AppState
) -> Result<(), String> {

    // --- PHASE 1: TRY FRESH FETCH (Preferred) ---
    let fresh_fetch_result = network_utils::fetch_http_content(&original_url, &headers).await;

    // Determine data and whether we need to save the new m3u8 backup
    let (media_data, should_save_backup, content_to_save) = match fresh_fetch_result {
        Ok(content) => {
            write_job_log(&job_id, LogCategory::General, LogLevel::Info, "Resume: Successfully refreshed playlist from server.").await;
            let data = m3u8_utils::parse_m3u8_content(&content, &original_url).await
                .map_err(|e| format!("Parsed invalid data from refresh: {}", e))?;

            let media = match data {
                PlaylistData::Media(m) => m,
                PlaylistData::Master(_) => return Err("Resumed URL is a Master Playlist.".to_string()),
            };

            update_job_after_refresh(&state, &job_id, &media, &headers).await?;
            (media, true, Some(content))
        },
        Err(e) => {
            // --- PHASE 2: FALLBACK TO SAVED FILE ---
            write_job_log(&job_id, LogCategory::General, LogLevel::Warning, &format!("Refresh failed ({}), attempting fallback to local M3U8...", e)).await;
            let backup_path = fs_utils::get_job_m3u8_backup_file_path(&job_id)?;
            let saved_content = fs_utils::read_text_from_file_async(&backup_path).await?;
            let data = m3u8_utils::parse_m3u8_content(&saved_content, &original_url).await
                .map_err(|e| format!("Failed to parse local backup: {}", e))?;

            let media = match data {
                PlaylistData::Media(m) => m,
                PlaylistData::Master(_) => return Err("Saved backup is a Master Playlist.".to_string()),
            };

            (media, false, None)
        }
    };

    // --- INTERMEDIATE: Update Backup (If Fresh) ---
    if should_save_backup {
        if let Some(content) = content_to_save {
            fs_utils::save_m3u8_content(&job_id, &content).await.ok();
        }
    }

    // --- PHASE 3: Hand off to the Engine ---
    let download_folder = fs_utils::get_download_dir(&job_id)?;
    let save_to_file = path_utils::join(save_folder, file_name);
    fs_utils::ensure_directory_exists_async(&download_folder).await?;
    download_services::start_download_task(
        job_id,
        headers,
        save_to_file,
        media_data,
        state
    ).await
}

pub async fn save_jobs_data(state: &AppState) -> Result<(), String> {
    let json_data = state.with_jobs(|jobs| { json_utils::serialize(jobs) })??;
    fs_utils::save_jobs_data(&json_data).await
}

pub async fn delete_job(
    job_id: String, 
    state: State<'_, AppState>
) -> Result<(), String> {
    if let Some(token) = state.active_tasks.get(&job_id) {
        token.cancel();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    state.with_jobs_mut(|jobs| {
        if let Some(index) = jobs.iter().position(|j| j.id == job_id) {
            jobs.remove(index);
            Ok(())
        } else {
            Err("Job not found".to_string())
        }
    })??;

    save_jobs_data(&state).await?;

    if let Ok(download_folder) = fs_utils::get_download_dir(&job_id) {
        if download_folder.exists() {
            if let Err(e) = fs_utils::remove_dir_all_async(&download_folder).await {
                println!("Warning: Could not delete temp folder for {}: {}", job_id, e);
            }
        }
    }

    for category in LogCategory::ALL {
        // Notice we use .as_str() here now!
        if let Ok(log_file) = fs_utils::get_job_log_file_path(&job_id, category) {
            let _ = fs_utils::remove_file_async(&log_file).await;
        }
    }

    if let Ok(backup_file) = fs_utils::get_job_m3u8_backup_file_path(&job_id) {
        let _ = fs_utils::remove_file_async(&backup_file).await;
    }

    Ok(())
}

pub async fn delete_completed_jobs(state: State<'_, AppState>) -> Result<(), String> {
    let ids_to_remove: Vec<String> = state.with_jobs(|jobs| {
        jobs.iter()
            .filter(|j| matches!(j.status, DownloadStatus::CompletedSuccess | DownloadStatus::CompletedError(_)))
            .map(|j| j.id.clone())
            .collect()
    })?;

    if ids_to_remove.is_empty() {
        return Ok(());
    }

    for job_id in ids_to_remove {
        // We ignore errors for individual items so one stuck file doesn't stop the whole batch
        let _ = delete_job(job_id, state.clone()).await;
    }

    Ok(())
}

pub async fn write_job_log(job_id: &str, category: LogCategory, level: LogLevel, message: &str) {
    let timestamp = chrono::Local::now().format("%H:%M:%S");
    let line = format!("[{}] [{}] {}\n", timestamp, level, message);

    if let Ok(log_file) = fs_utils::get_job_log_file_path(job_id, category) {
        let _ = fs_utils::append_to_file_async(&log_file, &line).await;
    }

    // If it's NOT the General category, AND the level is NOT Debug...
    if !matches!(category, LogCategory::General) {
        if !matches!(level, LogLevel::Debug) { 
            // This means Info, Warning, and Error will all be duplicated
            if let Ok(general_file) = fs_utils::get_job_log_file_path(job_id, LogCategory::General) {
                let duplicate_line = format!("[{}] [{}] [{}] {}\n", timestamp, level, category.as_str().to_uppercase(), message);
                let _ = fs_utils::append_to_file_async(&general_file, &duplicate_line).await;
            }
        }
    }
}

async fn init_job_state(
    state: &AppState,
    url: &str,
    headers: &HashMap<String, String>,
    save_folder: &str,
    file_name: &str,
    total_segments: u32
) -> Result<String, String> {
    let job = DownloadJob::new(
        url.to_string(),
        headers.clone(),
        save_folder.to_string(),
        file_name.to_string(),
        total_segments,
    );
    let download_id = job.id.clone();
    state.with_jobs_mut(|jobs| { jobs.push(job); }).log_err()?;
    save_jobs_data(&state).await?;
    Ok(download_id)
}

async fn update_job_after_refresh(
    state: &AppState,
    job_id: &str,
    media_data: &M3U8MediaPlaylist,
    fresh_headers: &HashMap<String, String>
) -> Result<(), String> {
    state.with_jobs_mut(|jobs| {
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            job.total_segments = media_data.total_segments;
            job.http_headers = fresh_headers.clone();
            // job.segments = convert_to_media_segments(&media_data.segments);  // No need?
            job.status = DownloadStatus::Downloading;
        }
    }).log_err()?;

    save_jobs_data(state).await?;
    Ok(())
}
