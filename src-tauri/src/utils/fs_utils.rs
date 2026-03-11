use std::env;
use std::fs as std_fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use open::that;
use tokio::fs as tokio_fs;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncSeekExt, SeekFrom};
use tauri::api::path::download_dir;
use directories::ProjectDirs;
use crate::constants;
use crate::types::DownloadJob;
use crate::types::LogCategory;
use super::LogErr;
use super::IntoString;

/* #region Get folders */

pub fn get_env_temp_dir() -> Result<PathBuf, String> {
    // Windows: C:\Users\You\AppData\Local\Temp
    // Mac/Linux: /tmp or /var/folders/...
    let mut path = env::temp_dir();
    path.push(constants::APP_NAME);
    Ok(path)
}

fn get_proj_dir() -> Result<ProjectDirs, String> {
    ProjectDirs::from("", "", constants::APP_NAME)
        .ok_or_else(|| "Could not resolve app directory".to_string())
}

pub fn get_data_root_dir() -> Result<PathBuf, String> {
    let proj_dirs = get_proj_dir()?;
    let data_dir = proj_dirs.data_local_dir();
    ensure_directory_exists(&data_dir)?;
    Ok(data_dir.to_path_buf())
}

pub fn get_job_dir(job_id: &str) -> Result<PathBuf, String> {
    let job_dir = get_data_root_dir()?.join("jobs").join(job_id);
    ensure_directory_exists(&job_dir)?;
    Ok(job_dir)
}

pub fn get_logs_dir() -> Result<PathBuf, String> {
    let logs_dir = get_data_root_dir()?.join("logs");
    ensure_directory_exists(&logs_dir)?;
    Ok(logs_dir)
}

pub fn get_jobs_db_file_path() -> Result<PathBuf, String> {
    Ok(get_data_root_dir()?.join("jobs.json"))
}

pub fn get_job_log_file_path(job_id: &str, log_category: LogCategory) -> Result<PathBuf, String> {
    Ok(get_job_dir(job_id)?.join(format!("{}_{}.log", job_id, log_category.as_str())))
}

pub fn get_job_m3u8_backup_file_path(job_id: &str) -> Result<PathBuf, String> {
    Ok(get_job_dir(job_id)?.join(format!("{}.m3u8", job_id)))
}

pub fn get_settings_file_path() -> Result<PathBuf, String> {
    let proj_dirs = get_proj_dir()?;
    let config_dir = proj_dirs.config_local_dir();
    ensure_directory_exists(&config_dir)?;
    Ok(config_dir.join("settings.json"))
}

pub fn get_download_dir(job_id: &str) -> Result<PathBuf, String> {
    let base_temp = get_env_temp_dir()?;
    let job_path = base_temp.join(job_id);
    ensure_directory_exists(&job_path)?;
    Ok(job_path)
}

/* #endregion */

/* #region file IO */

pub async fn create_file_async<P: AsRef<Path>>(path: P) -> Result<tokio_fs::File, String> {
    tokio_fs::File::create(path)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))
}

pub fn read_text_from_file<P: AsRef<Path>>(path: P) -> Result<String, String> {
    std_fs::read_to_string(path).map_err(|e| e.to_string())
}

pub async fn read_text_from_file_async<P: AsRef<Path>>(path: P) -> Result<String, String> {
    let path_ref = path.as_ref();
    tokio_fs::read_to_string(path_ref)
        .await
        .map_err(|e| format!("Failed to read file '{:?}': {}", path_ref, e))
}

pub async fn read_text_from_offset_async<P: AsRef<Path>>(path: P, offset: u64) -> Result<(String, u64), String> {
    let path = path.as_ref();

    let mut file = tokio_fs::File::open(path)
        .await
        .map_err(|e| format!("Failed to open file '{}': {}", path.display(), e))?;

    let file_len = file.metadata().await.map_err(|e| e.to_string())?.len();
    if offset >= file_len {
        return Ok((String::new(), file_len));
    }

    file.seek(SeekFrom::Start(offset))
        .await
        .map_err(|e| format!("Failed to seek in file '{}': {}", path.display(), e))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .await
        .map_err(|e| format!("Failed to read file '{}': {}", path.display(), e))?;

    let new_pos = offset + buffer.len() as u64;
    let content = String::from_utf8_lossy(&buffer).to_string();
    Ok((content, new_pos))
}

pub fn ensure_directory_exists<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let path_ref = path.as_ref();
    std_fs::create_dir_all(path_ref)
        .map_err(|e| format!("Failed to create directory at '{:?}': {}", path_ref, e))
}

pub async fn ensure_directory_exists_async<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let path_ref = path.as_ref();
    tokio_fs::create_dir_all(path_ref)
        .await
        .map_err(|e| format!("Failed to create directory at '{:?}': {}", path_ref, e))?;
    Ok(())
}

pub fn write_file_sync<P: AsRef<Path>>(path: P, contents: &str) -> Result<(), String> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        ensure_directory_exists(parent)?; 
    }

    match std_fs::write(path, contents) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to write to file '{}': {}", path.display(), e)),
    }
}

pub async fn write_file_async<P: AsRef<Path>>(path: P, contents: &str) -> Result<(), String> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        ensure_directory_exists_async(parent).await?; 
    }

    tokio_fs::write(path, contents).await
        .map_err(|e| format!("Failed to write file: {}", e)).log_err()
}

