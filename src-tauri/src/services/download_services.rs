use std::collections::HashMap;
use std::path::{PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, mpsc};
use tauri::api::process::{Command, CommandEvent};
use tokio::time::{sleep, interval};
use tokio::io::AsyncWriteExt;
use tokio::fs as tokio_fs;
use tokio_util::sync::CancellationToken;
use futures_util::StreamExt;
use crate::types::*;
use crate::utils::*;
use super::job_services::{save_jobs_data, write_job_log};

pub enum MergeOutcome {
    Success,
    Cancelled,
}

pub async fn start_download_task(
    job_id: String,
    headers: HashMap<String, String>,
    save_to_file: PathBuf,
    media_data: M3U8MediaPlaylist,
    state: AppState
) -> Result<(), String> {
    // 1. Create and Register the "Kill Switch"
    let cancel_token = CancellationToken::new();
    state.active_tasks.insert(job_id.clone(), cancel_token.clone());

    // 2. Clone data for the background thread
    let job_id_clone = job_id.clone();
    let state_clone = state.clone();

    // 3. Spawn the background worker
    // We do NOT await this. It runs in the background.
    tokio::spawn(async move {
        let result = download_and_merge(
            job_id_clone.clone(),
            headers,
            save_to_file,
            media_data,
            cancel_token,
            state_clone.clone()
        ).await;

        // 4. Handle Final Cleanup
        // Remove the token so the "Pause" button knows it's inactive
        state_clone.active_tasks.remove(&job_id_clone);

        if let Err(e) = result {
            eprintln!("Job {} failed: {}", job_id_clone, e);
            // Optional: You could update the job status to 'Error' here if download_and_merge didn't already
            write_job_log(&job_id_clone, LogCategory::General, LogLevel::Error, &format!("Critical Job Error: {}", e)).await;

            if state_clone.with_settings(|s| s.play_completion_sound).unwrap_or(true) {
                audio_utils::play_error_sound();
            }
        }
    });

    Ok(())
}

pub async fn download_and_merge(
    job_id: String,
    headers: HashMap<String, String>,
    save_to_file: PathBuf,
    media_data: M3U8MediaPlaylist,
    cancel_token: CancellationToken,
    state: AppState
) -> Result<(), String> {
    // Producer-Consumer
    let download_folder = fs_utils::get_download_dir(&job_id)?;
    let total_segments = media_data.total_segments;
    write_job_log(&job_id, LogCategory::Download, LogLevel::Info,
        &format!("Downloading {} segments to \"{}\"", total_segments, download_folder.to_string())).await;

    let (tx, rx) = mpsc::channel::<DownloadProgress>(10000);

    // Spawn the workers (Producer) - This returns immediately
    spawn_download_workers(media_data, download_folder.clone(), headers, tx, cancel_token.clone());

    // Run the monitor (Consumer) - This blocks until all downloads finish
    if let Err(e) = monitor_progress(rx, state.clone(), job_id.clone(), total_segments, cancel_token.clone()).await {
        write_job_log(&job_id, LogCategory::Download, LogLevel::Error, &format!("Download aborted: {}", e)).await;

        if let Ok(mut jobs) = state.download_jobs.lock() {
            if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
                job.status = DownloadStatus::CompletedError(e.clone());
                job.instant_speed = 0; 
            }
        }
        let _ = save_jobs_data(&state).await;
        return Err(e); // Exit immediately!
    }

    // If the token is cancelled, it means we broke out of the download loop early.
    // We must STOP here. Do not merge.
    if cancel_token.is_cancelled() {
        write_job_log(&job_id, LogCategory::Download, LogLevel::Warning, "Download Job Paused.").await;

        // Update UI Status to Paused
        if let Ok(mut jobs) = state.download_jobs.lock() {
            if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
                job.status = DownloadStatus::Paused;
                job.instant_speed = 0; // Reset speed to 0
            }
        }

        // Save state immediately
        save_jobs_data(&state).await?;
        return Ok(()); // Return OK to signal "Graceful Exit", but skip Merge.
    }

    write_job_log(&job_id, LogCategory::Download, LogLevel::Info, "Download Completed.").await;

    // Finalization Phase (Merge)
    let job_id_clone = job_id.clone();
    let outcome = finalize_merge(
        state,
        job_id_clone,
        download_folder.clone(),
        save_to_file,
        cancel_token.clone()).await?;

    // Cleanup
    if let MergeOutcome::Success = outcome {
        if let Ok(download_folder) = fs_utils::get_download_dir(&job_id) {
            let _ = fs_utils::remove_dir_all_async(&download_folder).await;
            write_job_log(&job_id, LogCategory::General, LogLevel::Info, &format!("Temp download folder \"{}\" deleted.", download_folder.to_string())).await;
        }
    } else {
        write_job_log(&job_id, LogCategory::General, LogLevel::Warning, &format!("Job doesn't complete success. Temp download folder \"{}\" was kept.", download_folder.to_string())).await;
    }

    Ok(())
}

