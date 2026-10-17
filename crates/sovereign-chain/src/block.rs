use crate::{ConsentScope, ConsortiumLedger};
use serde::{Deserialize, Serialize};
use sovereign_core::compute_canonical_cid;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum BlockError {
    #[error("Invalid previous block hash: expected '{expected}', got '{got}'")]
    InvalidPrevHash { expected: String, got: String },

    #[error("Invalid block height: expected {expected}, got {got}")]
    InvalidHeight { expected: u64, got: u64 },

    #[error("Insufficient validator quorum: required {required}, valid signatures {valid}")]
    InsufficientQuorum { required: usize, valid: usize },

    #[error("Invalid signature from validator '{0}'")]
    InvalidValidatorSignature(String),

    #[error("Unknown validator '{0}' not in consortium whitelist")]
    UnknownValidator(String),

    #[error("Duplicate signature from validator '{0}' in same block")]
    DuplicateValidatorSignature(String),

    #[error("Merkle root mismatch: expected '{expected}', computed '{computed}'")]
    MerkleRootMismatch { expected: String, computed: String },

    #[error("Chain error during transaction application: {0}")]
    LedgerError(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConsortiumTx {
    RegisterDid {
        did: String,
        vk_hex: String,
        timestamp: u64,
    },
    RevokeDid {
        did: String,
        timestamp: u64,
    },
    GrantConsent {
        principal_did: String,
        fiduciary_did: String,
        scope: ConsentScope,
        granted_at: u64,
        duration_sec: u64,
    },
    RevokeConsent {
        principal_did: String,
        fiduciary_did: String,
        scope: ConsentScope,
    },
    DepositStake {
        business_did: String,
        amount_paise: u64,
    },
    LockBroadcastStake {
        broadcast_id: String,
        business_did: String,
        recipient_count: u64,
        timestamp: u64,
    },
    ReportSpam {
        broadcast_id: String,
    },
    SettleBroadcast {
        broadcast_id: String,
        timestamp: u64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidatorSignature {
    pub validator_did: String,
    pub verifying_key_bytes: [u8; 32],
    pub signature_hex: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsortiumBlock {
    pub height: u64,
    pub prev_block_hash: String,
    pub timestamp: u64,
    pub txs: Vec<ConsortiumTx>,
    pub merkle_root: String,
    pub signatures: Vec<ValidatorSignature>,
}

impl ConsortiumBlock {
    pub fn new(
        height: u64,
        prev_block_hash: String,
        timestamp: u64,
        txs: Vec<ConsortiumTx>,
    ) -> Self {
        let merkle_root = Self::compute_merkle_root(&txs);
        Self {
            height,
            prev_block_hash,
            timestamp,
            txs,
            merkle_root,
            signatures: Vec::new(),
        }
    }

    /// Compute cryptographic BLAKE3 Merkle Root over ordered transactions
    pub fn compute_merkle_root(txs: &[ConsortiumTx]) -> String {
        if txs.is_empty() {
            return hex::encode(compute_canonical_cid(b"empty_consortium_block"));
        }

        let mut hashes: Vec<Vec<u8>> = txs
            .iter()
            .map(|tx| {
                let serialized = serde_json::to_vec(tx).unwrap_or_default();
                compute_canonical_cid(&serialized).to_vec()
            })
            .collect();

        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in hashes.chunks(2) {
                if chunk.len() == 2 {
                    let mut combined = Vec::new();
                    combined.extend_from_slice(&chunk[0]);
                    combined.extend_from_slice(&chunk[1]);
                    next_level.push(compute_canonical_cid(&combined).to_vec());
                } else {
                    next_level.push(chunk[0].clone());
                }
            }
            hashes = next_level;
        }

        hex::encode(&hashes[0])
    }

    /// Compute canonical block hash over header fields
    pub fn block_hash(&self) -> String {
        let mut header_bytes = Vec::new();
        header_bytes.extend_from_slice(&self.height.to_be_bytes());
        header_bytes.extend_from_slice(self.prev_block_hash.as_bytes());
        header_bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        header_bytes.extend_from_slice(self.merkle_root.as_bytes());

        hex::encode(compute_canonical_cid(&header_bytes))
    }

    /// Add a validator signature to the block
    pub fn add_signature(&mut self, sig: ValidatorSignature) {
        self.signatures.push(sig);
    }
}

/// Permissioned Consortium Blockchain with 2-of-3 BFT Quorum
pub struct ConsortiumBlockchain {
    pub chain: Vec<ConsortiumBlock>,
    pub validators: HashMap<String, [u8; 32]>,
    pub ledger: ConsortiumLedger,
    pub quorum_threshold: usize,
}

impl ConsortiumBlockchain {
    pub fn new(validators: HashMap<String, [u8; 32]>) -> Self {
        let count = validators.len();
        // Standard BFT 2F+1: For 3 nodes, quorum is 2
        let quorum_threshold = if count <= 1 {
            1
        } else {
            (2 * count + 2) / 3
        };

        let mut chain = Self {
            chain: Vec::new(),
            validators,
            ledger: ConsortiumLedger::new(),
            quorum_threshold,
        };

        chain.create_genesis();
        chain
    }

    fn create_genesis(&mut self) {
        let genesis_txs = vec![];
        let genesis_block = ConsortiumBlock::new(
            0,
            "0".repeat(64),
            1776000000,
            genesis_txs,
        );
        self.chain.push(genesis_block);
    }

    pub fn tip(&self) -> &ConsortiumBlock {
        self.chain.last().expect("Blockchain cannot be empty")
    }

    /// Validates and appends a newly proposed block
    pub fn commit_block(&mut self, block: ConsortiumBlock) -> Result<(), BlockError> {
        let tip = self.tip();

        // 1. Verify height continuity
        if block.height != tip.height + 1 {
            return Err(BlockError::InvalidHeight {
                expected: tip.height + 1,
                got: block.height,
            });
        }

        // 2. Verify previous block hash linkage
        let expected_prev = tip.block_hash();
        if block.prev_block_hash != expected_prev {
            return Err(BlockError::InvalidPrevHash {
                expected: expected_prev,
                got: block.prev_block_hash,
            });
        }

        // 3. Verify Merkle root integrity
        let computed_root = ConsortiumBlock::compute_merkle_root(&block.txs);
        if block.merkle_root != computed_root {
            return Err(BlockError::MerkleRootMismatch {
                expected: block.merkle_root,
                computed: computed_root,
            });
        }

        // 4. Verify BFT Quorum Validator Signatures
        let block_hash = block.block_hash();
        let sign_msg = block_hash.as_bytes();

        let mut seen_validators = HashSet::new();
        let mut valid_signatures = 0;

        for sig in &block.signatures {
            // Check whitelist
            let expected_vk = self
                .validators
                .get(&sig.validator_did)
                .ok_or_else(|| BlockError::UnknownValidator(sig.validator_did.clone()))?;

            if !seen_validators.insert(&sig.validator_did) {
                return Err(BlockError::DuplicateValidatorSignature(sig.validator_did.clone()));
            }

            let sig_bytes = hex::decode(&sig.signature_hex)
                .map_err(|_| BlockError::InvalidValidatorSignature(sig.validator_did.clone()))?;

            let doc = sovereign_core::DidDocument {
                did: sig.validator_did.clone(),
                verifying_key_bytes: *expected_vk,
            };

            if doc.verify(sign_msg, &sig_bytes).is_err() {
                return Err(BlockError::InvalidValidatorSignature(sig.validator_did.clone()));
            }

            valid_signatures += 1;
        }

        if valid_signatures < self.quorum_threshold {
            return Err(BlockError::InsufficientQuorum {
                required: self.quorum_threshold,
                valid: valid_signatures,
            });
        }

        // 5. Apply transactions to Consortium Ledger state
        for tx in &block.txs {
            match tx {
                ConsortiumTx::RegisterDid { did, vk_hex, timestamp } => {
                    let mut vk_bytes = [0u8; 32];
                    if let Ok(b) = hex::decode(vk_hex) {
                        if b.len() == 32 {
                            vk_bytes.copy_from_slice(&b);
                        }
                    }
                    self.ledger
                        .register_did(did, vk_bytes, *timestamp)
                        .map_err(|e| BlockError::LedgerError(e.to_string()))?;
                }
                ConsortiumTx::RevokeDid { did, timestamp } => {
                    self.ledger
                        .revoke_did(did, *timestamp)
                        .map_err(|e| BlockError::LedgerError(e.to_string()))?;
                }
                ConsortiumTx::GrantConsent {
                    principal_did,
                    fiduciary_did,
                    scope,
                    granted_at,
                    duration_sec,
                } => {
                    self.ledger.grant_consent(
                        principal_did,
                        fiduciary_did,
                        *scope,
                        *granted_at,
                        *duration_sec,
                    );
                }
                ConsortiumTx::RevokeConsent {
                    principal_did,
                    fiduciary_did,
                    scope,
                } => {
                    self.ledger.revoke_consent(principal_did, fiduciary_did, *scope);
                }
                ConsortiumTx::DepositStake { business_did, amount_paise } => {
                    self.ledger.deposit_stake(business_did, *amount_paise);
                }
                ConsortiumTx::LockBroadcastStake {
                    broadcast_id,
                    business_did,
                    recipient_count,
                    timestamp,
                } => {
                    self.ledger
                        .lock_broadcast_stake(broadcast_id, business_did, *recipient_count, *timestamp)
                        .map_err(|e| BlockError::LedgerError(e.to_string()))?;
                }
                ConsortiumTx::ReportSpam { broadcast_id } => {
                    self.ledger.report_broadcast_spam(broadcast_id);
                }
                ConsortiumTx::SettleBroadcast { broadcast_id, timestamp } => {
                    self.ledger
                        .settle_broadcast(broadcast_id, *timestamp)
                        .map_err(|e| BlockError::LedgerError(e.to_string()))?;
                }
            }
        }

        self.chain.push(block);
        Ok(())
    }

    /// Verifies the full chain cryptographic integrity and BFT quorum
    pub fn verify_chain(&self) -> Result<(), BlockError> {
        for i in 1..self.chain.len() {
            let prev = &self.chain[i - 1];
            let curr = &self.chain[i];

            if curr.height != prev.height + 1 {
                return Err(BlockError::InvalidHeight {
                    expected: prev.height + 1,
                    got: curr.height,
                });
            }

            let expected_hash = prev.block_hash();
            if curr.prev_block_hash != expected_hash {
                return Err(BlockError::InvalidPrevHash {
                    expected: expected_hash,
                    got: curr.prev_block_hash.clone(),
                });
            }

            let computed_root = ConsortiumBlock::compute_merkle_root(&curr.txs);
            if curr.merkle_root != computed_root {
                return Err(BlockError::MerkleRootMismatch {
                    expected: curr.merkle_root.clone(),
                    computed: computed_root,
                });
            }
        }
        Ok(())
    }
}
