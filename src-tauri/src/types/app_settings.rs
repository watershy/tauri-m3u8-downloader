use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppSettings {
    pub download_path: String,
    pub play_completion_sound: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            download_path: String::new(),
            play_completion_sound: true,
        }
    }
}