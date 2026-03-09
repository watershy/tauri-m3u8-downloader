use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct ResolutionOption {
    pub id: usize,
    pub resolution: String,
    pub bandwidth_kbps: u32,
    pub uri: String,
}

#[derive(Serialize)]
pub struct CheckMediaResult {
    pub success: bool,
    pub message: String,
    pub total_segments: u32,
    pub first_segment_url: String,
    pub save_folder: String,
    pub suggested_filename: String,
    pub resolutions: Option<Vec<ResolutionOption>>,
}

impl CheckMediaResult {
    pub fn success(
        total_segments: u32,
        first_segment_url: String,
        save_folder: String,
        suggested_filename: String,
        resolutions: Option<Vec<ResolutionOption>>) -> Self {
        Self {
            success: true,
            message: String::new(),
            total_segments: total_segments,
            first_segment_url: first_segment_url,
            save_folder: save_folder,
            suggested_filename: suggested_filename,
            resolutions,
        }
    }

    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message: message,
            total_segments: 0,
            first_segment_url: String::new(),
            save_folder: String::new(),
            suggested_filename: String::new(),
            resolutions: None,
        }
    }
}