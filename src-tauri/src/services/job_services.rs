use std::collections::HashMap;
use tauri::State;
use crate::services::download_services;
use crate::types::*;
use crate::utils::*;

pub async fn create_job(
    video_url: String,
    audio_url: Option<String>,
    headers: HashMap<String, String>,
    save_folder: String,
    file_name: String,
    state: AppState
) -> Result<String, String> { // Returns Job ID on success
    // 1. Fetch & Parse M3U8
    let video_m3u8_content = network_utils::fetch_http_text(&video_url, &headers).await?;
    let video_media_data: M3U8MediaPlaylist = match m3u8_utils::parse_m3u8_content(&video_m3u8_content, &video_url, &headers).await? {
        PlaylistData::Master(_) => return Err("Cannot download Master playlist directly".to_string()),
        PlaylistData::Media(d) => d,
    };

    // 2. Fetch & Parse Audio M3U8 (if it exists)
    let mut audio_m3u8_content: Option<String> = None;
    let mut audio_media_data: Option<M3U8MediaPlaylist> = None;
    if let Some(ref a_url) = audio_url {
        let content = network_utils::fetch_http_text(a_url, &headers).await?;
        match m3u8_utils::parse_m3u8_content(&content, a_url, &headers).await? {
            PlaylistData::Master(_) => return Err("Audio URL points to a Master Playlist".to_string()),
            PlaylistData::Media(d) => {
                audio_media_data = Some(d);
                audio_m3u8_content = Some(content);
            }
        }
    }

    // 2. Initialize Job State (Memory & Disk)
    let mut total_segments = video_media_data.total_segments;
    if let Some(ref a_data) = audio_media_data {
        total_segments += a_data.total_segments;
    }

    let job_id = init_job_state(&state, video_url.clone(), audio_url.clone(), &headers, &save_folder, &file_name, total_segments).await?;

    // 3. Setup Filesystem
    let download_folder = fs_utils::get_download_dir(&job_id)?; // Ensure folder exists
    fs_utils::ensure_directory_exists_async(&download_folder).await?;
    let save_to_file = path_utils::join(save_folder,file_name);

    // Save Video Backup
    let m3u8_backup_file_path = fs_utils::save_m3u8_content(&job_id, &video_m3u8_content, TrackType::Video)
        .await
        .map_err(|e| format!("Failed to save video M3U8 backup: {}", e))?;

    let mut audio_backup_file_path: Option<String> = None;
    if let Some(audio_content) = &audio_m3u8_content {
        // Save Audio Backup
        let path = fs_utils::save_m3u8_content(&job_id, audio_content, TrackType::Audio)
            .await
            .map_err(|e| format!("Failed to save audio M3U8 backup: {}", e))?;
        audio_backup_file_path = Some(path);
    }

    // 4. Write Initial Logs
    write_job_log(&job_id, LogCategory::General, LogLevel::Info, "Job initialized.").await;
    write_job_log(&job_id, LogCategory::General, LogLevel::Info, &format!("Video URL: \"{}\"", video_url)).await;
    if let Some(ref audio_url_val) = audio_url {
        write_job_log(&job_id, LogCategory::General, LogLevel::Info, &format!("Audio URL: \"{}\"", audio_url_val)).await;
    }

    write_job_log(&job_id, LogCategory::General, LogLevel::Info, &format!("Video M3U8 file saved to \"{}\"", m3u8_backup_file_path)).await;
    if let Some(path) = audio_backup_file_path {
        write_job_log(&job_id, LogCategory::General, LogLevel::Info, &format!("Audio M3U8 file saved to \"{}\"", path)).await;
    }

    write_job_log(&job_id, LogCategory::General, LogLevel::Info, &format!("Total Segments: {}", total_segments)).await;
    write_job_log(&job_id, LogCategory::General, LogLevel::Info, &format!("Video file to be downloaded to: \"{}\"", save_to_file.to_string())).await;

    // 5. Hand off to the Engine
    download_services::start_download_task(
        job_id.clone(),
        headers,
        save_to_file,
        video_media_data,
        audio_media_data,
        state
    ).await?;

    Ok(job_id)
}

