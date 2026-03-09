
use serde::{Deserialize, Serialize};
use url::Url;
use crate::utils::string_utils;
use super::ResolutionOption;

pub enum PlaylistType {
    Master,
    Media,
    Unknown,
}
pub enum PlaylistData {
    Master(M3U8MasterPlaylist),
    Media(M3U8MediaPlaylist),
}

#[derive(Debug)]
pub struct M3U8MasterPlaylist {
    pub version: Option<u8>,                    // Represents EXT-X-VERSION
    pub variant_streams: Vec<VariantStream>,    // List of EXT-X-STREAM-INF entries
    pub media_renditions: Vec<MediaRendition>,  // List of EXT-X-MEDIA entries
    pub iframe_streams: Vec<IFrameStream>,      // List of EXT-X-I-FRAME-STREAM-INF entries
    pub resolution_options: Vec<ResolutionOption>,
}

#[derive(Debug)]
pub struct VariantStream {
    pub bandwidth: u32,                  // BANDWIDTH attribute
    pub resolution: Option<String>,      // RESOLUTION attribute
    pub codecs: Option<String>,          // CODECS attribute
    pub uri: String,                     // URI of the variant stream
    pub audio_group: Option<String>,     // AUDIO group reference
    pub video_group: Option<String>,     // VIDEO group reference
    pub subtitles_group: Option<String>, // SUBTITLES group reference
}

#[derive(Debug)]
pub struct MediaRendition {
    pub r#type: String,                  // AUDIO, VIDEO, SUBTITLES, etc.
    pub group_id: String,                // GROUP-ID attribute
    pub name: String,                    // NAME attribute
    pub default: bool,                   // DEFAULT attribute
    pub autoselect: bool,                // AUTOSELECT attribute
    pub language: Option<String>,        // LANGUAGE attribute
    pub uri: Option<String>,             // URI for the rendition (optional)
}

#[derive(Debug)]
pub struct IFrameStream {
    pub bandwidth: u32,                  // BANDWIDTH attribute
    pub resolution: Option<String>,      // RESOLUTION attribute
    pub codecs: Option<String>,          // CODECS attribute
    pub uri: String,                     // URI of the I-frame stream
}

impl M3U8MasterPlaylist {
    pub fn parse(content: &str, master_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut playlist = M3U8MasterPlaylist {
            version: None,
            variant_streams: Vec::new(),
            media_renditions: Vec::new(),
            iframe_streams: Vec::new(),
            resolution_options: Vec::new(),
        };
    
        let lines: Vec<&str> = content.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
        for (index, line) in lines.iter().enumerate() {
            match *line {
                l if l.starts_with("#EXT-X-VERSION") => {
                    playlist.parse_version(l)?;
                }
                l if l.starts_with("#EXT-X-STREAM-INF") => {
                    playlist.parse_stream_inf(l, &lines, index, master_url)?;
                }
                l if l.starts_with("#EXT-X-MEDIA") => {
                    playlist.parse_media(l)?;
                }
                l if l.starts_with("#EXT-X-I-FRAME-STREAM-INF") => {
                    playlist.parse_iframe_stream_inf(l)?;
                }
                _ => {}
            }
        }

        playlist.resolution_options = playlist.variant_streams
            .iter()
            .enumerate()
            .map(|(index, stream)| ResolutionOption {
                id: index,
                // Fallback to "Unknown" if resolution attribute is missing
                resolution: stream.resolution.clone().unwrap_or_else(|| "Unknown".to_string()),
                bandwidth_kbps: stream.bandwidth / 1000,
                uri: stream.uri.clone(),
            })
            .collect();

        Ok(playlist)
    }
    

    fn parse_version(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.version = Some(line.split(':').nth(1).unwrap().parse()?);
        Ok(())
    }

    fn parse_stream_inf(&mut self, line: &str, lines: &[&str], current_index: usize, master_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let attributes = string_utils::parse_attributes(line)?;
        // Fetch the URI directly from the next line in the pre-parsed lines
        let uri = lines
            .get(current_index + 1) // Get the next line after the current line
            .ok_or("Missing URI for EXT-X-STREAM-INF")?
            .to_string();

        let full_uri = string_utils::resolve_m3u8_url(master_url, uri);
    
        self.variant_streams.push(VariantStream {
            bandwidth: attributes.get("BANDWIDTH").ok_or("BANDWIDTH missing")?.parse()?,
            resolution: attributes.get("RESOLUTION").map(|s| s.to_string()),
            codecs: attributes.get("CODECS").map(|s| string_utils::map_codec(Some(s))),
            uri: full_uri,
            audio_group: attributes.get("AUDIO").map(|s| s.to_string()),
            video_group: attributes.get("VIDEO").map(|s| s.to_string()),
            subtitles_group: attributes.get("SUBTITLES").map(|s| s.to_string()),
        });
    
        Ok(())
    }

