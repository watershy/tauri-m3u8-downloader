#[derive(serde::Serialize)]
pub struct LogUpdate {
    pub logs: String,
    pub new_offset: u64,
}
