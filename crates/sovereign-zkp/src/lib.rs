use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Number of seconds in 18 solar years (including leap days: 18 * 365.2425 * 86400)
pub const EIGHTEEN_YEARS_IN_SECONDS: u64 = 568_024_000;

#[derive(Error, Debug)]
pub enum ZkpError {
    #[error("Invalid cryptographic proof")]
    InvalidProof,

    #[error("Nullifier collision detected (Sybil attack prevention)")]
    NullifierReused,
}

pub type Result<T> = std::result::Result<T, ZkpError>;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AgeAttestation {
    pub is_adult: bool,
    pub proof_signature: Vec<u8>,
    pub nullifier: [u8; 32],
    pub verification_epoch: u64,
}

/// Simulated client-side zero-knowledge prover (Groth16 / Circom equivalence)
pub struct ZkAgeProver;

impl ZkAgeProver {
    /// Generates a Zero-Knowledge Proof that age >= 18 without revealing DOB or Aadhaar number
    pub fn prove_age(
        dob_timestamp_sec: u64,
        aadhaar_seed: &[u8],
        current_timestamp_sec: u64,
    ) -> AgeAttestation {
        let age_in_seconds = current_timestamp_sec.saturating_sub(dob_timestamp_sec);
        let is_adult = age_in_seconds >= EIGHTEEN_YEARS_IN_SECONDS;

        // Nullifier calculation: H(aadhaar_seed || "DPDP_NULLIFIER_v1")
        // Guarantees consistent anonymous identifier without leaking Aadhaar number
        let mut nullifier_input = Vec::new();
        nullifier_input.extend_from_slice(aadhaar_seed);
        nullifier_input.extend_from_slice(b"DPDP_NULLIFIER_v1");
        let nullifier = *blake3::hash(&nullifier_input).as_bytes();

        // Synthetic cryptographic circuit proof proof_bytes = H(is_adult || nullifier || current_timestamp)
        let mut proof_input = Vec::new();
        proof_input.push(if is_adult { 1 } else { 0 });
        proof_input.extend_from_slice(&nullifier);
        proof_input.extend_from_slice(&current_timestamp_sec.to_be_bytes());
        let proof_hash1 = blake3::hash(&proof_input);
        let proof_hash2 = blake3::hash(proof_hash1.as_bytes());

        let mut proof_signature = Vec::with_capacity(64);
        proof_signature.extend_from_slice(proof_hash1.as_bytes());
        proof_signature.extend_from_slice(proof_hash2.as_bytes());

        AgeAttestation {
            is_adult,
            proof_signature,
            nullifier,
            verification_epoch: current_timestamp_sec,
        }
    }
}

/// Verifies age attestation on the platform gateway
pub struct ZkAgeVerifier;

impl ZkAgeVerifier {
    pub fn verify(attestation: &AgeAttestation) -> Result<bool> {
        let mut proof_input = Vec::new();
        proof_input.push(if attestation.is_adult { 1 } else { 0 });
        proof_input.extend_from_slice(&attestation.nullifier);
        proof_input.extend_from_slice(&attestation.verification_epoch.to_be_bytes());

        let expected_hash1 = blake3::hash(&proof_input);
        let expected_hash2 = blake3::hash(expected_hash1.as_bytes());

        let mut expected_sig = Vec::with_capacity(64);
        expected_sig.extend_from_slice(expected_hash1.as_bytes());
        expected_sig.extend_from_slice(expected_hash2.as_bytes());

        if attestation.proof_signature != expected_sig {
            return Err(ZkpError::InvalidProof);
        }

        Ok(attestation.is_adult)
    }
}

/// Enforces DPDP Section 9 restrictions for minors
pub struct MinorProtectionPolicy;

impl MinorProtectionPolicy {
    pub fn can_target_ads(is_adult: bool) -> bool {
        // Section 9(3): Data fiduciary shall not undertake targeted advertising directed at children
        is_adult
    }

    pub fn can_track_behavior(is_adult: bool) -> bool {
        // Section 9(2): Data fiduciary shall not undertake tracking or behavioral monitoring of children
        is_adult
    }

    pub fn requires_parental_consent(is_adult: bool) -> bool {
        // Section 9(1): Verifiable parental consent required before processing children's personal data
        !is_adult
    }
}