pub async fn append_to_file_async<P: AsRef<Path>>(path: P, content: &str) -> Result<(), String> {
    let path = path.as_ref();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .map_err(|e| format!("Failed to open file: {}", e))?;
    file.write_all(content.as_bytes())
        .await
        .map_err(|e| format!("Failed to append to file {}: {}", path.display(), e))?;
    Ok(())
}

pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<(), String> {
    match std_fs::remove_file(&path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            // We use .as_ref().display() safely print the generic path type
            println!("No file found at: {}", path.as_ref().display());
            Ok(())
        },
        Err(e) => Err(e.to_string()),
    }
}

pub async fn remove_file_async<P: AsRef<Path>>(path: P) -> Result<(), String> {
    match tokio_fs::remove_file(path).await {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            Ok(())
        },
        Err(e) => Err(e.to_string()),
    }
}

pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<(), String> {
    match std_fs::remove_dir_all(&path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            println!("No directory found at: {}", path.as_ref().display());
            Ok(())
        },
        Err(e) => Err(e.to_string()),
    }
}

pub async fn remove_dir_all_async<P: AsRef<Path>>(path: P) -> Result<(), String> {
    match tokio_fs::remove_dir_all(path).await {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            Ok(())
        },
        Err(e) => Err(e.to_string()),
    }
}

pub async fn rename_file_async<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<(), String> {
    tokio_fs::rename(from, to)
        .await
        .map_err(|e| format!("Failed to rename temp file: {}", e))
}

pub fn get_file_size(file_path: &str) -> Result<u64, String> {
    let metadata = std_fs::metadata(&file_path).map_err(|e| e.to_string())?;
    Ok(metadata.len())
}

pub async fn get_file_size_async<P: AsRef<Path>>(path: P) -> Result<u64, String> {
    tokio_fs::metadata(path)
        .await
        .map(|metadata| metadata.len())
        .map_err(|e| format!("Failed to read file metadata: {}", e))
}

/* #endregion */

pub fn open_folder(folder_path: &str) -> Result<(), String> {
    that(folder_path).map_err(|e| e.to_string())?;
    Ok(())
}

// pub fn join_paths(base: &Path, sub: &str) -> PathBuf {
//     base.join(sub)
// }

pub async fn get_sorted_files(dir: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>, String> {
    let mut entries = tokio_fs::read_dir(dir).await.map_err(|e| e.to_string())?;
    let mut files = Vec::new();

    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let path = entry.path();
        // 1. Get extension, convert to str
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            // 2. Check if the extension is in our allowed list
            if extensions.contains(&ext) {
                files.push(path);
            }
        }
    }

    files.sort();
    
    Ok(files)
}

pub fn resolve_unique_filename(folder_path: &str, url: &str, active_paths: &[PathBuf]) -> String {
    // 1. Determine the target folder (User provided OR OS Default)
    let folder = if folder_path.is_empty() {
        download_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string())
    } else {
        folder_path.to_string() // Simple clone, easy to understand
    };

    // 2. Extract a safe filename from the URL
    // e.g. "http://site.com/movie.m3u8" -> "movie.mp4"
    let url_no_query = url.split('?').next().unwrap_or(url);
    let url_path = Path::new(url_no_query);
    let mut file_stem = url_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("video"); // Default if URL ends in /

    if file_stem.to_lowercase().ends_with(".mp4") {
        file_stem = &file_stem[..file_stem.len() - 4];
    }

    // We assume the output will be .mp4 for video downloads
    let base_filename = format!("{}.mp4", file_stem);

    // 3. Loop until we find a free name
    let mut counter = 0;
    loop {
        let candidate_name = if counter == 0 {
            base_filename.clone()
        } else {
            format!("{} ({}).mp4", file_stem, counter)
        };

        let full_path = Path::new(&folder).join(&candidate_name);

        let exists_on_disk = full_path.exists();
        let claimed_by_active_job = active_paths.contains(&full_path);
        if !exists_on_disk && !claimed_by_active_job {
            return candidate_name; 
        }

        counter += 1;
    }
}

pub async fn save_jobs_data(json_content: &str) -> Result<(), String> {
    let jobs_path: PathBuf = get_jobs_db_file_path()?;
    write_file_async(&jobs_path, json_content).await
}

pub fn load_jobs_from_disk()-> Vec<DownloadJob> {
    // 1. Get path. If error, return empty list (first run)
    let path = match get_jobs_db_file_path() {
        Ok(p) => p,
        Err(_) => return Vec::new(), 
    };

    if !path.exists() {
        return Vec::new();
    }

    // 2. Read and Deserialize
    // We use standard fs because this runs once at startup (blocking is fine/expected here)
    match std_fs::read_to_string(&path) {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_else(|e| {
                eprintln!("Failed to parse jobs.json: {}", e);
                Vec::new() // If file is corrupted, start fresh
            })
        },
        Err(_) => Vec::new(),
    }
}

pub async fn save_m3u8_content(job_id: &str, content: &str) -> Result<String, String> {
    let path = get_job_m3u8_backup_file_path(job_id)?;
    write_file_async(&path, content).await?;
    Ok(path.to_string())
}

pub async fn concatenate_files(input_paths: &[PathBuf], output_path: &PathBuf) -> Result<(), String> {
    let mut output_file = tokio_fs::File::create(output_path).await
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    for path in input_paths {
        let mut input_file = tokio_fs::File::open(path).await
            .map_err(|e| format!("Failed to open input {}: {}", path.display(), e))?;
            
        tokio::io::copy(&mut input_file, &mut output_file).await
            .map_err(|e| format!("Failed to copy content: {}", e))?;
    }
    
    output_file.flush().await.map_err(|e| e.to_string())?;
    Ok(())
}