use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::types::LogLevel;
use crate::utils::time_utils;

#[derive(Debug)]
pub enum DownloadProgress {
    BytesDownloaded(u64),      // A chunk of data arrived
    SegmentFinished(bool),     // bool = success or failure
    Log(LogLevel, String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Merging,
    Paused,
    CompletedSuccess,
    CompletedError(String),
}

impl DownloadStatus {
    pub fn is_ongoing(&self) -> bool {
        match self {
            DownloadStatus::Queued | 
            DownloadStatus::Downloading | 
            DownloadStatus::Merging | 
            DownloadStatus::Paused => true,
            DownloadStatus::CompletedSuccess | 
            DownloadStatus::CompletedError(_) => false,
        }
    }

    pub fn can_resume(&self) -> bool {
        match self {
            DownloadStatus::Paused | 
            DownloadStatus::CompletedError(_) => true,
            _ => false,
        }
    }

    pub fn can_pause(&self) -> bool {
        match self {
            DownloadStatus::Downloading |
            DownloadStatus::Merging |
            DownloadStatus::Queued => true,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownloadJob {
    pub id: String,
    pub m3u8_url: String,
    pub http_headers: HashMap<String, String>,
    pub file_name: String,
    pub save_folder: String,
    //pub video_type: String, // "HLS", "MP4", etc.
    pub status: DownloadStatus,

    pub downloaded_bytes: u64,      // Sum of already downloaded bytes
    pub total_segments: u32,        // Total count (e.g., 150 ts files)
    pub downloaded_segments: u32,   // Count of completed (e.g., 45)
    pub final_file_size: Option<u64>,

    pub instant_speed: u64,             // Bytes per second
    pub smoothed_speed: f64,            // Exponential Moving Average
    pub average_speed: u64,
    pub eta: u64,             // Seconds remaining
    pub progress: f32,
    pub start_time: u64,
    pub end_time: Option<u64>,
}

impl Default for DownloadJob {
    fn default() -> Self {
        DownloadJob {
            id: String::new(),
            file_name: String::new(),
            m3u8_url: String::new(),
            http_headers: HashMap::new(),
            save_folder: String::new(),
            //video_type: String::from("mp4"),
            status: DownloadStatus::Queued,
            downloaded_bytes: 0,
            total_segments: 0,
            downloaded_segments: 0,
            final_file_size: None,
            instant_speed: 0,
            smoothed_speed: 0.0,
            average_speed: 0,
            eta: 0,
            progress: 0.0,
            start_time: 0,
            end_time: None,
        }
    }
}

impl DownloadJob {
    pub fn new(url: String, http_headers: HashMap<String, String>, save_folder: String, file_name: String, total_segments: u32) -> Self {
        DownloadJob {
            id: Uuid::new_v4().to_string(),
            file_name,
            m3u8_url: url,
            http_headers: http_headers,
            save_folder,
            //video_type: "video/mp4".to_string(),
            status: DownloadStatus::Downloading,
            downloaded_bytes: 0,
            total_segments,
            downloaded_segments: 0,
            final_file_size: None,
            instant_speed: 0,
            smoothed_speed: 0.0,
            average_speed: 0,
            eta: 0,
            progress: 0.0,
            start_time: time_utils::get_epoch_now(),
            end_time: None,
        }
    }
}