fn spawn_download_workers(
    media_data: M3U8MediaPlaylist,
    base_folder: PathBuf,
    headers: HashMap<String, String>,
    tx: mpsc::Sender<DownloadProgress>,
    cancel_token: CancellationToken
) {
    let total_segments = media_data.total_segments;
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let semaphore = Arc::new(Semaphore::new(10));
        let shared_headers = Arc::new(headers);
        let is_ts = media_data.init_map_file.is_none();
        let extension = if is_ts { "m4s" } else { "ts" };
        let mut download_queue = Vec::new();

        if let Some(init_seg) = media_data.init_map_file {
            download_queue.push((init_seg.url, "init.mp4".to_string()));
        }
        for (index, segment) in media_data.segments.into_iter().enumerate() {
            download_queue.push((segment.url, format!("seq_{:05}.{}", index, extension)));
        }

        for (url, file_name) in download_queue {
            if cancel_token.is_cancelled() {
                break;      // Stop queuing new tasks immediately
            }

            let client = client.clone();
            let headers = Arc::clone(&shared_headers);
            let sem = Arc::clone(&semaphore);
            let folder = base_folder.clone();
            let tx = tx.clone();

            let permit = tokio::select! {
                _ = cancel_token.cancelled() => break, // Stop waiting if paused
                p = sem.acquire_owned() => p,
            };

            let owned_permit = match permit {
                Ok(p) => p,
                Err(_) => break, // Semaphore closed error
            };

            tokio::spawn(async move {
                let _permit = owned_permit;

                let file_path = folder.join(&file_name);
                let result = download_file_with_retry(
                    &client,
                    &url,
                    &file_path,
                    &headers,
                    is_ts,
                    tx.clone()).await;
                
                let success = result.is_ok();
                if let Err(e) = result {
                    let _ = tx.send(DownloadProgress::Log(LogLevel::Error, format!("Download failed for {}: {}", file_name, e))).await;
                } else {
                    let _ = tx.send(DownloadProgress::Log(LogLevel::Debug, format!("Downloaded segment: {} (out of {} segments)", file_name, total_segments))).await;
                }
                
                let _ = tx.send(DownloadProgress::SegmentFinished(success)).await;
            });
        }
        // Original 'tx' drops here automatically, allowing the receiver to close when workers finish
    });
}