async fn fetch_or_fallback_playlist(
    job_id: &str,
    url: &str,
    track_type: TrackType,
    headers: &HashMap<String, String>,
) -> Result<M3U8MediaPlaylist, String> {
    let track_name = track_type.as_str();

    match network_utils::fetch_http_text(url, headers).await {
        Ok(content) => {
            write_job_log(job_id, LogCategory::General, LogLevel::Info, &format!("Resume: Refreshed {} playlist from server.", track_name)).await;

            let data = m3u8_utils::parse_m3u8_content(&content, url, headers).await
                .map_err(|e| format!("Parsed invalid {} data from refresh: {}", track_name, e))?;
            let media = match data {
                PlaylistData::Media(m) => m,
                PlaylistData::Master(_) => return Err(format!("Resumed {} URL is a Master Playlist.", track_name)),
            };

            if let Err(e) = fs_utils::save_m3u8_content(job_id, &content, track_type).await {
                write_job_log(job_id, LogCategory::General, LogLevel::Warning, &format!("Failed to save {} backup: {}", track_name, e)).await;
            }
            
            Ok(media)
        },
        Err(e) => {
            write_job_log(job_id, LogCategory::General, LogLevel::Warning, &format!("{} refresh failed ({}), fallback to local M3U8...", track_name, e)).await;

            // Note: Update this to pass TrackType into your fs_utils!
            let backup_path = fs_utils::get_job_m3u8_backup_file_path(job_id, track_type)?;
            let saved_content = fs_utils::read_text_from_file_async(&backup_path).await?;
            let data = m3u8_utils::parse_m3u8_content(&saved_content, url, headers).await
                .map_err(|e| format!("Failed to parse local {} backup: {}", track_name, e))?;
            
            let media = match data {
                PlaylistData::Media(m) => m,
                PlaylistData::Master(_) => return Err(format!("Saved {} backup is a Master Playlist.", track_name)),
            };
            
            Ok(media)
        }
    }
}

pub async fn resume_job(
    job_id: String,
    video_url: String,
    audio_url: Option<String>,
    save_folder: String,
    file_name: String,
    headers: HashMap<String, String>,
    state: AppState
) -> Result<(), String> {
    // Fetch/Fallback Video
    let video_media = fetch_or_fallback_playlist(&job_id, &video_url, TrackType::Video, &headers).await?;

    // Fetch/Fallback Audio
    let mut audio_media: Option<M3U8MediaPlaylist> = None;
    if let Some(ref a_url) = audio_url {
        audio_media = Some(fetch_or_fallback_playlist(&job_id, a_url, TrackType::Audio, &headers).await?);
    }

    // Update state to handle the combined segment total, if your function needs it
    update_job_after_refresh(&state, &job_id, &video_media, &headers).await?; 

    // Hand off to the Engine
    let download_folder = fs_utils::get_download_dir(&job_id)?;
    let save_to_file = path_utils::join(save_folder, file_name);
    fs_utils::ensure_directory_exists_async(&download_folder).await?;
    
    download_services::start_download_task(
        job_id,
        headers,
        save_to_file,
        video_media,
        audio_media,
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
    // Stop any active downloads/merges
    if let Some(token) = state.active_tasks.get(&job_id) {
        token.cancel();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    // Remove from memory and save state
    state.with_jobs_mut(|jobs| {
        if let Some(index) = jobs.iter().position(|j| j.id == job_id) {
            jobs.remove(index);
            Ok(())
        } else {
            Err("Job not found".to_string())
        }
    })??;

    save_jobs_data(&state).await?;

    if let Ok(download_dir) = fs_utils::get_download_dir(&job_id) {
        if download_dir.exists() {
            if let Err(e) = fs_utils::remove_dir_all_async(&download_dir).await {
                println!("Warning: Could not delete temp folder for {}: {}", job_id, e);
            }
        }
    }

    // Delete the job directory (Logs, M3U8 Backups)
    if let Ok(job_dir) = fs_utils::get_job_dir(&job_id) {
        if job_dir.exists() {
            if let Err(e) = fs_utils::remove_dir_all_async(&job_dir).await {
                println!("Warning: Could not delete job directory for {}: {}", job_id, e);
            }
        }
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
    video_url: String,
    audio_url: Option<String>,
    headers: &HashMap<String, String>,
    save_folder: &str,
    file_name: &str,
    total_segments: u32
) -> Result<String, String> {
    let job = DownloadJob::new(
        video_url,
        audio_url,
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
