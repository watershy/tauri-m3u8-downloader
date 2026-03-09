use std::sync::{Arc, Mutex, MutexGuard};
use super::DownloadJob;
use super::AppSettings;

use tokio_util::sync::CancellationToken;
use dashmap::DashMap;

#[derive(Clone)]
pub struct AppState {
    pub download_jobs: Arc<Mutex<Vec<DownloadJob>>>,
    pub settings: Arc<Mutex<AppSettings>>,
    pub active_tasks: Arc<DashMap<String, CancellationToken>>,
    pub default_download_path: String,
}

impl AppState {
    pub fn new(download_jobs: Vec<DownloadJob>, initial_settings: AppSettings, default_download_path: String) -> Self {
        AppState {
            download_jobs: Arc::new(Mutex::new(download_jobs)),
            settings: Arc::new(Mutex::new(initial_settings)),
            active_tasks: Arc::new(DashMap::new()),
            default_download_path: default_download_path,
        }
    }

    fn guard_jobs(&self) -> Result<MutexGuard<Vec<DownloadJob>>, String> {
        self.download_jobs.lock()
            .map_err(|e| format!("Failed to lock jobs: {}", e))
    }

    fn guard_settings(&self) -> Result<MutexGuard<AppSettings>, String> {
        self.settings.lock()
            .map_err(|e| format!("Failed to lock settings: {}", e))
    }

    pub fn with_jobs<F, R>(&self, f: F) -> Result<R, String>
    where F: FnOnce(&Vec<DownloadJob>) -> R {
        let jobs = self.guard_jobs()?;
        Ok(f(&jobs))
    }

    pub fn with_jobs_mut<F, R>(&self, f: F) -> Result<R, String>
    where F: FnOnce(&mut Vec<DownloadJob>) -> R {
        let mut jobs = self.guard_jobs()?;
        Ok(f(&mut jobs))
    }

    pub fn with_settings<F, R>(&self, f: F) -> Result<R, String>
    where F: FnOnce(&AppSettings) -> R {
        let settings = self.guard_settings()?;
        Ok(f(&settings))
    }

    pub fn with_settings_mut<F, R>(&self, f: F) -> Result<R, String>
    where F: FnOnce(&mut AppSettings) -> R {
        let mut settings = self.guard_settings()?;
        Ok(f(&mut settings))
    }
}