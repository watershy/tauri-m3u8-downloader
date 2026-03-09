use serde::Serialize;
use super::logging_utils::LogErr;

pub fn serialize<T>(value: &T) -> Result<String, String> 
where T: Serialize + ?Sized 
{
    serde_json::to_string_pretty(value).map_err(|e| format!("JSON Serialization Error: {}", e)).log_err()
}