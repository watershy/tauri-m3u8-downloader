#[derive(Copy, Clone)]
pub enum LogCategory {
    General,
    Download,
    Merge,
}

impl LogCategory {
    pub const ALL: [LogCategory; 3] = [
        LogCategory::General,
        LogCategory::Download,
        LogCategory::Merge,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            LogCategory::General => "general",
            LogCategory::Download => "download",
            LogCategory::Merge => "merge",
        }
    }
}