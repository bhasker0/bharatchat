use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};
use crate::did::{IdentityKeyPair, DidDocument};
use crate::error::{SovereignError, Result};

/// Canonical Content Identifier computed via BLAKE3
pub fn compute_canonical_cid(content: &[u8]) -> [u8; 32] {
    *blake3::hash(content).as_bytes()
}

/// Metadata identifying the first creator of the content
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct OriginatorMetadata {
    pub originator_did: String,
    pub content_cid: [u8; 32],
    pub timestamp: u64,
    pub nonce: [u8; 32],
    pub signature: Vec<u8>,
}

/// Sealed traceability envelope (Rule 4(2) IT Rules 2021)
/// Only decodable by multi-sig threshold legal custodians upon valid warrant
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TraceabilityEnvelope {
    pub content_cid: [u8; 32],
    pub ephemeral_pubkey: [u8; 32],
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
}

/// Attestation attached by subsequent forwarders
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ForwardHopProof {
    pub forwarder_did: String,
    pub parent_cid: [u8; 32],
    pub forward_timestamp: u64,
    pub forwarder_signature: Vec<u8>,
}

/// Complete E2EE transmission packet carrying the protected content and the sealed traceability envelope
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MessagePacket {
    pub sender_did: String,
    pub recipient_did: String,
    /// Standard Double Ratchet E2EE message ciphertext
    pub e2ee_ciphertext: Vec<u8>,
    /// Sealed traceability envelope (contents unreadable to relay and recipient)
    pub trace_envelope: TraceabilityEnvelope,
    /// Forwarding audit trail
    pub forward_chain: Vec<ForwardHopProof>,
    pub packet_signature: Vec<u8>,
}

impl TraceabilityEnvelope {
    /// Creates a sealed traceability envelope for a newly composed message
    pub fn create(
        originator: &IdentityKeyPair,
        plaintext_content: &[u8],
        legal_public_key: &X25519PublicKey,
        timestamp: u64,
    ) -> Result<Self> {
        let content_cid = compute_canonical_cid(plaintext_content);

        let mut nonce = [0u8; 32];
        OsRng.fill_bytes(&mut nonce);

        // Sign (content_cid || timestamp || nonce)
        let mut sign_payload = Vec::new();
        sign_payload.extend_from_slice(&content_cid);
        sign_payload.extend_from_slice(&timestamp.to_be_bytes());
        sign_payload.extend_from_slice(&nonce);
        let signature = originator.sign(&sign_payload);

        let metadata = OriginatorMetadata {
            originator_did: originator.did.clone(),
            content_cid,
            timestamp,
            nonce,
            signature,
        };

        let metadata_bytes = serde_json::to_vec(&metadata)
            .map_err(|e| SovereignError::SerializationError(e.to_string()))?;

        // Ephemeral HPKE: ECDH with Legal Public Key + AES-256-GCM
        let ephemeral_secret = EphemeralSecret::random_from_rng(OsRng);
        let ephemeral_pubkey = X25519PublicKey::from(&ephemeral_secret);
        let shared_secret = ephemeral_secret.diffie_hellman(legal_public_key);

        let derived_aes_key = blake3::derive_key("Sovereign_Rule42_HPKE_v1", shared_secret.as_bytes());
        let cipher = Aes256Gcm::new_from_slice(&derived_aes_key)
            .map_err(|_| SovereignError::CipherError)?;

        let mut aes_nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut aes_nonce_bytes);
        let aes_nonce = Nonce::from_slice(&aes_nonce_bytes);

        let ciphertext = cipher.encrypt(aes_nonce, metadata_bytes.as_ref())
            .map_err(|_| SovereignError::CipherError)?;

        Ok(Self {
            content_cid,
            ephemeral_pubkey: *ephemeral_pubkey.as_bytes(),
            ciphertext,
            nonce: aes_nonce_bytes,
        })
    }

    /// Verifies that forwarded content matches the sealed envelope's CID without decrypting
    pub fn verify_content_integrity(&self, forwarded_content: &[u8]) -> bool {
        let cid = compute_canonical_cid(forwarded_content);
        cid == self.content_cid
    }

    /// Decrypts the originator metadata using the recovered legal secret key upon valid judicial warrant
    pub fn threshold_decrypt(
        &self,
        legal_secret_key: &StaticSecret,
        originator_doc: &DidDocument,
    ) -> Result<OriginatorMetadata> {
        let ephem_pub = X25519PublicKey::from(self.ephemeral_pubkey);
        let shared_secret = legal_secret_key.diffie_hellman(&ephem_pub);

        let derived_aes_key = blake3::derive_key("Sovereign_Rule42_HPKE_v1", shared_secret.as_bytes());
        let cipher = Aes256Gcm::new_from_slice(&derived_aes_key)
            .map_err(|_| SovereignError::CipherError)?;

        let aes_nonce = Nonce::from_slice(&self.nonce);
        let decrypted_bytes = cipher.decrypt(aes_nonce, self.ciphertext.as_ref())
            .map_err(|_| SovereignError::CipherError)?;

        let metadata: OriginatorMetadata = serde_json::from_slice(&decrypted_bytes)
            .map_err(|e| SovereignError::SerializationError(e.to_string()))?;

        // Verify originator's signature
        let mut sign_payload = Vec::new();
        sign_payload.extend_from_slice(&metadata.content_cid);
        sign_payload.extend_from_slice(&metadata.timestamp.to_be_bytes());
        sign_payload.extend_from_slice(&metadata.nonce);
        originator_doc.verify(&sign_payload, &metadata.signature)?;

        Ok(metadata)
    }
}

/// Forwarding helper that creates a ForwardHopProof and attaches it to the existing packet
pub fn forward_message(
    forwarder: &IdentityKeyPair,
    existing_envelope: &TraceabilityEnvelope,
    message_content: &[u8],
    forward_timestamp: u64,
) -> Result<ForwardHopProof> {
    if !existing_envelope.verify_content_integrity(message_content) {
        return Err(SovereignError::ContentHashMismatch);
    }

    let mut sign_payload = Vec::new();
    sign_payload.extend_from_slice(&existing_envelope.content_cid);
    sign_payload.extend_from_slice(&forward_timestamp.to_be_bytes());
    let forwarder_signature = forwarder.sign(&sign_payload);

    Ok(ForwardHopProof {
        forwarder_did: forwarder.did.clone(),
        parent_cid: existing_envelope.content_cid,
        forward_timestamp,
        forwarder_signature,
    })
}
