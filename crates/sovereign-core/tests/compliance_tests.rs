use sovereign_core::{
    IdentityKeyPair,
    compute_canonical_cid,
    TraceabilityEnvelope,
    forward_message,
    StorageMasterKey,
    BlindBloomFilter,
    ClientScanner,
    compute_perceptual_hash,
    BusinessAccount,
    BroadcastResolution,
    SovereignError,
};
use x25519_dalek::{StaticSecret, PublicKey as X25519PublicKey};
use rand::rngs::OsRng;

#[test]
fn test_rule_4_2_first_originator_traceability_across_forwards() {
    // 1. Setup Legal Multi-Sig Threshold Keypair (In production, derived via 3-of-4 MPC)
    let legal_secret = StaticSecret::random_from_rng(OsRng);
    let legal_public = X25519PublicKey::from(&legal_secret);

    // 2. Alice creates an original message
    let alice = IdentityKeyPair::generate();
    let original_message = b"Breaking news: Verified sovereign communication announcement.";
    let origination_time = 1729180800;

    let envelope = TraceabilityEnvelope::create(
        &alice,
        original_message,
        &legal_public,
        origination_time,
    ).expect("Traceability envelope creation must succeed");

    assert_eq!(envelope.content_cid, compute_canonical_cid(original_message));

    // 3. Bob receives the message and forwards to Charlie
    let bob = IdentityKeyPair::generate();
    let forward_time_1 = origination_time + 60;
    let hop_1 = forward_message(&bob, &envelope, original_message, forward_time_1)
        .expect("Forwarding identical content must succeed");

    assert_eq!(hop_1.forwarder_did, bob.did);

    // 4. Charlie forwards to Dave
    let charlie = IdentityKeyPair::generate();
    let forward_time_2 = forward_time_1 + 120;
    let hop_2 = forward_message(&charlie, &envelope, original_message, forward_time_2)
        .expect("Subsequent forwarding must succeed");

    assert_eq!(hop_2.forwarder_did, charlie.did);

    // 5. Test content alteration tampering: Attempting to forward modified content with original envelope fails!
    let tampered_message = b"Fake news: Altered malicious text!";
    let tamper_result = forward_message(&charlie, &envelope, tampered_message, forward_time_2);
    assert!(matches!(tamper_result, Err(SovereignError::ContentHashMismatch)));

    // 6. Judicial Execution: Court issues warrant for flagged content
    // Threshold decryption recovers Alice as the first originator
    let alice_doc = alice.document();
    let recovered_originator = envelope.threshold_decrypt(&legal_secret, &alice_doc)
        .expect("Judicial threshold decryption must succeed");

    assert_eq!(recovered_originator.originator_did, alice.did);
    assert_eq!(recovered_originator.content_cid, compute_canonical_cid(original_message));
    assert_eq!(recovered_originator.timestamp, origination_time);
}

#[test]
fn test_dpdp_act_section_12_crypto_shredding() {
    let alice = IdentityKeyPair::generate();
    let mut smk = StorageMasterKey::generate(&alice.did);

    let chat_backup_payload = b"Confidential financial transaction history and personal chats.";

    // 1. Normal encrypted storage
    let (ciphertext, nonce) = smk.encrypt_payload(chat_backup_payload)
        .expect("Payload encryption should succeed");

    // 2. Successful retrieval prior to deletion
    let decrypted = smk.decrypt_payload(&ciphertext, &nonce)
        .expect("Payload decryption should succeed before shredding");
    assert_eq!(decrypted, chat_backup_payload);

    // 3. User requests Account Deletion under DPDP Act Section 12 -> Execute Crypto-Shredding
    smk.crypto_shred();
    assert!(smk.is_shredded());

    // 4. Verify that data is now mathematically unrecoverable
    let fail_result = smk.decrypt_payload(&ciphertext, &nonce);
    assert!(matches!(fail_result, Err(SovereignError::KeyShredded)));

    // 5. Verify that new encryptions are permanently rejected
    let new_enc = smk.encrypt_payload(b"New data");
    assert!(matches!(new_enc, Err(SovereignError::KeyShredded)));
}

