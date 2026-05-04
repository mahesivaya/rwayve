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

    // decode ciphertext
    let ciphertext = general_purpose::STANDARD
        .decode(cipher_b64)
        .map_err(|e| format!("Cipher decode error: {:?}", e))?;

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
    key.as_bytes()
        .try_into()
        .unwrap_or_else(|_| panic!("AES_KEY must be 32 bytes"))
}
