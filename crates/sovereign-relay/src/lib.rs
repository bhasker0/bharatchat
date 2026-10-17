use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use thiserror::Error;
use sovereign_core::MessagePacket;

#[derive(Error, Debug)]
pub enum RelayError {
    #[error("Packet verification failed: missing or invalid traceability envelope")]
    InvalidTraceabilityEnvelope,

    #[error("Packet routing failed: {0}")]
    RoutingError(String),
}

pub type Result<T> = std::result::Result<T, RelayError>;

#[derive(Clone, Debug)]
pub struct QueuedPacket {
    pub packet: MessagePacket,
    pub queued_at: u64,
    pub expires_at: u64,
}

/// Zero-Retention In-Memory Ephemeral Relay Router
#[derive(Clone)]
pub struct RelayRouter {
    /// Active connected sessions: DID -> sender channel
    active_sessions: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<MessagePacket>>>>,
    /// Ephemeral in-memory offline buffers: DID -> Vec<QueuedPacket>
    offline_buffers: Arc<RwLock<HashMap<String, Vec<QueuedPacket>>>>,
    /// Default TTL for offline packets in seconds (e.g., 7 days = 604,800 sec)
    default_ttl_sec: u64,
}

impl RelayRouter {
    pub fn new(default_ttl_sec: u64) -> Self {
        Self {
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            offline_buffers: Arc::new(RwLock::new(HashMap::new())),
            default_ttl_sec,
        }
    }

    /// Registers an active client connection sink
    pub async fn register_client(
        &self,
        did: &str,
    ) -> mpsc::UnboundedReceiver<MessagePacket> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut sessions = self.active_sessions.write().await;
        sessions.insert(did.to_string(), tx);
        rx
    }

    /// Unregisters an active client connection sink
    pub async fn unregister_client(&self, did: &str) {
        let mut sessions = self.active_sessions.write().await;
        sessions.remove(did);
    }

    /// Checks if a DID has an active connected session
    pub async fn is_online(&self, did: &str) -> bool {
        let sessions = self.active_sessions.read().await;
        sessions.contains_key(did)
    }

    /// Routes an encrypted packet directly to recipient or stages in ephemeral memory
    pub async fn route_packet(
        &self,
        packet: MessagePacket,
        current_timestamp: u64,
    ) -> Result<bool> {
        // Validation: Every packet MUST contain an intact, valid TraceabilityEnvelope (Rule 4(2))
        if packet.trace_envelope.content_cid.is_empty() {
            return Err(RelayError::InvalidTraceabilityEnvelope);
        }

        let recipient = &packet.recipient_did;
        let sessions = self.active_sessions.read().await;

        if let Some(tx) = sessions.get(recipient) {
            // Recipient is online: stream directly to receiver sink
            tx.send(packet).map_err(|e| RelayError::RoutingError(e.to_string()))?;
            Ok(true) // Delivered live
        } else {
            // Recipient is offline: store temporarily in ephemeral memory buffer
            drop(sessions);
            let mut buffers = self.offline_buffers.write().await;
            let queue = buffers.entry(recipient.clone()).or_insert_with(Vec::new);

            queue.push(QueuedPacket {
                packet,
                queued_at: current_timestamp,
                expires_at: current_timestamp + self.default_ttl_sec,
            });

            Ok(false) // Staged offline
        }
    }

    /// Drains and retrieves all pending offline packets for a reconnected DID
    pub async fn drain_offline_packets(
        &self,
        did: &str,
        current_timestamp: u64,
    ) -> Vec<MessagePacket> {
        let mut buffers = self.offline_buffers.write().await;
        if let Some(queue) = buffers.remove(did) {
            queue
                .into_iter()
                .filter(|qp| qp.expires_at > current_timestamp)
                .map(|qp| qp.packet)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Purges all buffered records for a DID (DPDP Act Section 12 Right to Erasure)
    pub async fn purge_user(&self, did: &str) {
        let mut sessions = self.active_sessions.write().await;
        sessions.remove(did);
        let mut buffers = self.offline_buffers.write().await;
        buffers.remove(did);
    }
}
