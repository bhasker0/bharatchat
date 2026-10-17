pub mod abuse_report;
pub mod channels;
pub mod forwarding;
pub mod status;

pub use abuse_report::{AbuseCategory, AbuseDossier, AbuseReportError, AbuseReportRegistry, GrievanceStatus, GrievanceTicket};
pub use channels::{BroadcastChannel, ChannelError, ChannelManager, ChannelPost};
pub use forwarding::{ForwardEnvelope, ForwardLabel, ForwardingError};
pub use status::{StatusAudience, StatusManager, StatusStory};

/// Unified Sovereign Broadcast & Misinformation Prevention Subsystem
pub struct BroadcastService {
    pub status: status::StatusManager,
    pub channels: channels::ChannelManager,
    pub abuse: abuse_report::AbuseReportRegistry,
}

impl BroadcastService {
    pub fn new() -> Self {
        Self {
            status: status::StatusManager::new(),
            channels: channels::ChannelManager::new(),
            abuse: abuse_report::AbuseReportRegistry::new(),
        }
    }
}

impl Default for BroadcastService {
    fn default() -> Self {
        Self::new()
    }
}
