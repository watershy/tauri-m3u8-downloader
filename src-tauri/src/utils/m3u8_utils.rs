use crate::types::*;

pub async fn parse_m3u8_content(m3u8_content: &str, url: &str)-> Result<PlaylistData, String> {
    //println!("M3U8 file content: {}", m3u8_content);
    match get_playlist_type(&m3u8_content) {
        PlaylistType::Master => parse_master_playlist(&m3u8_content, url).await.map(PlaylistData::Master),
        PlaylistType::Media => parse_media_playlist(&m3u8_content, url).await.map(PlaylistData::Media),
        PlaylistType::Unknown => Err("Unknown or unsupported M3U8 file format".to_string()),
    }
}

pub fn get_playlist_type(m3u8_content: &str) -> PlaylistType {
    if m3u8_content.contains("#EXT-X-STREAM-INF") {
        PlaylistType::Master
    } else if m3u8_content.contains("#EXTINF") {
        PlaylistType::Media
    } else {
        PlaylistType::Unknown
    }
}

pub async fn parse_master_playlist(content: &str, url: &str) -> Result<M3U8MasterPlaylist, String> {
    M3U8MasterPlaylist::parse(content, url)
        .map_err(|e| format!("Failed to parse master playlist from {}: {}", url, e))
}

pub async fn parse_media_playlist(content: &str, url: &str) -> Result<M3U8MediaPlaylist, String> {
    M3U8MediaPlaylist::parse(content, url)
        .map_err(|e| format!("Failed to parse media playlist: {}", e))
}

// Helper: Scans for the MPEG-TS sync byte (0x47) repeating every 188 bytes
pub fn detect_ts_sync_offset(data: &[u8]) -> usize {
    // Only scan the first 4KB. If valid data isn't found by then, assume offset 0.
    let scan_limit = std::cmp::min(data.len(), 4096);
    
    // We need at least 3 packets (376 bytes range) to confirm the pattern
    if scan_limit < 377 { return 0; }

    for i in 0..(scan_limit - 376) {
        // Check for 0x47 at index i, i+188, and i+376
        if data[i] == 0x47
            && data[i + 188] == 0x47
            && data[i + 376] == 0x47
        {
            return i; // Found the real start!
        }
    }

    0 // Pattern not found, assume standard file (no offset)
}