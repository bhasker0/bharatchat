use sovereign_broadcast::{
    AbuseCategory, AbuseDossier, AbuseReportError, BroadcastService, ForwardEnvelope, ForwardLabel,
    ForwardingError, StatusAudience,
};
use sovereign_core::{compute_canonical_cid, IdentityKeyPair};

#[test]
fn test_ephemeral_status_24h_lifecycle_and_audience_privacy() {
    let service = BroadcastService::new();
    let now = 1776000000u64;

    let alice = "did:bharat:alice_status";
    let bob = "did:bharat:bob_status";
    let charlie = "did:bharat:charlie_status";
    let eve = "did:bharat:eve_stranger";

    // Setup mutual contacts: Alice has Bob and Charlie in contacts
    service.status.add_contact(alice, bob);
    service.status.add_contact(alice, charlie);

    // 1. Alice posts status with MyContactsExcept([bob])
    let audience = StatusAudience::MyContactsExcept(vec![bob.to_string()]);
    service.status.post_status(
        "story_01",
        alice,
        "cid_status_image_01",
        "Celebrating Republic Day in New Delhi! 🇮🇳",
        now,
        audience,
    );

    // 2. Charlie (contact, not excluded) can view
    let charlie_feed = service.status.get_active_statuses(alice, charlie, now + 100);
    assert_eq!(charlie_feed.len(), 1);
    assert_eq!(charlie_feed[0].caption, "Celebrating Republic Day in New Delhi! 🇮🇳");
    assert_eq!(charlie_feed[0].views, vec![charlie.to_string()]);

    // 3. Bob (contact, but explicitly excluded) CANNOT view
    let bob_feed = service.status.get_active_statuses(alice, bob, now + 100);
    assert!(bob_feed.is_empty());

    // 4. Eve (stranger, not a contact) CANNOT view
    let eve_feed = service.status.get_active_statuses(alice, eve, now + 100);
    assert!(eve_feed.is_empty());

    // 5. Expiration after 24 hours (86,400s) -> automatically purged and crypto-shredded
    let expired_feed = service.status.get_active_statuses(alice, charlie, now + 86401);
    assert!(expired_feed.is_empty());

    let purged = service.status.purge_expired(now + 86401);
    assert_eq!(purged, 0); // Already evicted on get_active_statuses
}

#[test]
fn test_broadcast_channels_admin_provenance_and_blinded_reactions() {
    let service = BroadcastService::new();
    let now = 1776000000u64;

    let admin = IdentityKeyPair::generate();
    let subscriber1 = "did:bharat:sub_01";
    let impostor = "did:bharat:impostor";

    // 1. Admin creates public broadcast channel
    let channel = service
        .channels
        .create_channel(
            "chan_pib_india",
            "PIB India Fact Check",
            "Official Government of India Fact Checking Broadcast",
            &admin.did,
            true, // Public
            now,
        )
        .unwrap();
    assert_eq!(channel.channel_id, "chan_pib_india");
    assert_eq!(channel.subscriber_count, 1);

    // 2. Subscriber joins channel
    service.channels.subscribe("chan_pib_india", subscriber1).unwrap();

    // 3. Impostor tries to publish post -> Rejected with UnauthorizedAdmin
    let fail_post = service.channels.publish_post(
        "chan_pib_india",
        "post_01",
        impostor,
        "cid_fake_announcement",
        "Fake news announcement",
        "sig_impostor",
        now + 10,
    );
    assert!(fail_post.is_err());

    // 4. Admin publishes official verified post
    let post_text = "Beware of fraudulent SMS claiming income tax refunds. Verify via incometax.gov.in.";
    let post_cid = hex::encode(compute_canonical_cid(post_text.as_bytes()));
    let post = service
        .channels
        .publish_post(
            "chan_pib_india",
            "post_01",
            &admin.did,
            &post_cid,
            post_text,
            "sig_ed25519_verified_admin",
            now + 20,
        )
        .unwrap();
    assert_eq!(post.post_id, "post_01");

    // 5. Blinded Reactions: 2 users react with thumbs up, 1 with flag
    service.channels.add_reaction("chan_pib_india", "post_01", "👍").unwrap();
    service.channels.add_reaction("chan_pib_india", "post_01", "👍").unwrap();
    service.channels.add_reaction("chan_pib_india", "post_01", "🇮🇳").unwrap();

    let feed = service.channels.get_channel_feed("chan_pib_india", subscriber1).unwrap();
    assert_eq!(feed.len(), 1);
    assert_eq!(feed[0].reactions.get("👍"), Some(&2));
    assert_eq!(feed[0].reactions.get("🇮🇳"), Some(&1));
}

