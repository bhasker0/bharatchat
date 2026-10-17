use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use zeroize::ZeroizeOnDrop;
use crate::error::{SovereignError, Result};

#[derive(Clone, Serialize, Deserialize)]
pub struct DidDocument {
    pub did: String,
    pub verifying_key_bytes: [u8; 32],
}

#[derive(ZeroizeOnDrop)]
pub struct IdentityKeyPair {
    #[zeroize(skip)]
    pub did: String,
    signing_key: SigningKey,
    #[zeroize(skip)]
    pub verifying_key: VerifyingKey,
}

impl IdentityKeyPair {
    /// Generates a new hardware-rooted identity key pair conforming to did:bharat
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let did = format!("did:bharat:{}", hex::encode(verifying_key.as_bytes()));

        Self {
            did,
            signing_key,
            verifying_key,
        }
    }

    /// Signs an arbitrary message using the identity key
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing_key.sign(message).to_bytes().to_vec()
    }

    /// Returns the public DID document
    pub fn document(&self) -> DidDocument {
        DidDocument {
            did: self.did.clone(),
            verifying_key_bytes: *self.verifying_key.as_bytes(),
        }
    }

    /// Converts identity key bytes to an X25519 public key representation for DH
    pub fn to_x25519_public(&self) -> [u8; 32] {
        let vk_bytes = self.verifying_key.as_bytes();
        let hash = blake3::hash(vk_bytes);
        *hash.as_bytes()
    }
}

impl DidDocument {
    /// Verifies an Ed25519 signature against this DID
    pub fn verify(&self, message: &[u8], signature_bytes: &[u8]) -> Result<()> {
        let verifying_key = VerifyingKey::from_bytes(&self.verifying_key_bytes)
            .map_err(|_| SovereignError::SignatureVerificationFailed)?;
        let signature = Signature::from_slice(signature_bytes)
            .map_err(|_| SovereignError::SignatureVerificationFailed)?;
        verifying_key.verify(message, &signature)
            .map_err(|_| SovereignError::SignatureVerificationFailed)
    }
}
