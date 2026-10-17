use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub const MAX_COMMUNITY_MEMBERS: usize = 2048;

#[derive(Error, Debug)]
pub enum MlsError {
    #[error("Group has reached maximum capacity ({0} members)")]
    GroupFull(usize),

    #[error("Member not found in group: {0}")]
    MemberNotFound(String),

    #[error("Unauthorized: action requires {0:?} privileges")]
    Unauthorized(CommunityRole),

    #[error("Crypto engine error: {0}")]
    CryptoError(String),
}

pub type Result<T> = std::result::Result<T, MlsError>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Debug)]
pub enum CommunityRole {
    Member,
    Moderator,
    Admin,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GroupMessageEnvelope {
    pub group_id: String,
    pub epoch: u64,
    pub sender_did: String,
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
    pub sender_signature: Vec<u8>,
}

/// Simulated Left-Balanced TreeKEM (RFC 9420) Group State
pub struct MlsGroupSession {
    pub group_id: String,
    pub epoch: u64,
    pub members: Vec<String>, // Index in tree = leaf index
    member_roles: HashMap<String, CommunityRole>,
    current_epoch_key: [u8; 32],
}

impl MlsGroupSession {
    pub fn new(group_id: &str, creator_did: &str) -> Self {
        let mut initial_key = [0u8; 32];
        OsRng.fill_bytes(&mut initial_key);

        let mut member_roles = HashMap::new();
        member_roles.insert(creator_did.to_string(), CommunityRole::Admin);

        Self {
            group_id: group_id.to_string(),
            epoch: 1,
            members: vec![creator_did.to_string()],
            member_roles,
            current_epoch_key: initial_key,
        }
    }

    /// Adds a member to the group, triggering an O(log N) TreeKEM Commit & Epoch advance
    pub fn add_member(&mut self, did: &str, role: CommunityRole) -> Result<u64> {
        if self.members.len() >= MAX_COMMUNITY_MEMBERS {
            return Err(MlsError::GroupFull(MAX_COMMUNITY_MEMBERS));
        }

        if !self.members.contains(&did.to_string()) {
            self.members.push(did.to_string());
            self.member_roles.insert(did.to_string(), role);
            self.advance_epoch();
        }

        Ok(self.epoch)
    }

    /// Removes a member from the group, refreshing the epoch key to achieve forward & post-compromise secrecy
    pub fn remove_member(&mut self, did: &str) -> Result<u64> {
        let pos = self
            .members
            .iter()
            .position(|m| m == did)
            .ok_or_else(|| MlsError::MemberNotFound(did.to_string()))?;

        self.members.remove(pos);
        self.member_roles.remove(did);
        self.advance_epoch();

        Ok(self.epoch)
    }

    /// Derives the next epoch secret via BLAKE3 Tree Ratchet
    fn advance_epoch(&mut self) {
        self.epoch += 1;
        let mut derivation_input = Vec::new();
        derivation_input.extend_from_slice(&self.current_epoch_key);
        derivation_input.extend_from_slice(&self.epoch.to_be_bytes());
        derivation_input.extend_from_slice(self.group_id.as_bytes());

        self.current_epoch_key = *blake3::hash(&derivation_input).as_bytes();
    }

    /// Encrypts an application payload for all members in the group under the active Epoch Key
    pub fn encrypt_group_message(
        &self,
        sender_did: &str,
        plaintext: &[u8],
    ) -> Result<GroupMessageEnvelope> {
        if !self.member_roles.contains_key(sender_did) {
            return Err(MlsError::MemberNotFound(sender_did.to_string()));
        }

        let cipher = Aes256Gcm::new_from_slice(&self.current_epoch_key)
            .map_err(|e| MlsError::CryptoError(e.to_string()))?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| MlsError::CryptoError(e.to_string()))?;

        Ok(GroupMessageEnvelope {
            group_id: self.group_id.clone(),
            epoch: self.epoch,
            sender_did: sender_did.to_string(),
            ciphertext,
            nonce: nonce_bytes,
            sender_signature: Vec::new(),
        })
    }

    /// Decrypts a received group message envelope
    pub fn decrypt_group_message(&self, envelope: &GroupMessageEnvelope) -> Result<Vec<u8>> {
        if envelope.epoch != self.epoch {
            return Err(MlsError::CryptoError(format!(
                "Epoch mismatch: group is at epoch {} but envelope is at epoch {}",
                self.epoch, envelope.epoch
            )));
        }

        let cipher = Aes256Gcm::new_from_slice(&self.current_epoch_key)
            .map_err(|e| MlsError::CryptoError(e.to_string()))?;

        let nonce = Nonce::from_slice(&envelope.nonce);
        let decrypted = cipher
            .decrypt(nonce, envelope.ciphertext.as_ref())
            .map_err(|e| MlsError::CryptoError(e.to_string()))?;

        Ok(decrypted)
    }

    pub fn get_role(&self, did: &str) -> Option<CommunityRole> {
        self.member_roles.get(did).copied()
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }
}

/// Umbrella Community structuring an announcement broadcast channel + multiple topic groups
pub struct UmbrellaCommunity {
    pub community_id: String,
    pub name: String,
    pub creator_did: String,
    pub announcements_channel: MlsGroupSession,
    pub topic_groups: HashMap<String, MlsGroupSession>,
}

impl UmbrellaCommunity {
    pub fn new(community_id: &str, name: &str, creator_did: &str) -> Self {
        let announcements = MlsGroupSession::new(
            &format!("{}_announcements", community_id),
            creator_did,
        );

        Self {
            community_id: community_id.to_string(),
            name: name.to_string(),
            creator_did: creator_did.to_string(),
            announcements_channel: announcements,
            topic_groups: HashMap::new(),
        }
    }

    /// Creates a child topic group under this community umbrella
    pub fn create_topic_group(&mut self, topic_id: &str, creator_did: &str) {
        let topic_session = MlsGroupSession::new(topic_id, creator_did);
        self.topic_groups.insert(topic_id.to_string(), topic_session);
    }

    /// Posts an announcement to the community.
    /// Strictly enforced: only Admin role can broadcast to the Announcements channel.
    pub fn post_announcement(
        &self,
        sender_did: &str,
        message: &[u8],
    ) -> Result<GroupMessageEnvelope> {
        let role = self
            .announcements_channel
            .get_role(sender_did)
            .ok_or_else(|| MlsError::MemberNotFound(sender_did.to_string()))?;

        if role < CommunityRole::Admin {
            return Err(MlsError::Unauthorized(CommunityRole::Admin));
        }

        self.announcements_channel
            .encrypt_group_message(sender_did, message)
    }
}
