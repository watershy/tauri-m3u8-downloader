use std::collections::HashMap;
use std::path::{Path, PathBuf};
use url::Url;

pub fn parse_attributes(line: &str) -> Result<HashMap<String, String>, String> {
    let attributes = line.splitn(2, ':').nth(1).ok_or("Invalid attribute line")?;
    let mut map = HashMap::new();

    for part in attributes.split(',') {
        if let Some((key, value)) = part.split_once('=') {
            map.insert(key.to_string(), value.trim_matches('"').to_string());
        }
    }

    Ok(map)
}

pub fn map_codec(codecs: Option<&str>) -> String {
    match codecs {
        Some(value) if value.contains("av01") => "av1".to_string(),
        Some(value) if value.contains("avc1") => "h264".to_string(),
        Some(value) => value.to_string(),
        None => "".to_string(),
    }
}

pub fn resolve_m3u8_url(master_url: &str, relative_uri: String) -> String {
    match Url::parse(master_url)
        .and_then(|base| base.join(&relative_uri))
    {
        Ok(media_url) => media_url.to_string(),
        Err(_) => relative_uri,
    }
}

pub trait IntoString {
    fn to_string(&self) -> String;
}

impl IntoString for PathBuf {
    fn to_string(&self) -> String {
        self.to_string_lossy().into_owned()
    }
}

impl IntoString for Path {
    fn to_string(&self) -> String {
        self.to_string_lossy().into_owned()
    }
}