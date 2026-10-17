use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

/// Granular Privacy Audience for 24-Hour Ephemeral Status Stories
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusAudience {
    /// Available to all mutual contacts
    MyContacts,
    /// Available to contacts except explicitly excluded DIDs
    MyContactsExcept(Vec<String>),
    /// Whitelist only: available strictly to selected DIDs
    OnlyShareWith(Vec<String>),
}

/// 24-Hour Ephemeral Status Story Model (WhatsApp Feature Parity)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusStory {
    pub story_id: String,
    pub author_did: String,
    pub content_cid: String,
    pub caption: String,
    pub created_at: u64,
    pub expires_at: u64, // created_at + 86400 (24h)
    pub audience: StatusAudience,
    pub views: Vec<String>,
}

impl StatusStory {
    pub fn new(
        story_id: &str,
        author_did: &str,
        content_cid: &str,
        caption: &str,
        created_at: u64,
        audience: StatusAudience,
    ) -> Self {
        Self {
            story_id: story_id.to_string(),
            author_did: author_did.to_string(),
            content_cid: content_cid.to_string(),
            caption: caption.to_string(),
            created_at,
            expires_at: created_at + 86400, // Strictly 24 hours
            audience,
            views: Vec::new(),
        }
    }

    /// Check if story has passed its 24-hour statutory lifetime
    pub fn is_expired(&self, current_ts: u64) -> bool {
        current_ts >= self.expires_at
    }

    /// Check whether a requesting viewer is authorized to decrypt/view this status
    pub fn can_view(&self, viewer_did: &str, is_contact: bool) -> bool {
        if viewer_did == self.author_did {
            return true;
        }

        match &self.audience {
            StatusAudience::MyContacts => is_contact,
            StatusAudience::MyContactsExcept(excluded) => {
                is_contact && !excluded.iter().any(|d| d == viewer_did)
            }
            StatusAudience::OnlyShareWith(whitelist) => {
                whitelist.iter().any(|d| d == viewer_did)
            }
        }
    }

    /// Record a view receipt from a peer
    pub fn record_view(&mut self, viewer_did: &str) {
        if !self.views.iter().any(|v| v == viewer_did) {
            self.views.push(viewer_did.to_string());
        }
    }
}

pub struct StatusManager {
    /// Author DID -> Active Stories
    stories: Mutex<HashMap<String, Vec<StatusStory>>>,
    /// User DID -> Set of approved Contact DIDs
    contacts: Mutex<HashMap<String, HashSet<String>>>,
}

impl StatusManager {
    pub fn new() -> Self {
        Self {
            stories: Mutex::new(HashMap::new()),
            contacts: Mutex::new(HashMap::new()),
        }
    }

    pub fn add_contact(&self, user_did: &str, contact_did: &str) {
        let mut map = self.contacts.lock().unwrap();
        map.entry(user_did.to_string())
            .or_insert_with(HashSet::new)
            .insert(contact_did.to_string());
    }

    pub fn post_status(
        &self,
        story_id: &str,
        author_did: &str,
        content_cid: &str,
        caption: &str,
        created_at: u64,
        audience: StatusAudience,
    ) -> StatusStory {
        let story = StatusStory::new(story_id, author_did, content_cid, caption, created_at, audience);
        let mut map = self.stories.lock().unwrap();
        map.entry(author_did.to_string())
            .or_insert_with(Vec::new)
            .push(story.clone());
        story
    }

    pub fn get_active_statuses(
        &self,
        author_did: &str,
        viewer_did: &str,
        current_ts: u64,
    ) -> Vec<StatusStory> {
        let contacts_map = self.contacts.lock().unwrap();
        let is_contact = contacts_map
            .get(author_did)
            .map(|set| set.contains(viewer_did))
            .unwrap_or(false);

        let mut stories_map = self.stories.lock().unwrap();
        if let Some(user_stories) = stories_map.get_mut(author_did) {
            // Evict expired statuses (crypto-shredding)
            user_stories.retain(|s| !s.is_expired(current_ts));

            user_stories
                .iter_mut()
                .filter(|s| s.can_view(viewer_did, is_contact))
                .map(|s| {
                    if viewer_did != author_did {
                        s.record_view(viewer_did);
                    }
                    s.clone()
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Purge and crypto-shred all statuses that have exceeded their 24-hour lifetime
    pub fn purge_expired(&self, current_ts: u64) -> usize {
        let mut map = self.stories.lock().unwrap();
        let mut purged = 0;
        for stories in map.values_mut() {
            let initial = stories.len();
            stories.retain(|s| !s.is_expired(current_ts));
            purged += initial - stories.len();
        }
        purged
    }
}

impl Default for StatusManager {
    fn default() -> Self {
        Self::new()
    }
}
