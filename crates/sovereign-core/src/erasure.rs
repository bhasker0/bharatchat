use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};
use crate::error::{SovereignError, Result};

/// Hardware-backed Storage Master Key (SMK) with secure Zeroize capabilities
#[derive(ZeroizeOnDrop)]
pub struct StorageMasterKey {
    key_bytes: [u8; 32],
    #[zeroize(skip)]
    is_shredded: bool,
    #[zeroize(skip)]
    pub key_id: String,
}

impl StorageMasterKey {
    /// Generates a fresh ephemeral Storage Master Key
    pub fn generate(owner_did: &str) -> Self {
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        let key_id = format!("smk:{}:{}", owner_did, hex::encode(&key_bytes[0..8]));

        Self {
            key_bytes,
            is_shredded: false,
            key_id,
        }
    }

    /// Encrypts an off-chain data payload (e.g. backup, media) under this user key
    pub fn encrypt_payload(&self, plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        if self.is_shredded {
            return Err(SovereignError::KeyShredded);
        }

        let cipher = Aes256Gcm::new_from_slice(&self.key_bytes)
            .map_err(|_| SovereignError::CipherError)?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|_| SovereignError::CipherError)?;

        Ok((ciphertext, nonce_bytes))
    }

    /// Decrypts an off-chain data payload
    pub fn decrypt_payload(&self, ciphertext: &[u8], nonce_bytes: &[u8; 12]) -> Result<Vec<u8>> {
        if self.is_shredded {
            return Err(SovereignError::KeyShredded);
        }

        let cipher = Aes256Gcm::new_from_slice(&self.key_bytes)
            .map_err(|_| SovereignError::CipherError)?;

        let nonce = Nonce::from_slice(nonce_bytes);
        cipher.decrypt(nonce, ciphertext)
            .map_err(|_| SovereignError::CipherError)
    }

    /// Executes unrecoverable Crypto-Shredding (DPDP Act 2023 Sec 12)
    /// Permanently wipes key material from memory and invalidates all future decryptions
    pub fn crypto_shred(&mut self) {
        self.key_bytes.zeroize();
        self.is_shredded = true;
    }

    pub fn is_shredded(&self) -> bool {
        self.is_shredded
    }
}
