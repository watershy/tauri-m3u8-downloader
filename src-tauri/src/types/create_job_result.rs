use serde::Serialize;

#[derive(Serialize)]
pub struct CreateJobResult {
    pub success: bool,
    pub message: String,
}
