//! Sovereign Core Cryptographic & Legal Compliance Library
//!
//! Provides the foundational implementations for:
//! 1. W3C Decentralized Identity (`did:bharat:<pubkey>`)
//! 2. Rule 4(2) IT Rules 2021 First-Originator Traceability without breaking E2EE
//! 3. DPDP Act 2023 Section 12 Crypto-Shredding & Erasure
//! 4. Client-side CSAM Prevention via blind perceptual hashing
//! 5. Anti-Spam Micro-Staking & Automated Slashing

pub mod error;
pub mod did;
pub mod traceability;
pub mod erasure;
pub mod csam_scanner;
pub mod staking;
pub mod ratchet;

pub use error::{SovereignError, Result};
pub use did::{IdentityKeyPair, DidDocument};
pub use traceability::{
    compute_canonical_cid,
    OriginatorMetadata,
    TraceabilityEnvelope,
    ForwardHopProof,
    MessagePacket,
    forward_message,
};
pub use erasure::StorageMasterKey;
pub use csam_scanner::{compute_perceptual_hash, BlindBloomFilter, ClientScanner, ComplianceAlert};
pub use staking::{
    BusinessAccount,
    ActiveBroadcast,
    BroadcastResolution,
    MICROSTAKE_PER_RECIPIENT_PAISE,
    MAXIMUM_SPAM_TOLERANCE_RATIO,
};
pub use ratchet::{
    DoubleRatchetSession,
    RatchetHeader,
    RatchetEncryptedMessage,
};
