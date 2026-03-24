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
                let audios: Vec<AudioOption> = master_data.media_renditions
                    .iter()
                    .filter(|m| m.r#type == "AUDIO" && m.uri.is_some())
                    .enumerate()
                    .map(|(index, m)| {
                        // Safely unwrap the URI since we filtered for is_some()
                        let raw_uri = m.uri.clone().unwrap();

                        // Make sure to resolve it against the master playlist URL.
                        let full_uri = crate::utils::string_utils::resolve_m3u8_url(url, raw_uri);

                        AudioOption {
                            id: index,
                            name: m.name.clone(),
                            language: m.language.clone().unwrap_or_else(|| "Unknown".to_string()),
                            uri: full_uri,
                        }
                    })
                    .collect();

                // 2. Wrap it in an Option
                let audio_options = if audios.is_empty() { None } else { Some(audios) };

                Ok(CheckMediaResult::success(
                    0,
                    String::new(),
                    save_folder,
                    suggested_file_name,
                    Some(master_data.resolution_options),
                    audio_options,
                ))
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
                    None,
                None))
            }
        },
        Err(e) => {
            println!("Error: {}", e);
            Ok(CheckMediaResult::failure(format!("Error: {}", e)))
        }
    }
}