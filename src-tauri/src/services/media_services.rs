use std::path::PathBuf;
use std::collections::HashMap;
use crate::types::*;
use crate::utils::*;

pub async fn check_media(
    url: &str,
    headers: &HashMap<String, String>,
    save_folder: String,
    active_paths: &[PathBuf],
) -> Result<CheckMediaResult, String> {
    let suggested_file_name = fs_utils::resolve_unique_filename(&save_folder, url, active_paths);
    let m3u8_content = match network_utils::fetch_http_content(&url, &headers).await {
        Ok(content) => content,
        Err(e) => {
            return Ok(CheckMediaResult::failure(e));
        }
    };

    match m3u8_utils::parse_m3u8_content(&m3u8_content, &url).await {
        Ok(data) => match data {
            PlaylistData::Master(master_data) => {
                Ok(CheckMediaResult::success(
                    0,
                    String::new(),
                    save_folder,
                    suggested_file_name,
                    Some(master_data.resolution_options)))
            },
            PlaylistData::Media(media_data) => {
                if !media_data.first_segment_url.is_empty() {
                    if let Err(e) = network_utils::validate_http_file_access(
                        &media_data.first_segment_url,
                        headers
                    ).await {
                        // Return a "Soft Failure" so the UI can display the message
                        return Ok(CheckMediaResult::failure(
                            format!("Playlist is valid, but cannot access media segments: {}", e)
                        ));
                    }
                }

                Ok(CheckMediaResult::success(
                    media_data.total_segments,
                    media_data.first_segment_url,
                    save_folder,
                    suggested_file_name,
                    None))
            }
        },
        Err(e) => {
            println!("Error: {}", e);
            Ok(CheckMediaResult::failure(format!("Error: {}", e)))
        }
    }
}