async fn monitor_progress(
    mut rx: mpsc::Receiver<DownloadProgress>,
    state: AppState,
    download_id: String,
    total_segments: u32,
    cancel_token: CancellationToken,
) -> Result<(), String> {
    let mut session_bytes: u64 = 0;
    let mut finished_segments = 0;
    let start_time = Instant::now();

    let mut last_update = Instant::now();
    let mut last_bytes = 0;

    let mut ticker = interval(Duration::from_millis(500));

    let update_state = |bytes: u64, segments: u32, speed: u64, progress: f32, eta: u64| {
        if let Ok(mut jobs) = state.download_jobs.lock() {
            if let Some(job) = jobs.iter_mut().find(|j| j.id == download_id) {
                job.downloaded_bytes = bytes;
                job.downloaded_segments = segments;
                job.instant_speed = speed;
                if job.smoothed_speed == 0.0 {
                    job.smoothed_speed = job.instant_speed as f64;
                }
                job.smoothed_speed = (job.instant_speed as f64 * 0.2) + (job.smoothed_speed * 0.8);
                job.progress = progress;
                job.eta = eta;
            }
        }
    };

    loop {
        // tokio::select! waits for MULTIPLE asynchronous events at the same time
        tokio::select! {
            // Event 1: A message arrived from a worker
            msg_opt = rx.recv() => {
                match msg_opt {
                    Some(msg) => {
                        match msg {
                            DownloadProgress::BytesDownloaded(b) => session_bytes += b,
                            DownloadProgress::Log(level, log_text) => {
                                write_job_log(&download_id, LogCategory::Download, level, &log_text).await;
                            },
                            DownloadProgress::SegmentFinished(success) => {
                                if success {
                                    finished_segments += 1;
                                } else {
                                    // FATAL ERROR: A segment failed completely.
                                    cancel_token.cancel(); // 1. Tell all other workers to abort
                                    update_state(session_bytes, finished_segments, 0, 0.0, 0); // 2. Zero out speed
                                    return Err("A segment failed to download after maximum retries.".to_string()); // 3. Abort monitor
                                }
                            }
                        }
                    },
                    None => {
                        // Channel closed (all workers finished or cancelled). Break the loop.
                        break; 
                    }
                }
            }
            // Event 2: 500ms has passed
            _ = ticker.tick() => {
                let now = Instant::now();
                let elapsed = now.duration_since(last_update).as_secs_f64();
                
                // If the workers were sleeping, session_bytes == last_bytes, making speed 0.
                let speed = if elapsed > 0.0 { ((session_bytes - last_bytes) as f64 / elapsed) as u64 } else { 0 };
                let progress = if total_segments > 0 { (finished_segments as f32 / total_segments as f32) * 0.95 } else { 0.0 };
                
                let eta = if finished_segments > 0 {
                    let avg = start_time.elapsed().as_secs_f64() / finished_segments as f64;
                    (avg * (total_segments - finished_segments) as f64) as u64
                } else { 0 };

                update_state(session_bytes, finished_segments, speed, progress, eta);
                
                last_update = now;
                last_bytes = session_bytes;
            }
        }
    }

    // FORCE FINAL UPDATE HERE ---
    // The loop has broken because downloads are done. 
    // We must write the final byte count to state.
    let total_elapsed = start_time.elapsed().as_secs_f64();
    let final_speed = if total_elapsed > 0.0 { (session_bytes as f64 / total_elapsed) as u64 } else { 0 };
    
    // Force progress to what we calculated (or 0.99 just before merge)
    let final_progress = if total_segments > 0 { (finished_segments as f32 / total_segments as f32) * 0.99 } else { 0.0 };
    update_state(session_bytes, finished_segments, final_speed, final_progress, 0);

    Ok(())  // Monitor completed successfully!
}

