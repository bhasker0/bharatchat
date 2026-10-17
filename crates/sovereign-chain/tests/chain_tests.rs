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
