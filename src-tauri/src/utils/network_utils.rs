use std::collections::HashMap;
use reqwest::Client;
use reqwest::header::{ HeaderValue, USER_AGENT, ACCEPT, ACCEPT_ENCODING, HeaderName, HeaderMap };
use std::io::Cursor;
use zstd::stream::decode_all;

use crate::constants;

pub async fn fetch_http_content(url: &str, headers: &HashMap<String, String>) -> Result<String, String> {
    let client = Client::new();
    let mut req_headers: reqwest::header::HeaderMap = reqwest::header::HeaderMap::new();
    req_headers.insert(USER_AGENT, HeaderValue::from_static(constants::USER_AGENT_VALUE));
    req_headers.insert(ACCEPT, HeaderValue::from_static(constants::ACCEPT_VALUE));
    req_headers.insert(ACCEPT_ENCODING, HeaderValue::from_static(constants::ACCEPT_ENCODING_VALUE));
    for (header_name, header_value) in headers {
        let header_name_bytes = header_name.as_bytes();
        if let Ok(h_name) = HeaderName::from_bytes(header_name_bytes) {
            if let Ok(h_value) = HeaderValue::from_str(header_value) {
                req_headers.insert(h_name, h_value);
            }
        }
    }

    let response = client.get(url)
        .headers(req_headers)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    print_headers(response.headers());
    if response.status().is_success() {
        extract_text_body(response).await
    } else {
        let error_message: String = format!("Failed to fetch URL due to non-success status code: {}.", response.status());
        Err(error_message)
    }
}

fn print_headers(headers: &HeaderMap) {
    for (key, value) in headers.iter() {
        match value.to_str() {
            Ok(val) => println!("{}: {}", key, val),
            Err(_) => println!("{}: [binary data]", key),
        }
    }
}

async fn extract_text_body(response: reqwest::Response) -> Result<String, String> {
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
        // `reqwest.text()` will handle plain text, gzip, and brotli
        response.text().await.map_err(|e| e.to_string())
    }
}

pub async fn validate_http_file_access(
    url: &str, 
    headers: &HashMap<String, String>
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let mut request = client.get(url);

    for (k, v) in headers {
        request = request.header(k, v);
    }

    request = request.header("Range", "bytes=0-0");
    let response = request.send().await.map_err(|e| e.to_string())?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Access check failed: HTTP {}", response.status()))
    }
}