async fn finalize_merge(
    state: AppState,
    job_id: String,
    base_folder: PathBuf,
    save_to_file: PathBuf,
    cancel_token: CancellationToken
) -> Result<MergeOutcome, String> {
    let save_path_str = save_to_file.to_string();

    // Update State -> Merging
    {
        let end_time = time_utils::get_epoch_now();
        if let Ok(mut jobs) = state.download_jobs.lock() {
            if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
                // Calculate final average speed
                let duration = (end_time - job.start_time).max(1);
                job.average_speed = job.downloaded_bytes / duration;
                job.instant_speed = 0;
                job.status = DownloadStatus::Merging;
                job.progress = 0.96;
            }
        }
    }

    // Perform Merge
    match merge_segments(&job_id, &base_folder, &save_path_str, cancel_token.clone()).await {
        Ok(_) => {
            // Success State
            if let Ok(mut jobs) = state.download_jobs.lock() {
                if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
                    job.status = DownloadStatus::CompletedSuccess;
                    job.progress = 1.0;
                    job.end_time = Some(time_utils::get_epoch_now());
                    if let Ok(size) = fs_utils::get_file_size(&save_path_str) {
                        job.final_file_size = Some(size);
                        job.downloaded_bytes = size;
                    }
                }
            }
            println!("Merged successfully: {}", save_path_str);

            if state.with_settings(|s| s.play_completion_sound).unwrap_or(true) {
                audio_utils::play_success_sound();
            }
            
            save_jobs_data(&state).await?;
            Ok(MergeOutcome::Success)
        },
        Err(e) => {
            if cancel_token.is_cancelled() {
                state.with_jobs_mut(|jobs| {
                    if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
                        job.status = DownloadStatus::Paused;
                        job.instant_speed = 0; // Reset speed
                    }
                }).unwrap_or_default();
                
                save_jobs_data(&state).await?;
                write_job_log(&job_id, LogCategory::Merge, LogLevel::Warning, "Merge Job Paused.").await;

                // Return Ok(()) to signal a graceful exit, skipping the error sound
                return Ok(MergeOutcome::Cancelled);
            }

            // Error State
            let err_msg = e.to_string();
            state.with_jobs_mut(|jobs| {
                if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
                    job.status = DownloadStatus::CompletedError(err_msg.clone());
                    job.end_time = Some(time_utils::get_epoch_now());
                }
            }).unwrap_or_default();

            if state.with_settings(|s| s.play_completion_sound).unwrap_or(true) {
                audio_utils::play_error_sound();
            }

            save_jobs_data(&state).await?;
            Err(format!("Merge failed: {}", e))
        }
    }
}

pub async fn download_file_with_retry(
    client: &reqwest::Client,
    url: &str,
    final_path: &PathBuf,
    headers: &HashMap<String, String>,
    is_ts: bool,
    progress_tx: mpsc::Sender<DownloadProgress>,
) -> Result<(), String> {
    // 1. RESUME CHECK: If file exists, report size and skip
    if check_existing_file(final_path, &progress_tx).await {
        return Ok(());
    }

    // 2. PREPARE TEMP PATH (e.g., video.ts -> video.ts.part)
    // We append .part so we don't lose the original extension info
    let mut temp_path = final_path.clone().into_os_string();
    temp_path.push(".part");
    let temp_path = PathBuf::from(temp_path);

    const MAX_RETRIES: u32 = 5;

    // 3. RETRY LOOP
    for attempt in 1..=MAX_RETRIES {
        match perform_single_download(client, url, &temp_path, headers, is_ts, &progress_tx).await {
            Ok(_) => {
                // Atomic Rename on Success: .part -> .ts
                if let Err(e) = tokio_fs::rename(&temp_path, final_path).await {
                    return Err(format!("Failed to rename temp file: {}", e));
                }
                return Ok(());
            },
            Err(e) => {
                // Log failure to UI
                let _ = progress_tx.send(DownloadProgress::Log(
                    LogLevel::Warning,
                    format!("Error on {} (Attempt {}/{}): {}", final_path.display(), attempt, MAX_RETRIES, e)
                )).await;

                handle_failure(attempt, MAX_RETRIES, &e, final_path).await;
            }
        }
    }

    Err(format!("Failed to download {} after {} attempts", final_path.display(), MAX_RETRIES))
}

