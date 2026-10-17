use serde::{Deserialize, Serialize};
use crate::error::{SovereignError, Result};

pub const MICROSTAKE_PER_RECIPIENT_PAISE: u64 = 5; // 5 paise per recipient (0.05 INR)
pub const MAXIMUM_SPAM_TOLERANCE_RATIO: f64 = 0.03; // 3% maximum spam complaint threshold
pub const PENALTY_SUSPENSION_DURATION_SEC: u64 = 14 * 24 * 3600; // 14-day suspension

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BusinessAccount {
    pub business_did: String,
    pub available_stake_paise: u64,
    pub locked_stake_paise: u64,
    pub is_suspended: bool,
    pub suspension_until: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ActiveBroadcast {
    pub broadcast_id: String,
    pub business_did: String,
    pub recipient_count: u64,
    pub locked_stake_paise: u64,
    pub spam_reports: u64,
    pub broadcast_timestamp: u64,
    pub is_resolved: bool,
}

#[derive(Debug, PartialEq)]
pub enum BroadcastResolution {
    ApprovedAndStakeRefunded { refunded_paise: u64 },
    SlashedForSpam {
        slashed_paise: u64,
        burned_paise: u64,
        integrity_pool_paise: u64,
        suspended_until: u64,
    },
}

impl BusinessAccount {
    pub fn new(business_did: &str, initial_deposit_paise: u64) -> Self {
        Self {
            business_did: business_did.to_string(),
            available_stake_paise: initial_deposit_paise,
            locked_stake_paise: 0,
            is_suspended: false,
            suspension_until: 0,
        }
    }

    /// Locks required micro-stake for sending a broadcast to N recipients
    pub fn lock_broadcast_stake(
        &mut self,
        broadcast_id: &str,
        recipient_count: u64,
        current_timestamp: u64,
    ) -> Result<ActiveBroadcast> {
        if self.is_suspended && current_timestamp < self.suspension_until {
            return Err(SovereignError::StakeViolation(
                "Account is suspended due to prior spam violations".to_string(),
            ));
        }

        let required_stake = recipient_count * MICROSTAKE_PER_RECIPIENT_PAISE;
        if self.available_stake_paise < required_stake {
            return Err(SovereignError::StakeViolation(format!(
                "Insufficient collateral: required {} paise, available {} paise",
                required_stake, self.available_stake_paise
            )));
        }

        self.available_stake_paise -= required_stake;
        self.locked_stake_paise += required_stake;

        Ok(ActiveBroadcast {
            broadcast_id: broadcast_id.to_string(),
            business_did: self.business_did.clone(),
            recipient_count,
            locked_stake_paise: required_stake,
            spam_reports: 0,
            broadcast_timestamp: current_timestamp,
            is_resolved: false,
        })
    }

    /// Resolves broadcast after epoch evaluation based on user spam reports
    pub fn evaluate_and_resolve(
        &mut self,
        broadcast: &mut ActiveBroadcast,
        evaluation_timestamp: u64,
    ) -> BroadcastResolution {
        let spam_ratio = if broadcast.recipient_count > 0 {
            broadcast.spam_reports as f64 / broadcast.recipient_count as f64
        } else {
            0.0
        };

        broadcast.is_resolved = true;
        self.locked_stake_paise -= broadcast.locked_stake_paise;

        if spam_ratio > MAXIMUM_SPAM_TOLERANCE_RATIO {
            // VIOLATION: Execute automated slashing
            let slashed = broadcast.locked_stake_paise;
            let burned = slashed / 2;
            let integrity_pool = slashed - burned;

            self.is_suspended = true;
            self.suspension_until = evaluation_timestamp + PENALTY_SUSPENSION_DURATION_SEC;

            BroadcastResolution::SlashedForSpam {
                slashed_paise: slashed,
                burned_paise: burned,
                integrity_pool_paise: integrity_pool,
                suspended_until: self.suspension_until,
            }
        } else {
            // COMPLIANT: Refund micro-stake
            self.available_stake_paise += broadcast.locked_stake_paise;
            BroadcastResolution::ApprovedAndStakeRefunded {
                refunded_paise: broadcast.locked_stake_paise,
            }
        }
    }
}