#[test]
fn test_forwarding_limits_and_viral_spread_friction() {
    let sender = "did:bharat:original_author";
    let cid = "cid_provocative_rumor_001";

    // Hop 0: Original direct message
    let hop0 = ForwardEnvelope::new_direct(cid, sender);
    assert_eq!(hop0.label, ForwardLabel::None);
    assert_eq!(hop0.max_allowed_targets, 5);
    assert!(hop0.validate_forward(&vec!["chat1".into(), "chat2".into(), "chat3".into()]).is_ok());

    // Hop 1-4: Standard forward
    let hop1 = hop0.forward_to_next();
    assert_eq!(hop1.label, ForwardLabel::Forwarded);
    assert_eq!(hop1.hop_count, 1);
    assert_eq!(hop1.max_allowed_targets, 5);

    let hop4 = hop1.forward_to_next().forward_to_next().forward_to_next();
    assert_eq!(hop4.hop_count, 4);
    assert_eq!(hop4.label, ForwardLabel::Forwarded);

    // Hop 5: Escalation to "Forwarded many times"
    let hop5 = hop4.forward_to_next();
    assert_eq!(hop5.hop_count, 5);
    assert_eq!(hop5.label, ForwardLabel::ForwardedManyTimes);
    assert_eq!(hop5.max_allowed_targets, 1);

    // Forwarding to 2 or more chats is BLOCKED by viral friction rule
    let multi_target = vec!["chat_grp_1".to_string(), "chat_grp_2".to_string()];
    let err = hop5.validate_forward(&multi_target).unwrap_err();
    assert_eq!(err, ForwardingError::FrequentlyForwardedLimitExceeded);

    // Forwarding strictly to 1 chat is permitted
    let single_target = vec!["chat_grp_1".to_string()];
    assert!(hop5.validate_forward(&single_target).is_ok());
}

#[test]
fn test_cryptographic_abuse_report_and_it_rules_grievance() {
    let service = BroadcastService::new();
    let now = 1776000000u64;

    let sender = IdentityKeyPair::generate();
    let reporter = "did:bharat:victim_bob";

    let abusive_content = "Extremist hate speech inciting violence in public market.";
    let message_cid = hex::encode(compute_canonical_cid(abusive_content.as_bytes()));

    // Sender signs message: message_cid || timestamp
    let mut sign_payload = Vec::new();
    sign_payload.extend_from_slice(message_cid.as_bytes());
    sign_payload.extend_from_slice(&now.to_be_bytes());

    let signature_bytes = sender.sign(&sign_payload);
    let signature_hex = hex::encode(signature_bytes);

    // 1. Valid abuse report submission
    let dossier = AbuseDossier {
        dossier_id: "DOSSIER-2026-001".to_string(),
        reported_message_cid: message_cid.clone(),
        sender_did: sender.did.clone(),
        sender_signature_hex: signature_hex.clone(),
        plaintext: abusive_content.to_string(),
        reporter_did: reporter.to_string(),
        category: AbuseCategory::HateSpeech,
        timestamp: now,
        jurisdiction: "INDIA_LOCALIZED_MUMBAI_BLR".to_string(),
    };

    let vk_bytes = sender.document().verifying_key_bytes;
    let ticket = service.abuse.submit_report(dossier, vk_bytes).unwrap();

    assert_eq!(ticket.ticket_id, "GRV-IN-DOSSIER-2026-001");
    assert_eq!(ticket.status, sovereign_broadcast::GrievanceStatus::Acknowledged);
    assert_eq!(ticket.resolution_deadline, now + (15 * 86400)); // 15-day statutory deadline

    // 2. Tampered content rejection (HashMismatch)
    let tampered_dossier = AbuseDossier {
        dossier_id: "DOSSIER-FAKE".to_string(),
        reported_message_cid: message_cid.clone(),
        sender_did: sender.did.clone(),
        sender_signature_hex: signature_hex,
        plaintext: "Tampered content that does not match original CID".to_string(),
        reporter_did: reporter.to_string(),
        category: AbuseCategory::HateSpeech,
        timestamp: now,
        jurisdiction: "INDIA_LOCALIZED_MUMBAI_BLR".to_string(),
    };
    let err = service.abuse.submit_report(tampered_dossier, vk_bytes).unwrap_err();
    match err {
        AbuseReportError::HashMismatch => {}
        _ => panic!("Expected HashMismatch error"),
    }
}
