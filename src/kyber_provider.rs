//! Kyber768 post-quantum key encapsulation for notification encryption.

use pqcrypto_kyber::kyber768;
use pqcrypto_traits::kem::{Ciphertext, PublicKey, SharedSecret};

/// Kyber768 provider for encapsulating shared secrets with device public keys.
#[derive(Clone)]
pub struct KyberProvider;

impl KyberProvider {
    pub fn new() -> Self {
        Self
    }

    /// Encapsulate a shared secret using the device's post-quantum public key.
    /// Returns (kyber_ciphertext, shared_secret) - 1088 bytes for ciphertext, 32 for shared secret.
    pub fn encapsulate(&self, public_key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), KyberError> {
        let pk = kyber768::PublicKey::from_bytes(public_key).map_err(|_| KyberError::InvalidPublicKey)?;
        let (ss, ct) = kyber768::encapsulate(&pk);
        Ok((ct.as_bytes().to_vec(), ss.as_bytes().to_vec()))
    }
}

impl Default for KyberProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum KyberError {
    InvalidPublicKey,
}

impl std::fmt::Display for KyberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KyberError::InvalidPublicKey => write!(f, "Invalid Kyber public key"),
        }
    }
}

impl std::error::Error for KyberError {}