#[test]
fn test_client_side_csam_prevention_scanner() {
    let mut bloom_filter = BlindBloomFilter::new(1024);

    let illicit_media = b"KNOWN_ILLICIT_MEDIA_SAMPLE_FOR_TESTING";
    let illicit_hash = compute_perceptual_hash(illicit_media);
    bloom_filter.insert(illicit_hash);

    let scanner = ClientScanner::new(bloom_filter);
    let now = 1729181000;

    // 1. Clean media passes inspection
    let clean_media = b"Clean family holiday photograph in Goa";
    let clean_result = scanner.inspect_and_gate(clean_media, now);
    assert!(clean_result.is_ok(), "Clean media must be allowed to proceed to encryption");

    // 2. Illicit media is detected and transmission is aborted
    let blocked_result = scanner.inspect_and_gate(illicit_media, now);
    assert!(matches!(blocked_result, Err(SovereignError::CsamDetected { .. })));

    // 3. Compliance alert is generated for law enforcement reporting (Rule 3(1)(b))
    let alert = scanner.generate_compliance_alert(illicit_media, now);
    assert_eq!(alert.matched_hash, illicit_hash);
    assert_eq!(alert.statutory_code, "POCSO_ACT_2012_AND_IT_RULES_2021_R3(1)(b)");
}

#[test]
fn test_business_broadcast_micro_staking_and_slashing() {
    let business_did = "did:bharat:business_retail_corp_99";
    // Deposit ₹1,000 (100,000 paise)
    let mut account = BusinessAccount::new(business_did, 100_000);

    let current_time = 1729182000;
    let recipient_count = 10_000; // 10,000 users @ 5 paise = 50,000 paise (₹500)

    // 1. Lock micro-stake for broadcast
    let mut broadcast = account.lock_broadcast_stake("bcast_001", recipient_count, current_time)
        .expect("Locking micro-stake must succeed");

    assert_eq!(account.available_stake_paise, 50_000);
    assert_eq!(account.locked_stake_paise, 50_000);
    assert_eq!(broadcast.locked_stake_paise, 50_000);

    // Scenario A: Compliant broadcast (only 50 complaints out of 10,000 = 0.5% < 3%)
    broadcast.spam_reports = 50;
    let resolution = account.evaluate_and_resolve(&mut broadcast, current_time + 86400);

    assert_eq!(resolution, BroadcastResolution::ApprovedAndStakeRefunded { refunded_paise: 50_000 });
    assert_eq!(account.available_stake_paise, 100_000);
    assert_eq!(account.locked_stake_paise, 0);
    assert!(!account.is_suspended);

    // Scenario B: Spam violation (500 complaints out of 10,000 = 5.0% > 3%)
    let mut spam_broadcast = account.lock_broadcast_stake("bcast_002", recipient_count, current_time + 100_000)
        .expect("Locking micro-stake must succeed");

    spam_broadcast.spam_reports = 500;
    let slash_resolution = account.evaluate_and_resolve(&mut spam_broadcast, current_time + 186_400);

    match slash_resolution {
        BroadcastResolution::SlashedForSpam { slashed_paise, burned_paise, integrity_pool_paise, .. } => {
            assert_eq!(slashed_paise, 50_000);
            assert_eq!(burned_paise, 25_000);
            assert_eq!(integrity_pool_paise, 25_000);
        }
        _ => panic!("Expected broadcast to be slashed!"),
    }

    assert!(account.is_suspended);
    assert_eq!(account.available_stake_paise, 50_000); // 50,000 lost to slashing

    // Subsequent broadcast attempts while suspended must be blocked
    let blocked_attempt = account.lock_broadcast_stake("bcast_003", 100, current_time + 186_500);
    assert!(matches!(blocked_attempt, Err(SovereignError::StakeViolation(_))));
}
