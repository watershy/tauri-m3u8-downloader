#[derive(Debug, Clone, Copy)]
pub enum TrackType {
    Video,
    Audio,
}

impl TrackType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TrackType::Video => "video",
            TrackType::Audio => "audio",
        }
    }
}