async fn perform_single_download(
    client: &reqwest::Client,
    url: &str,
    temp_path: &PathBuf,
    headers: &HashMap<String, String>,
    is_ts: bool,
    progress_tx: &mpsc::Sender<DownloadProgress>,
) -> Result<(), String> {
    // A. Build Request
    let mut request = client.get(url);
    for (k, v) in headers {
        request = request.header(k, v);
    }

    // B. Send Request
    let response = request.send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP Error: {}", response.status()));
    }

    // C. Create Temp File
    let mut file = tokio_fs::File::create(temp_path).await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    // D. Stream Loop with Header Stripping
    let mut stream = response.bytes_stream();
    let mut buffer: Vec<u8> = Vec::new();
    let mut header_processed = false;

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;

        if !is_ts || header_processed {
            file.write_all(&chunk).await.map_err(|e| e.to_string())?;
            let _ = progress_tx.send(DownloadProgress::BytesDownloaded(chunk.len() as u64)).await;
        } else {
            buffer.extend_from_slice(&chunk);
            if buffer.len() >= 1024 {
                let offset = m3u8_utils::detect_ts_sync_offset(&buffer);
                file.write_all(&buffer[offset..]).await.map_err(|e| e.to_string())?;
                let written_len = buffer.len().saturating_sub(offset);
                let _ = progress_tx.send(DownloadProgress::BytesDownloaded(written_len as u64)).await;

                header_processed = true;
                buffer.clear();
                buffer.shrink_to_fit();
            }
        }
    }

    // Edge Case: If stream ended but was smaller than our check limit (e.g. tiny file)
    if is_ts && !header_processed && !buffer.is_empty() {
        let offset = m3u8_utils::detect_ts_sync_offset(&buffer);
        file.write_all(&buffer[offset..]).await.map_err(|e| e.to_string())?;
        let _ = progress_tx.send(DownloadProgress::BytesDownloaded((buffer.len() - offset) as u64)).await;
    }

    // E. Flush to ensure safety
    file.flush().await.map_err(|e| e.to_string())?;

    Ok(())
}

async fn check_existing_file(path: &PathBuf, progress_tx: &mpsc::Sender<DownloadProgress>) -> bool {
    if let Ok(metadata) = tokio_fs::metadata(path).await {
        let size = metadata.len();
        if size > 0 {
            // Tell the UI we "downloaded" this already, so the Total Progress bar is correct.
            let _ = progress_tx.send(DownloadProgress::BytesDownloaded(size)).await;
            return true;
        }
    }
    false
}

/// Helper to calculate backoff and print logs
async fn handle_failure(attempt: u32, max_retries: u32, error: &str, path: &PathBuf) {
    if attempt < max_retries {
        let backoff: u64 = 2u64.pow(attempt - 1).min(8);    // 1s 2s 4s 8s 8s 8s...
        eprintln!("[!] {:?}: {} (retry {}/{} in {}s)", path.file_name(), error, attempt, max_retries, backoff);
        sleep(Duration::from_secs(backoff)).await;
    }
}

pub async fn merge_segments(
    job_id: &str,
    base_folder: &PathBuf, 
    save_to_file: &str,
    cancel_token: CancellationToken,
) -> Result<(), String> {
    // 1. Preparation
    let files = fs_utils::get_sorted_files(base_folder, &["mp4", "m4s", "ts"]).await?;
    if files.is_empty() {
        return Err("No video segments found to merge".to_string());
    }

    write_job_log(job_id, LogCategory::Merge, LogLevel::Info, &format!("Merging {} segments into {:?}", files.len(), save_to_file)).await;

    // 2. Strategy Selection
    let has_init_segment = files.iter().any(|p| 
        p.file_name().and_then(|n| n.to_str()).map(|s| s == "init.mp4").unwrap_or(false)
    );

    if has_init_segment {
        merge_strategy_binary(job_id, base_folder, &files, save_to_file, cancel_token).await
    } else {
        merge_strategy_concat(job_id, base_folder, &files, save_to_file, cancel_token).await
    }
}

macro_rules! build_args {
    ($($arg:expr),* $(,)?) => {
        vec![
            $($arg.to_string()),*
        ]
    };
}

