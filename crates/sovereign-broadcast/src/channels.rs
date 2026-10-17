use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("Channel '{0}' not found")]
    NotFound(String),
    #[error("Unauthorized: only channel admin '{0}' can publish")]
    UnauthorizedAdmin(String),
    #[error("Post '{0}' not found in channel '{1}'")]
    PostNotFound(String, String),
    #[error("Private channel requires subscription to view")]
    SubscriptionRequired,
    #[error("Channel '{0}' already exists")]
    AlreadyExists(String),
}

/// One-to-Many Public and Private Broadcast Channel (WhatsApp Channels Feature Parity)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BroadcastChannel {
    pub channel_id: String,
    pub name: String,
    pub description: String,
    pub admin_did: String,
    pub is_public: bool,
    pub created_at: u64,
    pub subscriber_count: usize,
}

/// Tamper-Evident Channel Post Signed with Admin Ed25519 Key
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelPost {
    pub post_id: String,
    pub channel_id: String,
    pub admin_did: String,
    pub content_cid: String,
    pub text: String,
    pub published_at: u64,
    pub signature_hex: String,
    /// Aggregate blinded reactions (Emoji -> Count). Subscriber DIDs remain private.
    pub reactions: HashMap<String, u32>,
}

impl ChannelPost {
    /// Cryptographically verify the tamper-evident post against the admin's Ed25519 verifying key
    pub fn verify_post_signature(&self, admin_verifying_key: [u8; 32]) -> bool {
        let mut sign_msg = Vec::new();
        sign_msg.extend_from_slice(self.content_cid.as_bytes());
        sign_msg.extend_from_slice(&self.published_at.to_be_bytes());

        let sig_bytes = match hex::decode(&self.signature_hex) {
            Ok(b) if b.len() == 64 => b,
            _ => return false,
        };

        let doc = sovereign_core::DidDocument {
            did: self.admin_did.clone(),
            verifying_key_bytes: admin_verifying_key,
        };
        doc.verify(&sign_msg, &sig_bytes).is_ok()
    }
}

pub struct ChannelManager {
    channels: Mutex<HashMap<String, (BroadcastChannel, HashSet<String>)>>,
    posts: Mutex<HashMap<String, Vec<ChannelPost>>>,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            channels: Mutex::new(HashMap::new()),
            posts: Mutex::new(HashMap::new()),
        }
    }

    pub fn create_channel(
        &self,
        channel_id: &str,
        name: &str,
        description: &str,
        admin_did: &str,
        is_public: bool,
        created_at: u64,
    ) -> Result<BroadcastChannel, ChannelError> {
        let mut map = self.channels.lock().unwrap();
        if map.contains_key(channel_id) {
            return Err(ChannelError::AlreadyExists(channel_id.to_string()));
        }

        let mut subscribers = HashSet::new();
        subscribers.insert(admin_did.to_string()); // Admin is default subscriber

        let channel = BroadcastChannel {
            channel_id: channel_id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            admin_did: admin_did.to_string(),
            is_public,
            created_at,
            subscriber_count: 1,
        };

        map.insert(channel_id.to_string(), (channel.clone(), subscribers));
        Ok(channel)
    }

    pub fn subscribe(&self, channel_id: &str, subscriber_did: &str) -> Result<(), ChannelError> {
        let mut map = self.channels.lock().unwrap();
        if let Some((channel, subs)) = map.get_mut(channel_id) {
            if subs.insert(subscriber_did.to_string()) {
                channel.subscriber_count = subs.len();
            }
            Ok(())
        } else {
            Err(ChannelError::NotFound(channel_id.to_string()))
        }
    }

    pub fn publish_post(
        &self,
        channel_id: &str,
        post_id: &str,
        author_did: &str,
        content_cid: &str,
        text: &str,
        signature_hex: &str,
        timestamp: u64,
    ) -> Result<ChannelPost, ChannelError> {
        let channels = self.channels.lock().unwrap();
        let (channel, _) = channels
            .get(channel_id)
            .ok_or_else(|| ChannelError::NotFound(channel_id.to_string()))?;

        // Strict admin authorization
        if channel.admin_did != author_did {
            return Err(ChannelError::UnauthorizedAdmin(channel.admin_did.clone()));
        }

        let post = ChannelPost {
            post_id: post_id.to_string(),
            channel_id: channel_id.to_string(),
            admin_did: author_did.to_string(),
            content_cid: content_cid.to_string(),
            text: text.to_string(),
            published_at: timestamp,
            signature_hex: signature_hex.to_string(),
            reactions: HashMap::new(),
        };

        let mut posts_map = self.posts.lock().unwrap();
        posts_map
            .entry(channel_id.to_string())
            .or_insert_with(Vec::new)
            .push(post.clone());

        Ok(post)
    }

    pub fn add_reaction(
        &self,
        channel_id: &str,
        post_id: &str,
        emoji: &str,
    ) -> Result<u32, ChannelError> {
        let mut posts_map = self.posts.lock().unwrap();
        let channel_posts = posts_map
            .get_mut(channel_id)
            .ok_or_else(|| ChannelError::NotFound(channel_id.to_string()))?;

        let post = channel_posts
            .iter_mut()
            .find(|p| p.post_id == post_id)
            .ok_or_else(|| ChannelError::PostNotFound(post_id.to_string(), channel_id.to_string()))?;

        let count = post.reactions.entry(emoji.to_string()).or_insert(0);
        *count += 1;
        Ok(*count)
    }

    pub fn get_channel_feed(
        &self,
        channel_id: &str,
        viewer_did: &str,
    ) -> Result<Vec<ChannelPost>, ChannelError> {
        let channels = self.channels.lock().unwrap();
        let (channel, subs) = channels
            .get(channel_id)
            .ok_or_else(|| ChannelError::NotFound(channel_id.to_string()))?;

        if !channel.is_public && !subs.contains(viewer_did) {
            return Err(ChannelError::SubscriptionRequired);
        }

        let posts_map = self.posts.lock().unwrap();
        Ok(posts_map.get(channel_id).cloned().unwrap_or_default())
    }
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}