    fn parse_media(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        let attributes = string_utils::parse_attributes(line)?;
        self.media_renditions.push(MediaRendition {
            r#type: attributes.get("TYPE").ok_or("TYPE missing")?.to_string(),
            group_id: attributes.get("GROUP-ID").ok_or("GROUP-ID missing")?.to_string(),
            name: attributes.get("NAME").ok_or("NAME missing")?.to_string(),
            default: attributes.get("DEFAULT").map_or(false, |v| v == "YES"),
            autoselect: attributes.get("AUTOSELECT").map_or(false, |v| v == "YES"),
            language: attributes.get("LANGUAGE").map(|s| s.to_string()),
            uri: attributes.get("URI").map(|s| s.to_string()),
        });
        Ok(())
    }

    fn parse_iframe_stream_inf(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        let attributes = string_utils::parse_attributes(line)?;
        self.iframe_streams.push(IFrameStream {
            bandwidth: attributes.get("BANDWIDTH").ok_or("BANDWIDTH missing")?.parse()?,
            resolution: attributes.get("RESOLUTION").map(|s| s.to_string()),
            codecs: attributes.get("CODECS").map(|s| string_utils::map_codec(Some(s))),
            uri: attributes.get("URI").ok_or("URI missing")?.to_string(),
        });
        Ok(())
    }
}

#[derive(Debug)]
pub struct M3U8MediaPlaylist {
    pub version: Option<u8>,
    pub target_duration: u32,
    pub media_sequence: Option<u32>,
    pub playlist_type: Option<String>,
    pub is_end_list: bool,
    pub segments: Vec<MediaSegment>,
    pub init_map_file: Option<MediaSegment>,
    pub first_segment_url: String,
    pub total_segments: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaSegment {
    pub url: String,                // Segment file url
    pub sequence: u32,              // 0, 1, 2... order is important for stitching later
    pub duration: f64,              // Parsed from m3u8 file
    pub is_discontinuity: bool,     // No use
    pub downloaded: bool,           // Indicate if the segment is downloaded or not
    pub size_bytes: u64,            // Size of segment file (after download)
}

impl M3U8MediaPlaylist {
    /// Parses the content of a media-type M3U8 playlist
    pub fn parse(content: &str, m3u8_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut playlist = M3U8MediaPlaylist {
            version: None,
            target_duration: 0,
            media_sequence: None,
            playlist_type: None,
            is_end_list: false,
            segments: Vec::new(),
            init_map_file: None,
            first_segment_url: String::new(),
            total_segments: 0,
        };

        let base_url = Url::parse(m3u8_url).map_err(|e| format!("Invalid base URL: {}", e))?;
        let mut current_segment: Option<MediaSegment> = None;
        let mut index = 0;
        for line in content.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
            if line.starts_with("#EXT-X-VERSION") {
                playlist.version = Some(line.split(':').nth(1).unwrap().parse()?);
            } else if line.starts_with("#EXT-X-TARGETDURATION") {
                playlist.target_duration = line.split(':').nth(1).unwrap().parse()?;
            } else if line.starts_with("#EXT-X-MEDIA-SEQUENCE") {
                playlist.media_sequence = Some(line.split(':').nth(1).unwrap().parse()?);
            } else if line.starts_with("#EXT-X-PLAYLIST-TYPE") {
                playlist.playlist_type = Some(line.split(':').nth(1).unwrap().to_string());
            } else if line.starts_with("#EXT-X-MAP") {
                let attributes = string_utils::parse_attributes(line)?; 
                if let Some(uri_str) = attributes.get("URI") {
                    let full_url = base_url.join(uri_str)?.to_string();
                    playlist.init_map_file = Some(MediaSegment {
                        url: full_url,
                        sequence: 0,
                        duration: 0.0,
                        is_discontinuity: false,
                        downloaded: false,
                        size_bytes: 0,
                    });
                }
            } else if line.starts_with("#EXTINF") {
                let duration = line.split(':').nth(1).unwrap().trim_end_matches(',').parse()?;
                current_segment = Some(MediaSegment {
                    url: String::new(),
                    sequence: index,
                    duration,
                    is_discontinuity: false,
                    downloaded: false,
                    size_bytes: 0,
                });
                index = index + 1;
            } else if line.starts_with("#EXT-X-DISCONTINUITY") {
                if let Some(segment) = current_segment.as_mut() {
                    segment.is_discontinuity = true;
                }
            } else if line.starts_with("#EXT-X-ENDLIST") {
                playlist.is_end_list = true;
            } else if !line.starts_with("#") {
                if let Some(mut segment) = current_segment.take() {
                    segment.url = base_url.join(line)?.to_string();
                    playlist.segments.push(segment);
                }
            }
        }

        playlist.total_segments = playlist.segments.len() as u32;
        if let Some(first) = playlist.segments.first() {
            playlist.first_segment_url = first.url.clone();
        }

        Ok(playlist)
    }
}
