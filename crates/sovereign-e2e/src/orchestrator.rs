use sovereign_chain::{ConsentScope, ConsortiumLedger};
use sovereign_core::IdentityKeyPair;
use sovereign_relay::RelayRouter;
use sovereign_storage::SovereignStorageCluster;
use std::sync::{Arc, Mutex};

pub struct SovereignAppEnvironment {
    pub chain: Arc<Mutex<ConsortiumLedger>>,
    pub relay: RelayRouter,
    pub storage: SovereignStorageCluster,
}

impl SovereignAppEnvironment {
    pub fn new() -> Self {
        Self {
            chain: Arc::new(Mutex::new(ConsortiumLedger::new())),
            relay: RelayRouter::new(3600), // 1 hour ephemeral retention
            storage: SovereignStorageCluster::new(),
        }
    }

    /// Onboards a new user with DID and registers on the consortium ledger with DPDP consent
    pub fn onboard_user(&self, fiduciary_did: &str, timestamp: u64) -> IdentityKeyPair {
        let identity = IdentityKeyPair::generate();
        let vk_bytes = identity.document().verifying_key_bytes;
        let mut chain = self.chain.lock().unwrap();
        chain.register_did(&identity.did, vk_bytes, timestamp).unwrap();
        chain.grant_consent(
            &identity.did,
            fiduciary_did,
            ConsentScope::TransactionalNotifications,
            timestamp,
            86400 * 365,
        );
        identity
    }
}

impl Default for SovereignAppEnvironment {
    fn default() -> Self {
        Self::new()
    }
}
