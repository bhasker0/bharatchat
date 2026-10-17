use thiserror::Error;

#[derive(Error, Debug)]
pub enum SovereignError {
    #[error("Cryptographic signature verification failed")]
    SignatureVerificationFailed,

    #[error("Encryption or decryption failed")]
    CipherError,

    #[error("Threshold signature or key shares insufficient (required {required}, got {provided})")]
    InsufficientThresholdShares { required: usize, provided: usize },

    #[error("Judicial warrant invalid or expired: {0}")]
    InvalidWarrant(String),

    #[error("Content hash mismatch during forwarding verification")]
    ContentHashMismatch,

    #[error("CSAM match detected! Content transmission blocked under POCSO/IT Rules")]
    CsamDetected { hash: String },

    #[error("Data principal encryption key has been shredded (DPDP Act Sec 12)")]
    KeyShredded,

    #[error("Broadcast stake violation: {0}")]
    StakeViolation(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, SovereignError>;
