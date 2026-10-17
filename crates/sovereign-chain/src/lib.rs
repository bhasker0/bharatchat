use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use sovereign_core::staking::{MICROSTAKE_PER_RECIPIENT_PAISE, MAXIMUM_SPAM_TOLERANCE_RATIO, PENALTY_SUSPENSION_DURATION_SEC};

pub mod block;
pub use block::{BlockError, ConsortiumBlock, ConsortiumBlockchain, ConsortiumTx, ValidatorSignature};

#[derive(Error, Debug)]
pub enum ChainError {
    #[error("DID already registered: {0}")]
    DidAlreadyExists(String),

    #[error("DID not found: {0}")]
    DidNotFound(String),

    #[error("DID has been revoked and purged")]
    DidRevoked,

    #[error("Consent not found or has been revoked")]
    ConsentRevoked,

    #[error("Insufficient staking balance: required {required}, available {available}")]
    InsufficientStake { required: u64, available: u64 },

    #[error("Broadcast not found: {0}")]
    BroadcastNotFound(String),
}

pub type Result<T> = std::result::Result<T, ChainError>;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum DidStatus {
    Active,
    RevokedAndPurged,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct OnChainDidRecord {
    pub did: String,
    pub verifying_key_bytes: [u8; 32],
    pub status: DidStatus,
    pub registered_at: u64,
    pub revoked_at: Option<u64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum ConsentScope {
    TransactionalNotifications,
    MarketingBroadcasts,
    CommerceUpdates,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConsentRecord {
    pub principal_did: String,
    pub fiduciary_did: String,
    pub scope: ConsentScope,
    pub is_active: bool,
    pub granted_at: u64,
    pub expires_at: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StakedBroadcast {
    pub broadcast_id: String,
    pub business_did: String,
    pub recipient_count: u64,
    pub locked_stake_paise: u64,
    pub spam_reports: u64,
    pub created_at: u64,
    pub is_settled: bool,
}

#[derive(Debug, PartialEq)]
pub enum SlashingOutcome {
    Refunded { refunded_paise: u64 },
    Slashed {
        slashed_paise: u64,
        burned_paise: u64,
        integrity_pool_paise: u64,
        suspended_until: u64,
    },
}

/// Simulated permissioned consortium blockchain state machine
pub struct ConsortiumLedger {
    pub did_records: HashMap<String, OnChainDidRecord>,
    pub consent_records: HashMap<(String, String, ConsentScope), ConsentRecord>,
    pub business_stakes: HashMap<String, (u64, u64)>, // available, locked
    pub business_suspensions: HashMap<String, u64>, // DID -> suspension expiry timestamp
    pub active_broadcasts: HashMap<String, StakedBroadcast>,
    pub total_burned_paise: u64,
    pub integrity_pool_paise: u64,
}

impl ConsortiumLedger {
    pub fn new() -> Self {
        Self {
            did_records: HashMap::new(),
            consent_records: HashMap::new(),
            business_stakes: HashMap::new(),
            business_suspensions: HashMap::new(),
            active_broadcasts: HashMap::new(),
            total_burned_paise: 0,
            integrity_pool_paise: 0,
        }
    }

    // --- DID REGISTRY ---
    pub fn register_did(&mut self, did: &str, vk: [u8; 32], timestamp: u64) -> Result<()> {
        if self.did_records.contains_key(did) {
            return Err(ChainError::DidAlreadyExists(did.to_string()));
        }

        self.did_records.insert(
            did.to_string(),
            OnChainDidRecord {
                did: did.to_string(),
                verifying_key_bytes: vk,
                status: DidStatus::Active,
                registered_at: timestamp,
                revoked_at: None,
            },
        );
        Ok(())
    }

    pub fn revoke_did(&mut self, did: &str, timestamp: u64) -> Result<()> {
        let record = self.did_records.get_mut(did)
            .ok_or_else(|| ChainError::DidNotFound(did.to_string()))?;

        record.status = DidStatus::RevokedAndPurged;
        record.revoked_at = Some(timestamp);
        Ok(())
    }

    pub fn is_did_active(&self, did: &str) -> bool {
        match self.did_records.get(did) {
            Some(r) => r.status == DidStatus::Active,
            None => false,
        }
    }

    // --- DPDP CONSENT LEDGER ---
    pub fn grant_consent(
        &mut self,
        principal_did: &str,
        fiduciary_did: &str,
        scope: ConsentScope,
        granted_at: u64,
        duration_sec: u64,
    ) {
        let key = (principal_did.to_string(), fiduciary_did.to_string(), scope);
        self.consent_records.insert(
            key,
            ConsentRecord {
                principal_did: principal_did.to_string(),
                fiduciary_did: fiduciary_did.to_string(),
                scope,
                is_active: true,
                granted_at,
                expires_at: granted_at + duration_sec,
            },
        );
    }

    pub fn revoke_consent(
        &mut self,
        principal_did: &str,
        fiduciary_did: &str,
        scope: ConsentScope,
    ) {
        let key = (principal_did.to_string(), fiduciary_did.to_string(), scope);
        if let Some(record) = self.consent_records.get_mut(&key) {
            record.is_active = false;
        }
    }

    pub fn has_active_consent(
        &self,
        principal_did: &str,
        fiduciary_did: &str,
        scope: ConsentScope,
        current_time: u64,
    ) -> bool {
        let key = (principal_did.to_string(), fiduciary_did.to_string(), scope);
        match self.consent_records.get(&key) {
            Some(r) => r.is_active && r.expires_at > current_time,
            None => false,
        }
    }

    // --- BROADCAST STAKING & SLASHING POOL ---
    pub fn deposit_stake(&mut self, business_did: &str, amount_paise: u64) {
        let entry = self.business_stakes.entry(business_did.to_string()).or_insert((0, 0));
        entry.0 += amount_paise;
    }

    pub fn lock_broadcast_stake(
        &mut self,
        broadcast_id: &str,
        business_did: &str,
        recipient_count: u64,
        timestamp: u64,
    ) -> Result<()> {
        if let Some(&expiry) = self.business_suspensions.get(business_did) {
            if timestamp < expiry {
                return Err(ChainError::InsufficientStake { required: 0, available: 0 });
            }
        }

        let required = recipient_count * MICROSTAKE_PER_RECIPIENT_PAISE;
        let entry = self.business_stakes.entry(business_did.to_string()).or_insert((0, 0));

        if entry.0 < required {
            return Err(ChainError::InsufficientStake { required, available: entry.0 });
        }

        entry.0 -= required;
        entry.1 += required;

        self.active_broadcasts.insert(
            broadcast_id.to_string(),
            StakedBroadcast {
                broadcast_id: broadcast_id.to_string(),
                business_did: business_did.to_string(),
                recipient_count,
                locked_stake_paise: required,
                spam_reports: 0,
                created_at: timestamp,
                is_settled: false,
            },
        );

        Ok(())
    }

    pub fn report_broadcast_spam(&mut self, broadcast_id: &str) {
        if let Some(bcast) = self.active_broadcasts.get_mut(broadcast_id) {
            bcast.spam_reports += 1;
        }
    }

    pub fn settle_broadcast(&mut self, broadcast_id: &str, current_time: u64) -> Result<SlashingOutcome> {
        let bcast = self.active_broadcasts.get_mut(broadcast_id)
            .ok_or_else(|| ChainError::BroadcastNotFound(broadcast_id.to_string()))?;

        bcast.is_settled = true;
        let locked = bcast.locked_stake_paise;
        let business = bcast.business_did.clone();

        let stake_entry = self.business_stakes.get_mut(&business).unwrap();
        stake_entry.1 -= locked;

        let spam_ratio = if bcast.recipient_count > 0 {
            bcast.spam_reports as f64 / bcast.recipient_count as f64
        } else {
            0.0
        };

        if spam_ratio > MAXIMUM_SPAM_TOLERANCE_RATIO {
            let burned = locked / 2;
            let pool = locked - burned;

            self.total_burned_paise += burned;
            self.integrity_pool_paise += pool;

            let suspended_until = current_time + PENALTY_SUSPENSION_DURATION_SEC;
            self.business_suspensions.insert(business, suspended_until);

            Ok(SlashingOutcome::Slashed {
                slashed_paise: locked,
                burned_paise: burned,
                integrity_pool_paise: pool,
                suspended_until,
            })
        } else {
            stake_entry.0 += locked;
            Ok(SlashingOutcome::Refunded { refunded_paise: locked })
        }
    }
}
