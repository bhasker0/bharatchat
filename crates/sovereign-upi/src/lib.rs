use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpiError {
    #[error("Device SIM binding token invalid or mismatch")]
    SimBindingMismatch,

    #[error("Transaction is already finalized")]
    TransactionFinalized,

    #[error("Amount must be greater than zero")]
    InvalidAmount,
}

pub type Result<T> = std::result::Result<T, UpiError>;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SimBindingToken {
    pub device_hardware_hash: String,
    pub sim_iccid_hash: String,
    pub token: String,
    pub bound_at: u64,
}

impl SimBindingToken {
    /// Binds physical SIM card and hardware enclave token
    pub fn create(device_id: &str, sim_iccid: &str, timestamp: u64) -> Self {
        let dev_hash = hex::encode(blake3::hash(device_id.as_bytes()).as_bytes());
        let sim_hash = hex::encode(blake3::hash(sim_iccid.as_bytes()).as_bytes());

        let mut token_input = Vec::new();
        token_input.extend_from_slice(dev_hash.as_bytes());
        token_input.extend_from_slice(sim_hash.as_bytes());
        token_input.extend_from_slice(&timestamp.to_be_bytes());
        let token = hex::encode(blake3::hash(&token_input).as_bytes());

        Self {
            device_hardware_hash: dev_hash,
            sim_iccid_hash: sim_hash,
            token,
            bound_at: timestamp,
        }
    }

    /// Verifies current hardware against registered SIM binding
    pub fn verify(&self, current_device_id: &str, current_sim_iccid: &str) -> bool {
        let cur_dev = hex::encode(blake3::hash(current_device_id.as_bytes()).as_bytes());
        let cur_sim = hex::encode(blake3::hash(current_sim_iccid.as_bytes()).as_bytes());

        self.device_hardware_hash == cur_dev && self.sim_iccid_hash == cur_sim
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum PaymentStatus {
    Initiated,
    PendingBankSwitch,
    Success,
    Failed,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpiPaymentOrder {
    pub transaction_id: String,
    pub reference_id: String,
    pub payee_vpa: String,
    pub payee_name: String,
    pub amount_inr: f64,
    pub currency: String,
    pub note: String,
    pub status: PaymentStatus,
    pub initiated_at: u64,
    pub finalized_at: Option<u64>,
    pub data_jurisdiction: String, // "INDIA_LOCALIZED" as per RBI Directive
}

impl UpiPaymentOrder {
    /// Scaffolds a new UPI 2.0 Payment Order conforming to NPCI Intent standards
    pub fn new(
        payee_vpa: &str,
        payee_name: &str,
        amount_inr: f64,
        note: &str,
        timestamp: u64,
    ) -> Result<Self> {
        if amount_inr <= 0.0 {
            return Err(UpiError::InvalidAmount);
        }

        let txn_id = format!("BHARAT{}", timestamp);
        let ref_id = format!("REF{}", timestamp);

        Ok(Self {
            transaction_id: txn_id,
            reference_id: ref_id,
            payee_vpa: payee_vpa.to_string(),
            payee_name: payee_name.to_string(),
            amount_inr,
            currency: "INR".to_string(),
            note: note.to_string(),
            status: PaymentStatus::Initiated,
            initiated_at: timestamp,
            finalized_at: None,
            data_jurisdiction: "INDIA_LOCALIZED_MUMBAI_BLR".to_string(),
        })
    }

    /// Generates standard NPCI UPI Intent & QR URI (`upi://pay`)
    pub fn generate_upi_uri(&self) -> String {
        format!(
            "upi://pay?pa={}&pn={}&tr={}&tid={}&am={:.2}&cu=INR&tn={}",
            self.payee_vpa,
            url_encode(&self.payee_name),
            self.reference_id,
            self.transaction_id,
            self.amount_inr,
            url_encode(&self.note)
        )
    }

    /// Transitions status to PendingBankSwitch
    pub fn dispatch_to_bank_switch(&mut self) -> Result<()> {
        if self.status != PaymentStatus::Initiated {
            return Err(UpiError::TransactionFinalized);
        }
        self.status = PaymentStatus::PendingBankSwitch;
        Ok(())
    }

    /// Finalizes payment outcome (Success or Failed)
    pub fn finalize(&mut self, is_success: bool, timestamp: u64) -> Result<()> {
        if self.status == PaymentStatus::Success || self.status == PaymentStatus::Failed {
            return Err(UpiError::TransactionFinalized);
        }

        self.status = if is_success { PaymentStatus::Success } else { PaymentStatus::Failed };
        self.finalized_at = Some(timestamp);
        Ok(())
    }
}

fn url_encode(input: &str) -> String {
    input.replace(' ', "%20")
}
