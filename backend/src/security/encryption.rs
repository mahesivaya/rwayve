use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose;
use rand::{RngCore, thread_rng};

pub fn encrypt(text: &str) -> Result<(String, String)> {
    let key_bytes = get_key();
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; 12];
    thread_rng().fill_bytes(&mut nonce_bytes);

    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, text.as_bytes())
        .map_err(|e| anyhow::anyhow!("encryption failed: {:?}", e))?;

    Ok((
        general_purpose::STANDARD.encode(nonce_bytes),
        general_purpose::STANDARD.encode(ciphertext),
    ))
}

pub fn decrypt(nonce_b64: &str, cipher_b64: &str) -> Result<String, String> {
    let key_bytes = get_key();
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    // decode nonce
    let nonce = general_purpose::STANDARD
        .decode(nonce_b64)
        .map_err(|e| format!("Nonce decode error: {:?}", e))?;

    // AES-GCM nonce is fixed 12 bytes; passing anything else makes
    // `Nonce::from_slice` panic in generic-array. Reject explicitly so
    // callers get a clean error instead of a panic.
    if nonce.len() != 12 {
        return Err(format!(
            "Invalid nonce length: expected 12, got {}",
            nonce.len()
        ));
    }

    // decode ciphertext
    let ciphertext = general_purpose::STANDARD
        .decode(cipher_b64)
        .map_err(|e| format!("Cipher decode error: {:?}", e))?;

    if ciphertext.is_empty() {
        return Err("Empty ciphertext".to_string());
    }

    // decrypt
    let decrypted = cipher
        .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|e| format!("Decrypt error: {:?}", e))?;

    // utf8 conversion
    let text = String::from_utf8(decrypted).map_err(|e| format!("UTF8 error: {:?}", e))?;

    Ok(text)
}

fn get_key() -> [u8; 32] {
    let key = std::env::var("AES_KEY").unwrap_or_else(|_| panic!("AES_KEY not set"));
    let trimmed = key.trim();

    if trimmed.len() == 64 && trimmed.bytes().all(|b| b.is_ascii_hexdigit()) {
        return decode_hex64(trimmed).unwrap_or_else(|e| panic!("{e}"));
    }

    trimmed
        .as_bytes()
        .try_into()
        .unwrap_or_else(|_| panic!("AES_KEY must be Hex64 (64 hex chars for 32 bytes)"))
}

fn decode_hex64(hex: &str) -> Result<[u8; 32], String> {
    if hex.len() != 64 {
        return Err("AES_KEY Hex64 must be exactly 64 hex characters".to_string());
    }

    let mut bytes = [0u8; 32];

    for (idx, chunk) in hex.as_bytes().chunks_exact(2).enumerate() {
        let hi = hex_value(chunk[0])?;
        let lo = hex_value(chunk[1])?;
        bytes[idx] = (hi << 4) | lo;
    }

    Ok(bytes)
}

fn hex_value(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err("AES_KEY Hex64 contains a non-hex character".to_string()),
    }
}
