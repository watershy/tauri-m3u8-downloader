use std::collections::HashMap;
use wreq::header::HeaderMap;
use std::io::Cursor;
use zstd::stream::decode_all;

use crate::utils::network_utils;

pub fn build_impersonating_client() -> wreq::Client {
    wreq::Client::builder()
        .emulation(wreq_util::Emulation::Chrome137)
        .build()
        .expect("CRITICAL: Failed to build wreq client. TLS backend is unavailable.")
}

/// Injects standard Chrome 137 headers and user-provided overrides into a RequestBuilder.
pub fn apply_browser_headers(
    mut request: wreq::RequestBuilder, 
    headers: &HashMap<String, String>
) -> wreq::RequestBuilder {
    request = request.header(wreq::header::USER_AGENT, crate::constants::USER_AGENT_VALUE);
    request = request.header(wreq::header::ACCEPT, crate::constants::ACCEPT_VALUE);
    request = request.header(wreq::header::ACCEPT_LANGUAGE, crate::constants::ACCEPT_LANGUAGE_VALUE);
    request = request.header(wreq::header::ACCEPT_ENCODING, crate::constants::ACCEPT_ENCODING_VALUE);

    for (k, v) in headers {
        request = request.header(k, v);
    }

    request
}

pub async fn fetch_http_text(url: &str, headers: &HashMap<String, String>) -> Result<String, String> {
    let response = send_request(url, headers).await?;
    if response.status().is_success() {
        extract_text_body(response).await
    } else {
        Err(format!("Failed to fetch URL due to non-success status code: {}.", response.status()))
    }
}

pub async fn fetch_http_bytes(url: &str, headers: &HashMap<String, String>) -> Result<Vec<u8>, String> {
    let response = send_request(url, headers).await?;
    if response.status().is_success() {
        let bytes = response.bytes().await.map_err(|e| e.to_string())?;
        Ok(bytes.to_vec())
    } else {
        Err(format!("Failed to fetch bytes due to non-success status code: {}.", response.status()))
    }
}

async fn send_request(url: &str, headers: &HashMap<String, String>) -> Result<wreq::Response, String> {
    let client = build_impersonating_client();
    let request = client.get(url);
    let request = apply_browser_headers(request, headers);
    request.send().await.map_err(|e| e.to_string())
}

fn print_headers(headers: &HeaderMap) {
    for (key, value) in headers.iter() {
        match value.to_str() {
            Ok(val) => println!("{}: {}", key, val),
            Err(_) => println!("{}: [binary data]", key),
        }
    }
}

async fn extract_text_body(response: wreq::Response) -> Result<String, String> {
    let content_encoding = response.headers()
        .get("content-encoding")
        .and_then(|val| val.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    if content_encoding.contains("zstd") {
        // Manually decompress Zstd
        let bytes = response.bytes().await.map_err(|e| e.to_string())?;
        let decompressed_bytes = decode_all(Cursor::new(&bytes)).map_err(|e| e.to_string())?;
        String::from_utf8(decompressed_bytes).map_err(|e| e.to_string())
    } else {
        response.text().await.map_err(|e| e.to_string())
    }
}

pub async fn validate_http_file_access(
    url: &str, 
    headers: &HashMap<String, String>
) -> Result<(), String> {
    let client = build_impersonating_client();

    let request = client.get(url);
    let mut request = network_utils::apply_browser_headers(request, headers);
    request = request.header("Range", "bytes=0-0");

    let response = request.send().await.map_err(|e| e.to_string())?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Access check failed: HTTP {}", response.status()))
    }
}