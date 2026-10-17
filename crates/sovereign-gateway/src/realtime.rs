//! Real-Time WebSocket, Presence & Delivery Receipts Subsystem
//! Conforms to WhatsApp feature parity and DPDP Act Section 6 privacy mandates.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryStatus {
    /// 1 Grey Tick: Accepted into relay mesh with Rule 4(2) envelope
    Sent,
    /// 2 Grey Ticks: Delivered to recipient device
    Delivered,
    /// 2 Blue Ticks: Decrypted and viewed by recipient (if privacy allows)
    Read,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadReceiptPolicy {
    Enabled,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LastSeenPolicy {
    Everyone,
    ContactsOnly,
    Nobody,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PresenceInfo {
    pub is_online: bool,
    pub last_seen_ts: u64,
    pub policy: LastSeenPolicy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypingEvent {
    pub chat_id: String,
    pub user_did: String,
    pub is_typing: bool,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeliveryReceiptEvent {
    pub message_id: String,
    pub sender_did: String,
    pub recipient_did: String,
    pub status: DeliveryStatus,
    pub timestamp: u64,
    pub blue_ticks: bool,
}

pub struct RealtimeManager {
    /// Message ID -> (sender_did, recipient_did, DeliveryStatus)
    message_statuses: Mutex<HashMap<String, (String, String, DeliveryStatus)>>,
    /// User DID -> ReadReceiptPolicy (DPDP Section 6)
    read_receipt_policies: Mutex<HashMap<String, ReadReceiptPolicy>>,
    /// User DID -> PresenceInfo
    presence_registry: Mutex<HashMap<String, PresenceInfo>>,
    /// User DID -> Set of approved Contact DIDs
    contacts_graph: Mutex<HashMap<String, HashSet<String>>>,
    /// Chat ID -> Active Typers (User DID -> Timestamp)
    active_typers: Mutex<HashMap<String, HashMap<String, u64>>>,
}

impl RealtimeManager {
    pub fn new() -> Self {
        Self {
            message_statuses: Mutex::new(HashMap::new()),
            read_receipt_policies: Mutex::new(HashMap::new()),
            presence_registry: Mutex::new(HashMap::new()),
            contacts_graph: Mutex::new(HashMap::new()),
            active_typers: Mutex::new(HashMap::new()),
        }
    }

    /// Set user's Read Receipt privacy policy (DPDP Section 6)
    pub fn set_read_receipt_policy(&self, user_did: &str, policy: ReadReceiptPolicy) {
        let mut map = self.read_receipt_policies.lock().unwrap();
        map.insert(user_did.to_string(), policy);
    }

    /// Add an approved contact relationship
    pub fn add_contact(&self, user_did: &str, contact_did: &str) {
        let mut graph = self.contacts_graph.lock().unwrap();
        graph
            .entry(user_did.to_string())
            .or_insert_with(HashSet::new)
            .insert(contact_did.to_string());
    }

    /// Record message acceptance by relay -> 1 Grey Tick (Sent)
    pub fn record_sent(
        &self,
        message_id: &str,
        sender_did: &str,
        recipient_did: &str,
        timestamp: u64,
    ) -> DeliveryReceiptEvent {
        let mut map = self.message_statuses.lock().unwrap();
        map.insert(
            message_id.to_string(),
            (
                sender_did.to_string(),
                recipient_did.to_string(),
                DeliveryStatus::Sent,
            ),
        );

        DeliveryReceiptEvent {
            message_id: message_id.to_string(),
            sender_did: sender_did.to_string(),
            recipient_did: recipient_did.to_string(),
            status: DeliveryStatus::Sent,
            timestamp,
            blue_ticks: false,
        }
    }

    /// Record delivery to active device -> 2 Grey Ticks (Delivered)
    pub fn record_delivered(&self, message_id: &str, timestamp: u64) -> Option<DeliveryReceiptEvent> {
        let mut map = self.message_statuses.lock().unwrap();
        if let Some((sender, recipient, status)) = map.get_mut(message_id) {
            *status = DeliveryStatus::Delivered;
            Some(DeliveryReceiptEvent {
                message_id: message_id.to_string(),
                sender_did: sender.clone(),
                recipient_did: recipient.clone(),
                status: DeliveryStatus::Delivered,
                timestamp,
                blue_ticks: false,
            })
        } else {
            None
        }
    }

    /// Record message read by recipient -> 2 Blue Ticks (Read) or 2 Grey Ticks if privacy disabled
    pub fn record_read(
        &self,
        message_id: &str,
        reader_did: &str,
        timestamp: u64,
    ) -> Option<DeliveryReceiptEvent> {
        let mut map = self.message_statuses.lock().unwrap();
        if let Some((sender, recipient, status)) = map.get_mut(message_id) {
            if recipient != reader_did {
                return None;
            }

            // Check DPDP read receipt preferences for both parties
            let policies = self.read_receipt_policies.lock().unwrap();
            let sender_policy = policies
                .get(sender)
                .copied()
                .unwrap_or(ReadReceiptPolicy::Enabled);
            let recipient_policy = policies
                .get(recipient)
                .copied()
                .unwrap_or(ReadReceiptPolicy::Enabled);

            let allow_blue_ticks = sender_policy == ReadReceiptPolicy::Enabled
                && recipient_policy == ReadReceiptPolicy::Enabled;

            if allow_blue_ticks {
                *status = DeliveryStatus::Read;
                Some(DeliveryReceiptEvent {
                    message_id: message_id.to_string(),
                    sender_did: sender.clone(),
                    recipient_did: recipient.clone(),
                    status: DeliveryStatus::Read,
                    timestamp,
                    blue_ticks: true,
                })
            } else {
                // Keep status at Delivered to preserve caller's privacy
                *status = DeliveryStatus::Delivered;
                Some(DeliveryReceiptEvent {
                    message_id: message_id.to_string(),
                    sender_did: sender.clone(),
                    recipient_did: recipient.clone(),
                    status: DeliveryStatus::Delivered,
                    timestamp,
                    blue_ticks: false,
                })
            }
        } else {
            None
        }
    }

    /// Update user presence and last seen policy
    pub fn update_presence(
        &self,
        user_did: &str,
        is_online: bool,
        timestamp: u64,
        policy: LastSeenPolicy,
    ) {
        let mut reg = self.presence_registry.lock().unwrap();
        reg.insert(
            user_did.to_string(),
            PresenceInfo {
                is_online,
                last_seen_ts: timestamp,
                policy,
            },
        );
    }

    /// Query peer presence respecting DPDP last-seen privacy rules
    pub fn query_presence(&self, viewer_did: &str, target_did: &str) -> Option<PresenceInfo> {
        let reg = self.presence_registry.lock().unwrap();
        let info = reg.get(target_did)?;

        match info.policy {
            LastSeenPolicy::Everyone => Some(info.clone()),
            LastSeenPolicy::Nobody => None,
            LastSeenPolicy::ContactsOnly => {
                let graph = self.contacts_graph.lock().unwrap();
                if let Some(contacts) = graph.get(target_did) {
                    if contacts.contains(viewer_did) {
                        return Some(info.clone());
                    }
                }
                None
            }
        }
    }

    /// Set ephemeral typing state
    pub fn set_typing(&self, chat_id: &str, user_did: &str, is_typing: bool, timestamp: u64) {
        let mut typers = self.active_typers.lock().unwrap();
        let chat_typers = typers.entry(chat_id.to_string()).or_insert_with(HashMap::new);
        if is_typing {
            chat_typers.insert(user_did.to_string(), timestamp);
        } else {
            chat_typers.remove(user_did);
        }
    }

    /// Query active typers in a chat (evicts expired timestamps > 5s)
    pub fn get_active_typers(&self, chat_id: &str, current_ts: u64) -> Vec<String> {
        let mut typers = self.active_typers.lock().unwrap();
        if let Some(chat_typers) = typers.get_mut(chat_id) {
            chat_typers.retain(|_, ts| current_ts.saturating_sub(*ts) <= 5);
            chat_typers.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for RealtimeManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// RFC 6455 WebSocket Handshake & Framing Implementation
// ============================================================================

/// Computes the RFC 6455 Sec-WebSocket-Accept response header
pub fn compute_ws_accept(sec_websocket_key: &str) -> String {
    let magic = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let combined = format!("{}{}", sec_websocket_key.trim(), magic);
    let hash = sha1_digest(combined.as_bytes());
    base64_encode(&hash)
}

/// Encodes a text payload into a standard RFC 6455 unmasked WebSocket frame
pub fn encode_ws_text_frame(text: &str) -> Vec<u8> {
    let payload = text.as_bytes();
    let len = payload.len();
    let mut frame = Vec::with_capacity(len + 10);

    // FIN bit set (0x80) + Opcode 0x1 (Text)
    frame.push(0x81);

    if len <= 125 {
        frame.push(len as u8);
    } else if len <= 65535 {
        frame.push(126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(len as u64).to_be_bytes());
    }

    frame.extend_from_slice(payload);
    frame
}

/// Parses an incoming RFC 6455 WebSocket frame (supports unmasking)
pub fn parse_ws_frame(data: &[u8]) -> Option<(u8, Vec<u8>)> {
    if data.len() < 2 {
        return None;
    }

    let first = data[0];
    let opcode = first & 0x0F;
    let second = data[1];
    let is_masked = (second & 0x80) != 0;
    let mut payload_len = (second & 0x7F) as usize;

    let mut cursor = 2;
    if payload_len == 126 {
        if data.len() < cursor + 2 {
            return None;
        }
        payload_len = u16::from_be_bytes([data[cursor], data[cursor + 1]]) as usize;
        cursor += 2;
    } else if payload_len == 127 {
        if data.len() < cursor + 8 {
            return None;
        }
        let mut b = [0u8; 8];
        b.copy_from_slice(&data[cursor..cursor + 8]);
        payload_len = u64::from_be_bytes(b) as usize;
        cursor += 8;
    }

    let mask = if is_masked {
        if data.len() < cursor + 4 {
            return None;
        }
        let m = [data[cursor], data[cursor + 1], data[cursor + 2], data[cursor + 3]];
        cursor += 4;
        Some(m)
    } else {
        None
    };

    if data.len() < cursor + payload_len {
        return None;
    }

    let mut payload = data[cursor..cursor + payload_len].to_vec();
    if let Some(m) = mask {
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= m[i % 4];
        }
    }

    Some((opcode, payload))
}

// ============================================================================
// Zero-Dependency FIPS 180-1 SHA-1 & Base64 Routines for RFC 6455
// ============================================================================

fn sha1_digest(data: &[u8]) -> [u8; 20] {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    let ml = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&ml.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 80];
        for (i, part) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([part[0], part[1], part[2], part[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1u32),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDCu32),
                _ => (b ^ c ^ d, 0xCA62C1D6u32),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut out = [0u8; 20];
    out[0..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out
}

fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;
    while i < input.len() {
        let b0 = input[i] as u32;
        let b1 = if i + 1 < input.len() { input[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < input.len() { input[i + 2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);

        if i + 1 < input.len() {
            out.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }

        if i + 2 < input.len() {
            out.push(TABLE[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }

        i += 3;
    }
    out
}
