use std::collections::HashMap;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MAXIMUM_CALL_PARTICIPANTS: usize = 32;

#[derive(Error, Debug)]
pub enum CallError {
    #[error("Call room is at maximum capacity (32 participants)")]
    RoomFull,

    #[error("Participant not found in session: {0}")]
    ParticipantNotFound(String),

    #[error("Call session is already terminated")]
    CallTerminated,
}

pub type Result<T> = std::result::Result<T, CallError>;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum CallType {
    AudioOnly,
    AudioVideo,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum CallStatus {
    Initiating,
    Active,
    Terminated,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SFrameKeyDistribution {
    pub epoch: u64,
    pub sframe_key_bytes: [u8; 32],
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SignalingEnvelope {
    pub session_id: String,
    pub sender_did: String,
    pub target_did: Option<String>, // None for broadcast
    pub payload_type: String,       // "OFFER", "ANSWER", "ICE_CANDIDATE"
    pub sdp_or_ice: String,
    pub epoch: u64,
}

pub struct CallSession {
    pub session_id: String,
    pub initiator_did: String,
    pub call_type: CallType,
    pub status: CallStatus,
    pub participants: HashMap<String, u64>, // DID -> joined timestamp
    current_epoch: u64,
    current_sframe_key: [u8; 32],
}

impl CallSession {
    /// Initializes a new 1:1 or Group Call (up to 32 participants)
    pub fn new(initiator_did: &str, call_type: CallType, timestamp: u64) -> Self {
        let mut initial_sframe_key = [0u8; 32];
        OsRng.fill_bytes(&mut initial_sframe_key);

        let mut participants = HashMap::new();
        participants.insert(initiator_did.to_string(), timestamp);

        let session_id = format!("call_{}_{}", initiator_did, timestamp);

        Self {
            session_id,
            initiator_did: initiator_did.to_string(),
            call_type,
            status: CallStatus::Initiating,
            participants,
            current_epoch: 1,
            current_sframe_key: initial_sframe_key,
        }
    }

    /// Adds a participant to the call and issues the active SFrame room key
    pub fn join(&mut self, participant_did: &str, timestamp: u64) -> Result<SFrameKeyDistribution> {
        if self.status == CallStatus::Terminated {
            return Err(CallError::CallTerminated);
        }

        if self.participants.len() >= MAXIMUM_CALL_PARTICIPANTS {
            return Err(CallError::RoomFull);
        }

        self.participants.insert(participant_did.to_string(), timestamp);
        self.status = CallStatus::Active;

        Ok(SFrameKeyDistribution {
            epoch: self.current_epoch,
            sframe_key_bytes: self.current_sframe_key,
        })
    }

    /// Removes a participant and rotates the SFrame room key (Backward/Forward Secrecy)
    pub fn leave(&mut self, participant_did: &str) -> Result<Option<SFrameKeyDistribution>> {
        if self.participants.remove(participant_did).is_none() {
            return Err(CallError::ParticipantNotFound(participant_did.to_string()));
        }

        if self.participants.is_empty() {
            self.status = CallStatus::Terminated;
            Ok(None)
        } else {
            // Rotate SFrame key so leaving participant cannot decrypt future media frames
            self.current_epoch += 1;
            OsRng.fill_bytes(&mut self.current_sframe_key);

            Ok(Some(SFrameKeyDistribution {
                epoch: self.current_epoch,
                sframe_key_bytes: self.current_sframe_key,
            }))
        }
    }

    /// Generates WebRTC signaling packet for Offer/Answer/ICE
    pub fn create_signaling(
        &self,
        sender_did: &str,
        target_did: Option<&str>,
        payload_type: &str,
        sdp_or_ice: &str,
    ) -> Result<SignalingEnvelope> {
        if self.status == CallStatus::Terminated {
            return Err(CallError::CallTerminated);
        }

        Ok(SignalingEnvelope {
            session_id: self.session_id.clone(),
            sender_did: sender_did.to_string(),
            target_did: target_did.map(|s| s.to_string()),
            payload_type: payload_type.to_string(),
            sdp_or_ice: sdp_or_ice.to_string(),
            epoch: self.current_epoch,
        })
    }

    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }
}
