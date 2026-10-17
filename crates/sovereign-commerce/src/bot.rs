use crate::interactive::InteractiveMessage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BusinessProfile {
    pub merchant_did: String,
    pub business_name: String,
    pub description: String,
    pub working_hours_start: u32, // e.g. 9 for 09:00 IST
    pub working_hours_end: u32,   // e.g. 19 for 19:00 IST
    pub is_away_manual: bool,
    pub away_message: String,
    pub greeting_message: String,
}

impl BusinessProfile {
    pub fn default_retailer(merchant_did: &str, business_name: &str) -> Self {
        Self {
            merchant_did: merchant_did.to_string(),
            business_name: business_name.to_string(),
            description: "Official Verified Merchant on BharatChat Sovereign Network".to_string(),
            working_hours_start: 9,
            working_hours_end: 19,
            is_away_manual: false,
            away_message: "Thank you for reaching out! We are currently outside our operating hours (09:00 - 19:00 IST). We will respond as soon as we open.".to_string(),
            greeting_message: format!("Namaste! Welcome to {}. How can we serve you today?", business_name),
        }
    }

    /// Determines if the business is currently away
    pub fn is_away(&self, current_hour_ist: u32) -> bool {
        if self.is_away_manual {
            return true;
        }
        current_hour_ist < self.working_hours_start || current_hour_ist >= self.working_hours_end
    }
}

pub struct ConversationalBotEngine {
    pub profile: BusinessProfile,
    pub last_interaction_map: HashMap<String, u64>, // customer_did -> timestamp
    pub keyword_routes: HashMap<String, InteractiveMessage>,
}

impl ConversationalBotEngine {
    pub fn new(profile: BusinessProfile) -> Self {
        Self {
            profile,
            last_interaction_map: HashMap::new(),
            keyword_routes: HashMap::new(),
        }
    }

    pub fn register_keyword_route(&mut self, keyword: &str, message: InteractiveMessage) {
        self.keyword_routes
            .insert(keyword.to_uppercase().trim().to_string(), message);
    }

    /// Handles an incoming inbound message from a customer
    pub fn process_inbound(
        &mut self,
        customer_did: &str,
        text: &str,
        timestamp: u64,
        current_hour_ist: u32,
    ) -> Option<InteractiveMessage> {
        let is_first_contact = match self.last_interaction_map.get(customer_did) {
            None => true,
            Some(&last) => timestamp.saturating_sub(last) > 86400, // 24 hours
        };

        // Update interaction timestamp
        self.last_interaction_map
            .insert(customer_did.to_string(), timestamp);

        // Check if away
        if self.profile.is_away(current_hour_ist) {
            return Some(InteractiveMessage::new_button_message(
                &self.profile.away_message,
                vec![
                    ("view_catalog", "Browse Catalog"),
                    ("leave_message", "Leave Message"),
                ],
                Some(&self.profile.business_name),
                Some("Automated Response"),
            ));
        }

        // Check keyword matches first
        let cleaned = text.trim().to_uppercase();
        if let Some(msg) = self.keyword_routes.get(&cleaned) {
            return Some(msg.clone());
        }

        // If first contact in 24 hours, send greeting
        if is_first_contact {
            return Some(InteractiveMessage::new_button_message(
                &self.profile.greeting_message,
                vec![
                    ("view_catalog", "🛍️ View Catalog"),
                    ("track_order", "📦 Track Order"),
                    ("talk_human", "💬 Live Support"),
                ],
                Some(&self.profile.business_name),
                Some("Powered by BharatChat Sovereign Network"),
            ));
        }

        None
    }
}
