//! AES-256-GCM encryption with HKDF key derivation.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm,
};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

/// Encrypt plaintext with AES-256-GCM using a key derived from the shared secret via HKDF.
/// Returns (ciphertext_with_auth_tag, nonce). The auth tag (16 bytes) is appended to ciphertext.
pub fn encrypt(shared_secret: &[u8], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
    let hk = Hkdf::<Sha256>::new(None, shared_secret);
    let mut key = [0u8; 32];
    hk.expand(b"", &mut key).map_err(|_| CryptoError::Hkdf)?;

    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| CryptoError::Key)?;
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);

    let ciphertext = cipher
        .encrypt(&nonce.into(), plaintext)
        .map_err(|_| CryptoError::Encrypt)?;

    Ok((ciphertext, nonce.to_vec()))
}

#[derive(Debug)]
pub enum CryptoError {
    Hkdf,
    Key,
    Encrypt,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::Hkdf => write!(f, "HKDF error"),
            CryptoError::Key => write!(f, "Invalid key"),
            CryptoError::Encrypt => write!(f, "Encryption failed"),
        }
    }
}

impl std::error::Error for CryptoError {}
