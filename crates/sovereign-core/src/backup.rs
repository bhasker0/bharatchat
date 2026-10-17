use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use crate::error::{SovereignError, Result};
use crate::traceability::compute_canonical_cid;

/// 64-digit (256-bit) E2EE Encrypted Chat Backup Archive (WhatsApp Feature Parity)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedBackupArchive {
    pub version: u32,
    pub timestamp: u64,
    pub salt: [u8; 16],
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
    pub data_hash: [u8; 32],
}

impl EncryptedBackupArchive {
    pub const BACKUP_KEY_HEX_LEN: usize = 64;

    /// Derives a 256-bit symmetric key from a 64-digit hex key and salt via repeated BLAKE3 KDF
    pub fn derive_backup_key(hex_key_64: &str, salt: &[u8; 16]) -> Result<[u8; 32]> {
        if hex_key_64.len() != Self::BACKUP_KEY_HEX_LEN || !hex_key_64.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(SovereignError::CryptoError(
                "Backup key must be exactly 64 hexadecimal digits (256-bit)".to_string(),
            ));
        }

        let raw_bytes = hex::decode(hex_key_64)
            .map_err(|_| SovereignError::CryptoError("Invalid hex encoding for backup key".to_string()))?;

        let mut key_arr = [0u8; 32];
        key_arr.copy_from_slice(&raw_bytes);

        // BLAKE3 KDF stretching
        let mut key_material: [u8; 32];
        let mut hasher = blake3::Hasher::new_keyed(&key_arr);
        hasher.update(salt);
        key_material = *hasher.finalize().as_bytes();

        for i in 0..1_000u32 {
            let mut round_hasher = blake3::Hasher::new();
            round_hasher.update(&key_material);
            round_hasher.update(&i.to_be_bytes());
            key_material = *round_hasher.finalize().as_bytes();
        }

        Ok(key_material)
    }

    /// Encrypts local SQLite / message database into a zero-knowledge backup archive
    pub fn create_backup(plaintext_db: &[u8], hex_key_64: &str, timestamp: u64) -> Result<Self> {
        let mut salt = [0u8; 16];
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce_bytes);

        let key_bytes = Self::derive_backup_key(hex_key_64, &salt)?;
        let cipher_key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(cipher_key);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext_db)
            .map_err(|_| SovereignError::CipherError)?;

        let data_hash = compute_canonical_cid(plaintext_db);

        Ok(Self {
            version: 1,
            timestamp,
            salt,
            nonce: nonce_bytes,
            ciphertext,
            data_hash,
        })
    }

    /// Restores and verifies plaintext database using the 64-digit hex key
    pub fn restore_backup(&self, hex_key_64: &str) -> Result<Vec<u8>> {
        let key_bytes = Self::derive_backup_key(hex_key_64, &self.salt)?;
        let cipher_key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(cipher_key);
        let nonce = Nonce::from_slice(&self.nonce);

        let plaintext = cipher
            .decrypt(nonce, self.ciphertext.as_slice())
            .map_err(|_| SovereignError::CipherError)?;

        // Verify data hash integrity
        let computed_hash = compute_canonical_cid(&plaintext);
        if computed_hash != self.data_hash {
            return Err(SovereignError::ContentHashMismatch);
        }

        Ok(plaintext)
    }
}
