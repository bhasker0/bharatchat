use sovereign_core::{IdentityKeyPair, TraceabilityEnvelope, MessagePacket};
use sovereign_relay::RelayRouter;
use x25519_dalek::{StaticSecret, PublicKey as X25519PublicKey};
use rand::rngs::OsRng;

#[tokio::test]
async fn test_ephemeral_relay_live_and_offline_routing() {
    let router = RelayRouter::new(3600); // 1 hour TTL
    let legal_secret = StaticSecret::random_from_rng(OsRng);
    let legal_pub = X25519PublicKey::from(&legal_secret);

    let alice = IdentityKeyPair::generate();
    let bob = IdentityKeyPair::generate();

    let message_text = b"Confidential relay routing test payload";
    let envelope = TraceabilityEnvelope::create(&alice, message_text, &legal_pub, 1729183000)
        .expect("Envelope creation must succeed");

    let packet = MessagePacket {
        sender_did: alice.did.clone(),
        recipient_did: bob.did.clone(),
        e2ee_ciphertext: vec![1, 2, 3, 4, 5],
        trace_envelope: envelope.clone(),
        forward_chain: vec![],
        packet_signature: vec![0; 64],
    };

    // 1. Bob is offline: packet is staged in ephemeral memory
    assert!(!router.is_online(&bob.did).await);
    let delivered_live = router.route_packet(packet.clone(), 1729183000).await
        .expect("Routing to offline user should stage successfully");
    assert!(!delivered_live, "Should report staged offline");

    // 2. Bob connects: drains offline queue
    let mut bob_rx = router.register_client(&bob.did).await;
    assert!(router.is_online(&bob.did).await);

    let drained = router.drain_offline_packets(&bob.did, 1729183010).await;
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].sender_did, alice.did);
    assert_eq!(drained[0].recipient_did, bob.did);

    // 3. Alice sends second packet while Bob is online: delivered live directly via channel
    let delivered_live_2 = router.route_packet(packet.clone(), 1729183020).await
        .expect("Routing to online user should succeed");
    assert!(delivered_live_2, "Should report delivered live");

    let live_received = bob_rx.recv().await
        .expect("Bob should receive packet directly through channel");
    assert_eq!(live_received.sender_did, alice.did);

    // 4. DPDP purge: purges all state for Bob
    router.purge_user(&bob.did).await;
    assert!(!router.is_online(&bob.did).await);
}
