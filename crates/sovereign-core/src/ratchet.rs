//! Double Ratchet Protocol Implementation (Signal Protocol Equivalence)
//!
//! Provides pairwise End-to-End Encryption with:
//! 1. Forward Secrecy (FS): Compromised message keys cannot decrypt past messages.
//! 2. Post-Compromise Security (PCS / Break-in Recovery): Future DH ratchets recover secrecy.
//! 3. Out-of-order message handling with bounded key storage.
//! 4. Zeroization of ephemeral secrets and message keys.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};
use zeroize::Zeroize;

use crate::error::{Result, SovereignError};

const MAX_SKIP: u32 = 1000;

/// Message Header carrying ratchet state
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct RatchetHeader {
    pub dh_pubkey: [u8; 32],
    pub pn: u32,
    pub n: u32,
}

/// Fully encrypted message payload
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RatchetEncryptedMessage {
    pub header: RatchetHeader,
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
}

/// Double Ratchet State Machine
pub struct DoubleRatchetSession {
    /// Our current ephemeral DH keypair
    our_dh_secret: StaticSecret,
    pub our_dh_pubkey: X25519PublicKey,

    /// Peer's current ephemeral DH public key
    pub peer_dh_pubkey: Option<X25519PublicKey>,

    /// Root KDF key
    root_key: [u8; 32],

    /// Sending symmetric chain key
    chain_key_send: Option<[u8; 32]>,

    /// Receiving symmetric chain key
    chain_key_recv: Option<[u8; 32]>,

    /// Message number in sending chain
    ns: u32,

    /// Message number in receiving chain
    nr: u32,

    /// Previous sending chain length
    pn: u32,

    /// Skipped message keys cache: (peer_dh_pubkey, message_num) -> message_key
    skipped_message_keys: HashMap<([u8; 32], u32), [u8; 32]>,
}

impl DoubleRatchetSession {
    /// Initialize session as Alice (the initiator who already has Bob's initial public key)
    pub fn init_alice(shared_master_secret: [u8; 32], bob_dh_pubkey: [u8; 32]) -> Self {
        let our_dh_secret = StaticSecret::random_from_rng(OsRng);
        let our_dh_pubkey = X25519PublicKey::from(&our_dh_secret);
        let peer_dh_pubkey = X25519PublicKey::from(bob_dh_pubkey);

        let dh_shared = our_dh_secret.diffie_hellman(&peer_dh_pubkey);
        let (root_key, chain_key_send) = kdf_rk(&shared_master_secret, dh_shared.as_bytes());

        Self {
            our_dh_secret,
            our_dh_pubkey,
            peer_dh_pubkey: Some(peer_dh_pubkey),
            root_key,
            chain_key_send: Some(chain_key_send),
            chain_key_recv: None,
            ns: 0,
            nr: 0,
            pn: 0,
            skipped_message_keys: HashMap::new(),
        }
    }

    /// Initialize session as Bob (the receiver who awaits Alice's first message)
    pub fn init_bob(shared_master_secret: [u8; 32], our_keypair: (StaticSecret, X25519PublicKey)) -> Self {
        Self {
            our_dh_secret: our_keypair.0,
            our_dh_pubkey: our_keypair.1,
            peer_dh_pubkey: None,
            root_key: shared_master_secret,
            chain_key_send: None,
            chain_key_recv: None,
            ns: 0,
            nr: 0,
            pn: 0,
            skipped_message_keys: HashMap::new(),
        }
    }

    /// Encrypt a plaintext message with Forward Secrecy
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<RatchetEncryptedMessage> {
        let ck_send = self.chain_key_send.ok_or_else(|| {
            SovereignError::CryptoError("Sending chain key not initialized".to_string())
        })?;

        let (next_ck, mut msg_key) = kdf_ck(&ck_send);
        self.chain_key_send = Some(next_ck);

        let header = RatchetHeader {
            dh_pubkey: *self.our_dh_pubkey.as_bytes(),
            pn: self.pn,
            n: self.ns,
        };
        self.ns += 1;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(&msg_key)
            .map_err(|e| SovereignError::CryptoError(e.to_string()))?;
        
        let header_bytes = serde_json::to_vec(&header)
            .map_err(|e| SovereignError::SerializationError(e.to_string()))?;

        let payload = aes_gcm::aead::Payload {
            msg: plaintext,
            aad: &header_bytes,
        };

        let ciphertext = cipher
            .encrypt(nonce, payload)
            .map_err(|e| SovereignError::CryptoError(format!("AES-GCM encryption failed: {}", e)))?;

        // Zeroize message key in memory immediately
        msg_key.zeroize();

        Ok(RatchetEncryptedMessage {
            header,
            ciphertext,
            nonce: nonce_bytes,
        })
    }

