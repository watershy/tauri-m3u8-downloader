use tauri::State;
use crate::types::*;
use crate::utils::*;
use crate::services::job_services;

#[tauri::command]
pub async fn create_job(
    download_url: String,
    save_folder: String,
    file_name: String,
    http_headers: std::collections::HashMap<String, String>,
    app_state: State<'_, AppState>
) -> Result<CreateJobResult, String> {
    let state_owned = app_state.inner().clone();
    job_services::create_job(
        download_url,
        http_headers,
        save_folder,
        file_name,
        state_owned)
        .await
        .map_err(|e| format!("Failed to download video: {}", e)).log_err()
        .map(|_| Ok(CreateJobResult {
            success: true,
            message: String::new(),
        }))
        .unwrap_or_else(|err_msg| Ok(CreateJobResult {
            success: false,
            message: err_msg,
        }))
}

#[tauri::command]
pub async fn resume_job(
    job_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let job_data = app_state.with_jobs(|jobs| {
        let job = jobs.iter().find(|j| j.id == job_id).ok_or("Job not found".to_string())?;

        if !job.status.can_resume() {
            return Ok::<_, String>(None);
        }

        Ok(Some((
            job.m3u8_url.clone(),
            job.save_folder.clone(),
            job.file_name.clone(),
            job.http_headers.clone()
        )))
    })??;

    match job_data {
        Some((url, save_folder, file_name, headers)) => {
            let state_owned = app_state.inner().clone();
            job_services::resume_job(
                job_id,
                url,
                save_folder,
                file_name,
                headers,
                state_owned
            ).await
        },
        None => {
            Ok(())          // Already running/done, just return success
        }
    }
}

#[tauri::command]
pub async fn pause_job(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // get will work because DashMap is handling the lock
    if let Some(token) = state.active_tasks.get(&job_id) {
        token.cancel();
        return Ok(());
    }

    let updated = state.with_jobs_mut(|jobs| {
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            if job.status.can_pause() {
                job.status = DownloadStatus::Paused;
                job.instant_speed = 0;
                return true;
            }
        }
        false
    }).unwrap_or(false);

    if updated {
        job_services::save_jobs_data(&state).await?;
        Ok(())
    } else {
        Ok(())
    }
}

#[tauri::command]
pub fn get_jobs(state: State<AppState>) -> Vec<DownloadJob> {
    state.with_jobs(|jobs| jobs.clone()).unwrap_or_default()
}

#[tauri::command]
pub async fn get_job_logs(job_id: String, category: String, offset: u64) -> Result<LogUpdate, String> {
    let log_category = match category.as_str() {
        "download" => LogCategory::Download,
        "merge" => LogCategory::Merge,
        _ => LogCategory::General, 
    };

    let log_file = fs_utils::get_job_log_file_path(&job_id, log_category)?;
    if !log_file.exists() {
        return Ok(LogUpdate {
            logs: String::new(),
            new_offset: 0
        });
    }

    let (content, new_pos) = fs_utils::read_text_from_offset_async(&log_file, offset).await.log_err()?;
    Ok(LogUpdate {
        logs: content,
        new_offset: new_pos,
    })
}

#[tauri::command]
pub async fn delete_job(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    job_services::delete_job(job_id, state).await
}

#[tauri::command]
pub async fn delete_completed_jobs(
    state: State<'_, AppState>,
) -> Result<(), String> {
    job_services::delete_completed_jobs(state).await
}