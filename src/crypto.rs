// crypto — CV_SECRET → AES-256-GCM for assets at rest in the image
//
// Format: [12-byte nonce][ciphertext + 16-byte GCM tag]
// Key: PBKDF2-HMAC-SHA256, 100_000 rounds, fixed salt (see encrypt binary).

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{anyhow, Result};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

const PBKDF2_SALT: &[u8] = b"cv_pipeline_salt_v1";
const PBKDF2_ROUNDS: u32 = 100_000;

pub fn derive_key(secret: &str) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(secret.as_bytes(), PBKDF2_SALT, PBKDF2_ROUNDS, &mut key);
    key
}

pub fn encrypt(plaintext: &[u8], secret: &str) -> Result<Vec<u8>> {
    let key_bytes = derive_key(secret);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| anyhow!("encryption failed: {e}"))?;
    let mut output = Vec::with_capacity(12 + ciphertext.len());
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

pub fn decrypt(encrypted: &[u8], secret: &str) -> Result<Vec<u8>> {
    if encrypted.len() < 28 {
        return Err(anyhow!("ciphertext too short — re-run encrypt binary"));
    }
    let (nonce_bytes, ciphertext) = encrypted.split_at(12);
    let key_bytes = derive_key(secret);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow!("decryption failed — check CV_SECRET is correct"))
}
