use sovereign_chain::{ConsortiumLedger, ConsentScope, DidStatus, SlashingOutcome};

#[test]
fn test_consortium_did_and_dpdp_consent_lifecycle() {
    let mut ledger = ConsortiumLedger::new();
    let user_did = "did:bharat:citizen_vikram_singh";
    let business_did = "did:bharat:enterprise_flipkart_rewards";
    let vk = [1u8; 32];
    let now = 1729185000;

    // 1. Register DID
    ledger.register_did(user_did, vk, now).expect("DID registration should succeed");
    assert!(ledger.is_did_active(user_did));

    // 2. Grant explicit DPDP marketing consent for 30 days
    ledger.grant_consent(user_did, business_did, ConsentScope::MarketingBroadcasts, now, 30 * 86400);
    assert!(ledger.has_active_consent(user_did, business_did, ConsentScope::MarketingBroadcasts, now + 100));

    // 3. User revokes consent -> immediately inactive
    ledger.revoke_consent(user_did, business_did, ConsentScope::MarketingBroadcasts);
    assert!(!ledger.has_active_consent(user_did, business_did, ConsentScope::MarketingBroadcasts, now + 100));

    // 4. DPDP Account Erasure -> DID revoked & purged
    ledger.revoke_did(user_did, now + 200).expect("Revocation should succeed");
    assert!(!ledger.is_did_active(user_did));
    assert_eq!(ledger.did_records.get(user_did).unwrap().status, DidStatus::RevokedAndPurged);
}

#[test]
fn test_onchain_broadcast_staking_and_automated_slashing() {
    let mut ledger = ConsortiumLedger::new();
    let business_did = "did:bharat:retail_promo_corp";
    let now = 1729185000;

    // 1. Business deposits ₹500 (50,000 paise)
    ledger.deposit_stake(business_did, 50_000);

    // 2. Lock micro-stake for 5,000 users (5,000 * 5 paise = 25,000 paise)
    ledger.lock_broadcast_stake("bcast_chain_1", business_did, 5_000, now)
        .expect("Stake lock should succeed");

    assert_eq!(ledger.business_stakes.get(business_did).unwrap(), &(25_000, 25_000));

    // 3. High spam reports (400 reports / 5000 recipients = 8% > 3%)
    for _ in 0..400 {
        ledger.report_broadcast_spam("bcast_chain_1");
    }

    // 4. Settle broadcast: triggers on-chain slashing
    let outcome = ledger.settle_broadcast("bcast_chain_1", now + 86400)
        .expect("Settlement should succeed");

    match outcome {
        SlashingOutcome::Slashed { slashed_paise, burned_paise, integrity_pool_paise, .. } => {
            assert_eq!(slashed_paise, 25_000);
            assert_eq!(burned_paise, 12_500);
            assert_eq!(integrity_pool_paise, 12_500);
        }
        _ => panic!("Expected broadcast to be slashed!"),
    }

    assert_eq!(ledger.total_burned_paise, 12_500);
    assert_eq!(ledger.integrity_pool_paise, 12_500);
}

