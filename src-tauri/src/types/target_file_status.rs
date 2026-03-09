#[derive(serde::Serialize)]
pub enum TargetFileStatus {
    Ready,
    Exists,
    Busy,   // Locked by an active download
}