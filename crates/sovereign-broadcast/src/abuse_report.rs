use serde::{Deserialize, Serialize};
use sovereign_core::compute_canonical_cid;
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AbuseReportError {
    #[error("Integrity check failed: plaintext BLAKE3 hash does not match reported message CID")]
    HashMismatch,
    #[error("Signature verification failed: sender DID did not sign this message")]
    InvalidSenderSignature,
    #[error("Grievance ticket '{0}' not found")]
    TicketNotFound(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbuseCategory {
    Csam,
    Terrorism,
    HateSpeech,
    Harassment,
    FinancialFraud,
    Copyright,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrievanceStatus {
    /// Acknowledged within 24 hours under IT Rules 2021 Rule 3(2)
    Acknowledged,
    /// Under active review by Resident Grievance Officer
    InInvestigation,
    /// Action taken (account suspended / content blocked / CERT-In alerted)
    Resolved,
    /// Dismissed as frivolous / unsubstantiated
    Dismissed,
}

/// Verifiable Cryptographic Abuse Dossier
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AbuseDossier {
    pub dossier_id: String,
    pub reported_message_cid: String,
    pub sender_did: String,
    pub sender_signature_hex: String,
    pub plaintext: String,
    pub reporter_did: String,
    pub category: AbuseCategory,
    pub timestamp: u64,
    pub jurisdiction: String,
}

/// Official Regulatory Grievance Record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrievanceTicket {
    pub ticket_id: String,
    pub dossier: AbuseDossier,
    pub status: GrievanceStatus,
    pub created_at: u64,
    pub acknowledged_at: u64,
    pub resolution_deadline: u64, // 15 days under IT Rules Rule 3(2)
}

pub struct AbuseReportRegistry {
    tickets: Mutex<HashMap<String, GrievanceTicket>>,
}

impl AbuseReportRegistry {
    pub fn new() -> Self {
        Self {
            tickets: Mutex::new(HashMap::new()),
        }
    }

    /// Submit a verifiable abuse dossier with cryptographic integrity checks
    pub fn submit_report(
        &self,
        dossier: AbuseDossier,
        sender_verifying_key: [u8; 32],
    ) -> Result<GrievanceTicket, AbuseReportError> {
        // 1. Verify Plaintext BLAKE3 Canonical CID
        let computed_cid = hex::encode(compute_canonical_cid(dossier.plaintext.as_bytes()));
        if computed_cid != dossier.reported_message_cid {
            return Err(AbuseReportError::HashMismatch);
        }

        // 2. Verify Sender Ed25519 Signature
        // Sign message = computed_cid || timestamp
        let mut sign_msg = Vec::new();
        sign_msg.extend_from_slice(computed_cid.as_bytes());
        sign_msg.extend_from_slice(&dossier.timestamp.to_be_bytes());

        let sig_bytes = hex::decode(&dossier.sender_signature_hex)
            .map_err(|_| AbuseReportError::InvalidSenderSignature)?;
        if sig_bytes.len() != 64 {
            return Err(AbuseReportError::InvalidSenderSignature);
        }

        // Verify ed25519 signature via DidDocument
        let doc = sovereign_core::DidDocument {
            did: dossier.sender_did.clone(),
            verifying_key_bytes: sender_verifying_key,
        };
        if doc.verify(&sign_msg, &sig_bytes).is_err() {
            return Err(AbuseReportError::InvalidSenderSignature);
        }

        // 3. Issue statutory grievance ticket (24h ack, 15d resolution deadline)
        let ticket_id = format!("GRV-IN-{}", dossier.dossier_id);
        let ticket = GrievanceTicket {
            ticket_id: ticket_id.clone(),
            dossier,
            status: GrievanceStatus::Acknowledged,
            created_at: 1776000000,
            acknowledged_at: 1776000000,
            resolution_deadline: 1776000000 + (15 * 86400),
        };

        let mut map = self.tickets.lock().unwrap();
        map.insert(ticket_id, ticket.clone());

        Ok(ticket)
    }

    pub fn get_ticket(&self, ticket_id: &str) -> Option<GrievanceTicket> {
        let map = self.tickets.lock().unwrap();
        map.get(ticket_id).cloned()
    }
}

impl Default for AbuseReportRegistry {
    fn default() -> Self {
        Self::new()
    }
}
