use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

pub fn get_iv(iv_str: &Option<String>, sequence: u32) -> Result<[u8; 16], String> {
    let mut iv = [0u8; 16];
    
    if let Some(s) = iv_str {
        // Strip the "0x" prefix if it exists
        let clean_hex = s.trim_start_matches("0x").trim_start_matches("0X");
        let decoded = hex::decode(clean_hex).map_err(|e| format!("Invalid IV hex: {}", e))?;
        
        if decoded.len() <= 16 {
            // Pad with leading zeros if the hex was too short
            let start_idx = 16 - decoded.len();
            iv[start_idx..].copy_from_slice(&decoded);
            Ok(iv)
        } else {
            Err("IV is too long (must be 16 bytes)".to_string())
        }
    } else {
        // HLS Spec: If no IV is provided, use the sequence number as a big-endian 32-bit int
        let seq_bytes = sequence.to_be_bytes();
        iv[12..16].copy_from_slice(&seq_bytes);
        Ok(iv)
    }
}

pub fn decrypt_aes128<'a>(data: &'a mut [u8], key: &[u8], iv: &[u8; 16]) -> Result<&'a [u8], String> {
    let pt = Aes128CbcDec::new(key.into(), iv.into())
        .decrypt_padded_mut::<Pkcs7>(data)
        .map_err(|e| format!("AES Decryption error: {:?}", e))?;
    Ok(pt)
}