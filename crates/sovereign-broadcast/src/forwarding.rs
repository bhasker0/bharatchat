use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const FREQUENTLY_FORWARDED_THRESHOLD: u32 = 5;
pub const MAX_STANDARD_FORWARD_TARGETS: usize = 5;
pub const MAX_FREQUENTLY_FORWARDED_TARGETS: usize = 1;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ForwardingError {
    #[error("Frequently forwarded messages (>= 5 hops) can only be forwarded to 1 chat at a time to prevent viral misinformation")]
    FrequentlyForwardedLimitExceeded,
    #[error("Standard messages cannot be forwarded to more than 5 chats simultaneously")]
    StandardForwardLimitExceeded,
    #[error("Cannot forward to an empty target list")]
    EmptyTargetList,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForwardLabel {
    /// Not forwarded (original message)
    None,
    /// Forwarded 1 to 4 times (single arrow)
    Forwarded,
    /// Forwarded 5 or more times (double arrow) - restricted to 1 chat at a time
    ForwardedManyTimes,
}

/// Cryptographically trackable forwarding metadata (Rule 4(2) compatible & viral friction enforced)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForwardEnvelope {
    pub message_cid: String,
    pub original_sender_did: String,
    pub hop_count: u32,
    pub label: ForwardLabel,
    pub max_allowed_targets: usize,
}

impl ForwardEnvelope {
    pub fn new_direct(message_cid: &str, sender_did: &str) -> Self {
        Self {
            message_cid: message_cid.to_string(),
            original_sender_did: sender_did.to_string(),
            hop_count: 0,
            label: ForwardLabel::None,
            max_allowed_targets: MAX_STANDARD_FORWARD_TARGETS,
        }
    }

    /// Forward message to the next hop, escalating friction if >= 5 hops
    pub fn forward_to_next(&self) -> Self {
        let new_hops = self.hop_count + 1;
        let (label, max_allowed) = if new_hops >= FREQUENTLY_FORWARDED_THRESHOLD {
            (ForwardLabel::ForwardedManyTimes, MAX_FREQUENTLY_FORWARDED_TARGETS)
        } else {
            (ForwardLabel::Forwarded, MAX_STANDARD_FORWARD_TARGETS)
        };

        Self {
            message_cid: self.message_cid.clone(),
            original_sender_did: self.original_sender_did.clone(),
            hop_count: new_hops,
            label,
            max_allowed_targets: max_allowed,
        }
    }

    /// Validates forward attempt against statutory and safety caps
    pub fn validate_forward(&self, target_chat_ids: &[String]) -> Result<(), ForwardingError> {
        if target_chat_ids.is_empty() {
            return Err(ForwardingError::EmptyTargetList);
        }

        if self.label == ForwardLabel::ForwardedManyTimes && target_chat_ids.len() > MAX_FREQUENTLY_FORWARDED_TARGETS {
            return Err(ForwardingError::FrequentlyForwardedLimitExceeded);
        }

        if target_chat_ids.len() > MAX_STANDARD_FORWARD_TARGETS {
            return Err(ForwardingError::StandardForwardLimitExceeded);
        }

        Ok(())
    }
}
