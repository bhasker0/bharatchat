use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sovereign_core::IdentityKeyPair;
use std::collections::HashMap;
use thiserror::Error;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

#[derive(Error, Debug)]
pub enum MultiDeviceError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Device already revoked")]
    DeviceRevoked,

    #[error("Cryptographic pairing error: {0}")]
    CryptoError(String),

    #[error("Invalid pairing nonce or signature mismatch")]
    InvalidNonce,
}

pub type Result<T> = std::result::Result<T, MultiDeviceError>;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum CompanionDeviceType {
    DesktopMac,
    DesktopWindows,
    DesktopLinux,
    WebBrowser,
    Tablet,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CompanionDeviceRecord {
    pub device_id: String,
    pub device_name: String,
    pub device_type: CompanionDeviceType,
    pub companion_pubkey: [u8; 32],
    pub paired_at: u64,
    pub is_active: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct QrPairingChallenge {
    pub session_id: String,
    pub companion_pubkey: [u8; 32],
    pub pairing_nonce: [u8; 16],
    pub created_at: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PairingBundlePayload {
    pub primary_did: String,
    pub device_id: String,
    pub pairing_nonce: [u8; 16],
    pub authorized_at: u64,
    pub primary_signature: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EncryptedPairingBundle {
    pub primary_ephemeral_pubkey: [u8; 32],
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
}

pub struct CompanionProver {
    secret: StaticSecret,
    pub public_key: X25519PublicKey,
    pub pairing_nonce: [u8; 16],
}

impl CompanionProver {
    /// Step 1: Secondary device generates an ephemeral challenge displayed as a QR Code
    pub fn new_pairing_challenge(session_id: &str, timestamp: u64) -> (Self, QrPairingChallenge) {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public_key = X25519PublicKey::from(&secret);

        let mut pairing_nonce = [0u8; 16];
        OsRng.fill_bytes(&mut pairing_nonce);

        let challenge = QrPairingChallenge {
            session_id: session_id.to_string(),
            companion_pubkey: *public_key.as_bytes(),
            pairing_nonce,
            created_at: timestamp,
        };

        (
            Self {
                secret,
                public_key,
                pairing_nonce,
            },
            challenge,
        )
    }

    /// Step 3: Companion receives the encrypted bundle and completes onboarding
    pub fn finalize_pairing(
        &self,
        bundle: &EncryptedPairingBundle,
    ) -> Result<PairingBundlePayload> {
        let primary_pub = X25519PublicKey::from(bundle.primary_ephemeral_pubkey);
        let shared_secret = self.secret.diffie_hellman(&primary_pub);

        let derived_key = blake3::derive_key("Sovereign_Companion_Pairing_v1", shared_secret.as_bytes());
        let cipher = Aes256Gcm::new_from_slice(&derived_key)
            .map_err(|e| MultiDeviceError::CryptoError(e.to_string()))?;

        let nonce = Nonce::from_slice(&bundle.nonce);
        let decrypted = cipher
            .decrypt(nonce, bundle.ciphertext.as_ref())
            .map_err(|e| MultiDeviceError::CryptoError(e.to_string()))?;

        let payload: PairingBundlePayload = serde_json::from_slice(&decrypted)
            .map_err(|e| MultiDeviceError::CryptoError(e.to_string()))?;

        if payload.pairing_nonce != self.pairing_nonce {
            return Err(MultiDeviceError::InvalidNonce);
        }

        Ok(payload)
    }
}

pub struct MultiDeviceManager {
    pub primary_did: String,
    pub devices: HashMap<String, CompanionDeviceRecord>,
}

impl MultiDeviceManager {
    pub fn new(primary_did: &str) -> Self {
        Self {
            primary_did: primary_did.to_string(),
            devices: HashMap::new(),
        }
    }

    /// Step 2: Primary device scans QR code challenge and approves companion device
    pub fn approve_pairing(
        &mut self,
        primary_keypair: &IdentityKeyPair,
        challenge: &QrPairingChallenge,
        device_type: CompanionDeviceType,
        device_name: &str,
        timestamp: u64,
    ) -> Result<(EncryptedPairingBundle, CompanionDeviceRecord)> {
        let device_id = format!(
            "dev_{}_{}",
            hex::encode(&challenge.companion_pubkey[0..4]),
            timestamp
        );

        let companion_pub = X25519PublicKey::from(challenge.companion_pubkey);
        let primary_ephemeral_secret = StaticSecret::random_from_rng(OsRng);
        let primary_ephemeral_pub = X25519PublicKey::from(&primary_ephemeral_secret);

        let shared_secret = primary_ephemeral_secret.diffie_hellman(&companion_pub);
        let derived_key = blake3::derive_key("Sovereign_Companion_Pairing_v1", shared_secret.as_bytes());

        // Primary signs (device_id || companion_pubkey || nonce || timestamp)
        let mut sign_payload = Vec::new();
        sign_payload.extend_from_slice(device_id.as_bytes());
        sign_payload.extend_from_slice(&challenge.companion_pubkey);
        sign_payload.extend_from_slice(&challenge.pairing_nonce);
        sign_payload.extend_from_slice(&timestamp.to_be_bytes());
        let primary_signature = primary_keypair.sign(&sign_payload);

        let payload = PairingBundlePayload {
            primary_did: self.primary_did.clone(),
            device_id: device_id.clone(),
            pairing_nonce: challenge.pairing_nonce,
            authorized_at: timestamp,
            primary_signature,
        };

        let payload_bytes = serde_json::to_vec(&payload)
            .map_err(|e| MultiDeviceError::CryptoError(e.to_string()))?;

        let cipher = Aes256Gcm::new_from_slice(&derived_key)
            .map_err(|e| MultiDeviceError::CryptoError(e.to_string()))?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, payload_bytes.as_ref())
            .map_err(|e| MultiDeviceError::CryptoError(e.to_string()))?;

        let bundle = EncryptedPairingBundle {
            primary_ephemeral_pubkey: *primary_ephemeral_pub.as_bytes(),
            ciphertext,
            nonce: nonce_bytes,
        };

        let record = CompanionDeviceRecord {
            device_id: device_id.clone(),
            device_name: device_name.to_string(),
            device_type,
            companion_pubkey: challenge.companion_pubkey,
            paired_at: timestamp,
            is_active: true,
        };

        self.devices.insert(device_id, record.clone());

        Ok((bundle, record))
    }

    /// Revokes authorization for a companion device (instant multi-device unlink)
    pub fn revoke_device(&mut self, device_id: &str) -> Result<()> {
        let record = self
            .devices
            .get_mut(device_id)
            .ok_or_else(|| MultiDeviceError::DeviceNotFound(device_id.to_string()))?;

        if !record.is_active {
            return Err(MultiDeviceError::DeviceRevoked);
        }

        record.is_active = false;
        Ok(())
    }

    /// Checks if a device has active companion authorization
    pub fn is_device_authorized(&self, device_id: &str) -> bool {
        self.devices.get(device_id).map_or(false, |d| d.is_active)
    }

    /// Returns list of all active linked companion devices
    pub fn list_active_devices(&self) -> Vec<&CompanionDeviceRecord> {
        self.devices.values().filter(|d| d.is_active).collect()
    }
}