async fn merge_strategy_binary(
    job_id: &str,
    base_folder: &PathBuf,
    files: &[PathBuf],
    save_to_file: &str,
    cancel_token: CancellationToken
) -> Result<(), String> {
    write_job_log(job_id, LogCategory::Merge, LogLevel::Info, "Strategy: Binary Merge (Fragmented MP4)").await;

    let temp_merged_path = base_folder.join("temp_merged.mp4");

    fs_utils::concatenate_files(files, &temp_merged_path).await?;

    let args = build_args!(
        "-hide_banner",
        "-y",
        "-i", temp_merged_path.to_string_lossy(),
        "-c", "copy",
        "-movflags", "+faststart",
        save_to_file
    );

    let result = run_ffmpeg_command(args, job_id, cancel_token).await;
    result
}

async fn merge_strategy_concat(
    job_id: &str,
    base_folder: &PathBuf,
    files: &[PathBuf],
    save_to_file: &str,
    cancel_token: CancellationToken,
) -> Result<(), String> {
    write_job_log(job_id, LogCategory::Merge, LogLevel::Info, "Strategy: FFmpeg Concat Demuxer").await;

    // 1. Generate List
    let list_path = base_folder.join("segments.txt");
    // let content = files.iter()
    //     .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
    //     .map(|name| format!("file '{}'\n", name))
    //     .collect::<String>();
    let content = files.iter()
        .map(|p| {
            // FFmpeg prefers forward slashes, even on Windows
            let path_str = p.to_string_lossy().replace('\\', "/");
            format!("file '{}'\n", path_str)
        })
        .collect::<String>();

    fs_utils::write_file_async(&list_path, &content).await?;

    let args = build_args!(
        "-hide_banner",
        "-f", "concat",
        "-safe", "0",
        "-i", list_path.to_string_lossy(),
        "-c", "copy",
        "-movflags", "+faststart",
        "-y", save_to_file
    );

    run_ffmpeg_command(args, job_id, cancel_token).await
}

async fn run_ffmpeg_command(
    args: Vec<String>,
    job_id: &str,
    cancel_token: CancellationToken,
) -> Result<(), String> {
    let display_args: Vec<String> = args.iter()
        .map(|arg| {
            if arg.contains(' ') { format!("\"{}\"", arg) } else { arg.to_string() }
        })
        .collect();
    let cmd_str = format!("ffmpeg {}", display_args.join(" "));
    write_job_log(job_id, LogCategory::Merge, LogLevel::Info, &format!("Running command: {}", cmd_str)).await;

    // 1. Spawn the Sidecar (Notice the added map_err and ?)
    let (mut rx, child) = Command::new_sidecar("ffmpeg")
        .map_err(|e| format!("Failed to initialize ffmpeg sidecar: {}", e))?
        .args(args)
        .spawn()
        .map_err(|e| format!("Failed to spawn ffmpeg sidecar: {}", e))?;

    let job_id_clone = job_id.to_string();

    // 2. Process events and handle cancellation simultaneously
    tokio::select! {
        result = async {
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Stderr(line) | CommandEvent::Stdout(line) => {
                        write_job_log(&job_id_clone, LogCategory::Merge, LogLevel::Debug, &format!("[FFmpeg] {}", line)).await;
                    }
                    CommandEvent::Error(err) => {
                        return Err(format!("FFmpeg execution error: {}", err));
                    }
                    CommandEvent::Terminated(payload) => {
                        if payload.code == Some(0) {
                            return Ok(());
                        } else {
                            return Err(format!("FFmpeg exited with error code: {:?}", payload.code));
                        }
                    }
                    _ => {}
                }
            }
            Err("FFmpeg process channel closed unexpectedly".to_string())
        } => {
            match result {
                Ok(_) => {
                    write_job_log(job_id, LogCategory::Merge, LogLevel::Info, "Merge successful.").await;
                    Ok(())
                }
                Err(e) => {
                    write_job_log(job_id, LogCategory::Merge, LogLevel::Error, &e).await;
                    Err(e)
                }
            }
        }

        // Branch B: App paused/exited via cancellation token
        _ = cancel_token.cancelled() => {
            let _ = child.kill();
            write_job_log(job_id, LogCategory::Merge, LogLevel::Warning, "Merge aborted due to app pause/exit.").await;
            Err("MERGE_CANCELLED".into())
        }
    }
}