#[test]
fn test_consortium_blockchain_bft_quorum_and_block_commits() {
    use sovereign_core::IdentityKeyPair;
    use sovereign_chain::{ConsortiumBlock, ConsortiumBlockchain, ConsortiumTx, ValidatorSignature};
    use std::collections::HashMap;

    // 1. Setup 3 Consortium Validator Nodes (MeitY, UIDAI, RBI)
    let val_meity = IdentityKeyPair::generate();
    let val_uidai = IdentityKeyPair::generate();
    let val_rbi = IdentityKeyPair::generate();

    let mut validators = HashMap::new();
    validators.insert(val_meity.did.clone(), val_meity.document().verifying_key_bytes);
    validators.insert(val_uidai.did.clone(), val_uidai.document().verifying_key_bytes);
    validators.insert(val_rbi.did.clone(), val_rbi.document().verifying_key_bytes);

    let mut blockchain = ConsortiumBlockchain::new(validators);
    assert_eq!(blockchain.chain.len(), 1); // Genesis block exists
    assert_eq!(blockchain.quorum_threshold, 2); // 2-of-3 BFT quorum

    // 2. Prepare Block 1 with Transactions
    let citizen_key = IdentityKeyPair::generate();
    let now = 1776000100;

    let txs = vec![
        ConsortiumTx::RegisterDid {
            did: citizen_key.did.clone(),
            vk_hex: hex::encode(citizen_key.document().verifying_key_bytes),
            timestamp: now,
        },
        ConsortiumTx::GrantConsent {
            principal_did: citizen_key.did.clone(),
            fiduciary_did: "did:bharat:varanasi_silk".to_string(),
            scope: ConsentScope::CommerceUpdates,
            granted_at: now,
            duration_sec: 86400 * 30,
        },
    ];

    let mut block1 = ConsortiumBlock::new(1, blockchain.tip().block_hash(), now, txs);
    let block1_hash = block1.block_hash();

    // 3. Collect 2 Validator Signatures (MeitY + RBI meets 2-of-3 threshold)
    let sig_meity = val_meity.sign(block1_hash.as_bytes());
    let sig_rbi = val_rbi.sign(block1_hash.as_bytes());

    block1.add_signature(ValidatorSignature {
        validator_did: val_meity.did.clone(),
        verifying_key_bytes: val_meity.document().verifying_key_bytes,
        signature_hex: hex::encode(sig_meity),
    });
    block1.add_signature(ValidatorSignature {
        validator_did: val_rbi.did.clone(),
        verifying_key_bytes: val_rbi.document().verifying_key_bytes,
        signature_hex: hex::encode(sig_rbi),
    });

    // 4. Commit Block 1
    blockchain.commit_block(block1).expect("Block 1 with 2-of-3 BFT quorum should commit cleanly");
    assert_eq!(blockchain.chain.len(), 2);
    assert_eq!(blockchain.tip().height, 1);

    // 5. Verify State was Applied to Consortium Ledger
    assert!(blockchain.ledger.is_did_active(&citizen_key.did));
    assert!(blockchain.ledger.has_active_consent(
        &citizen_key.did,
        "did:bharat:varanasi_silk",
        ConsentScope::CommerceUpdates,
        now + 500
    ));

    // 6. Verify Full Blockchain Cryptographic Integrity
    blockchain.verify_chain().expect("Full chain verification should pass");
}

#[test]
fn test_consortium_blockchain_insufficient_quorum_and_tamper_rejection() {
    use sovereign_core::IdentityKeyPair;
    use sovereign_chain::{BlockError, ConsortiumBlock, ConsortiumBlockchain, ValidatorSignature};
    use std::collections::HashMap;

    let val1 = IdentityKeyPair::generate();
    let val2 = IdentityKeyPair::generate();
    let val3 = IdentityKeyPair::generate();

    let mut validators = HashMap::new();
    validators.insert(val1.did.clone(), val1.document().verifying_key_bytes);
    validators.insert(val2.did.clone(), val2.document().verifying_key_bytes);
    validators.insert(val3.did.clone(), val3.document().verifying_key_bytes);

    let mut blockchain = ConsortiumBlockchain::new(validators);

    // Reject 1: Insufficient quorum (only 1 signature provided out of required 2)
    let now = 1776000200;
    let mut block_low_quorum = ConsortiumBlock::new(1, blockchain.tip().block_hash(), now, vec![]);
    let hash = block_low_quorum.block_hash();
    let sig1 = val1.sign(hash.as_bytes());
    block_low_quorum.add_signature(ValidatorSignature {
        validator_did: val1.did.clone(),
        verifying_key_bytes: val1.document().verifying_key_bytes,
        signature_hex: hex::encode(sig1),
    });

    let res = blockchain.commit_block(block_low_quorum);
    assert!(matches!(res, Err(BlockError::InsufficientQuorum { required: 2, valid: 1 })));

    // Reject 2: Tampered previous block hash
    let mut block_bad_prev = ConsortiumBlock::new(1, "bad_hash_ffff".to_string(), now, vec![]);
    let hash_bad = block_bad_prev.block_hash();
    let sig1_b = val1.sign(hash_bad.as_bytes());
    let sig2_b = val2.sign(hash_bad.as_bytes());
    block_bad_prev.add_signature(ValidatorSignature {
        validator_did: val1.did.clone(),
        verifying_key_bytes: val1.document().verifying_key_bytes,
        signature_hex: hex::encode(sig1_b),
    });
    block_bad_prev.add_signature(ValidatorSignature {
        validator_did: val2.did.clone(),
        verifying_key_bytes: val2.document().verifying_key_bytes,
        signature_hex: hex::encode(sig2_b),
    });

    let res = blockchain.commit_block(block_bad_prev);
    assert!(matches!(res, Err(BlockError::InvalidPrevHash { .. })));

    // Reject 3: Tampered Merkle root
    let mut block_tampered_merkle = ConsortiumBlock::new(1, blockchain.tip().block_hash(), now, vec![]);
    block_tampered_merkle.merkle_root = "tampered_root_0000".to_string();
    let res = blockchain.commit_block(block_tampered_merkle);
    assert!(matches!(res, Err(BlockError::MerkleRootMismatch { .. })));
}