    /// Decrypt a message with ratchet advancement and out-of-order key recovery
    pub fn decrypt(&mut self, msg: &RatchetEncryptedMessage) -> Result<Vec<u8>> {
        let header_bytes = serde_json::to_vec(&msg.header)
            .map_err(|e| SovereignError::SerializationError(e.to_string()))?;

        // 1. Check if key is already in skipped keys
        let key_idx = (msg.header.dh_pubkey, msg.header.n);
        if let Some(mut mk) = self.skipped_message_keys.remove(&key_idx) {
            let res = decrypt_payload(&mk, &msg.nonce, &msg.ciphertext, &header_bytes);
            mk.zeroize();
            return res;
        }

        // 2. If new DH key received, perform DH Ratchet step
        let msg_peer_dh = X25519PublicKey::from(msg.header.dh_pubkey);
        if Some(msg_peer_dh) != self.peer_dh_pubkey {
            self.skip_message_keys(msg.header.pn)?;
            self.dh_ratchet(msg_peer_dh);
        }

        // 3. Skip message keys in current receiving chain if out of order
        self.skip_message_keys(msg.header.n)?;

        // 4. Derive message key from receiving chain
        let ck_recv = self.chain_key_recv.ok_or_else(|| {
            SovereignError::CryptoError("Receiving chain key not initialized".to_string())
        })?;
        let (next_ck, mut msg_key) = kdf_ck(&ck_recv);
        self.chain_key_recv = Some(next_ck);
        self.nr += 1;

        let res = decrypt_payload(&msg_key, &msg.nonce, &msg.ciphertext, &header_bytes);
        msg_key.zeroize();
        res
    }

    fn dh_ratchet(&mut self, new_peer_dh: X25519PublicKey) {
        self.pn = self.ns;
        self.ns = 0;
        self.nr = 0;
        self.peer_dh_pubkey = Some(new_peer_dh);

        // DH Ratchet Receive
        let dh_recv = self.our_dh_secret.diffie_hellman(&new_peer_dh);
        let (rk1, ck_recv) = kdf_rk(&self.root_key, dh_recv.as_bytes());
        self.root_key = rk1;
        self.chain_key_recv = Some(ck_recv);

        // Generate fresh DH pair for sending (Post-Compromise Security)
        self.our_dh_secret = StaticSecret::random_from_rng(OsRng);
        self.our_dh_pubkey = X25519PublicKey::from(&self.our_dh_secret);

        // DH Ratchet Send
        let dh_send = self.our_dh_secret.diffie_hellman(&new_peer_dh);
        let (rk2, ck_send) = kdf_rk(&self.root_key, dh_send.as_bytes());
        self.root_key = rk2;
        self.chain_key_send = Some(ck_send);
    }

    fn skip_message_keys(&mut self, until: u32) -> Result<()> {
        if self.nr + MAX_SKIP < until {
            return Err(SovereignError::CryptoError("Too many skipped messages".to_string()));
        }

        if let Some(mut ck) = self.chain_key_recv {
            while self.nr < until {
                let (next_ck, msg_key) = kdf_ck(&ck);
                ck = next_ck;
                if let Some(peer_key) = self.peer_dh_pubkey {
                    self.skipped_message_keys.insert((*peer_key.as_bytes(), self.nr), msg_key);
                }
                self.nr += 1;
            }
            self.chain_key_recv = Some(ck);
        }
        Ok(())
    }
}

impl Drop for DoubleRatchetSession {
    fn drop(&mut self) {
        self.root_key.zeroize();
        if let Some(ref mut ck) = self.chain_key_send {
            ck.zeroize();
        }
        if let Some(ref mut ck) = self.chain_key_recv {
            ck.zeroize();
        }
        for (_, mut key) in self.skipped_message_keys.drain() {
            key.zeroize();
        }
    }
}

/// Helper KDF for Root Key evolution
fn kdf_rk(root_key: &[u8; 32], dh_out: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let mut hasher = blake3::Hasher::new_keyed(root_key);
    hasher.update(b"Sovereign Double Ratchet RK-KDF v1");
    hasher.update(dh_out);
    let mut out = [0u8; 64];
    let mut reader = hasher.finalize_xof();
    reader.fill(&mut out);

    let mut next_rk = [0u8; 32];
    let mut next_ck = [0u8; 32];
    next_rk.copy_from_slice(&out[0..32]);
    next_ck.copy_from_slice(&out[32..64]);
    (next_rk, next_ck)
}

/// Helper KDF for Chain Key evolution and Message Key extraction
fn kdf_ck(chain_key: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let mut hasher = blake3::Hasher::new_keyed(chain_key);
    hasher.update(b"Sovereign Double Ratchet CK-KDF v1");
    let mut out = [0u8; 64];
    let mut reader = hasher.finalize_xof();
    reader.fill(&mut out);

    let mut next_ck = [0u8; 32];
    let mut msg_key = [0u8; 32];
    next_ck.copy_from_slice(&out[0..32]);
    msg_key.copy_from_slice(&out[32..64]);
    (next_ck, msg_key)
}

fn decrypt_payload(
    key: &[u8; 32],
    nonce_bytes: &[u8; 12],
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| SovereignError::CryptoError(e.to_string()))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let payload = aes_gcm::aead::Payload {
        msg: ciphertext,
        aad,
    };
    cipher
        .decrypt(nonce, payload)
        .map_err(|e| SovereignError::CryptoError(format!("Decryption authentication failed: {}", e)))
}
