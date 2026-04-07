use anyhow::Result;
use aes_gcm::{
    Aes256Gcm,
    Key,
    Nonce,
    aead::{Aead, KeyInit}
};
use rand::{RngCore, thread_rng};
use base64::engine::general_purpose;
use base64::Engine;

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

pub fn decrypt(nonce_b64: &str, cipher_b64: &str) -> String {
    let key_bytes = get_key();
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let nonce = general_purpose::STANDARD.decode(nonce_b64).unwrap();
    let ciphertext = general_purpose::STANDARD.decode(cipher_b64).unwrap();

    let decrypted = cipher
        .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
        .unwrap();

    String::from_utf8(decrypted).unwrap()
}

fn get_key() -> [u8; 32] {
    std::env::var("AES_KEY")
        .expect("AES_KEY not set")
        .as_bytes()
        .try_into()
        .expect("AES_KEY must be 32 bytes")
}