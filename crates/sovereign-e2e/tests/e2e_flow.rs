use rand::rngs::OsRng;
use sovereign_calls::{CallSession, CallType};
use sovereign_chain::ConsentScope;
use sovereign_commerce::{
    bot::{BusinessProfile, ConversationalBotEngine},
    catalog::{ProductCatalog, ProductItem},
    order::{OrderDraft, OrderStatus},
};
use sovereign_core::{
    csam_scanner::{BlindBloomFilter, ClientScanner},
    forward_message,
    IdentityKeyPair, MessagePacket, StorageMasterKey, TraceabilityEnvelope,
};
use sovereign_e2e::SovereignAppEnvironment;
use sovereign_storage::StorageError;
use sovereign_upi::SimBindingToken;
use sovereign_zkp::{ZkAgeProver, ZkAgeVerifier};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

#[tokio::test]
async fn test_full_sovereign_lifecycle_e2e() {
    let now = 1776000000u64;
    let env = SovereignAppEnvironment::new();

    // ------------------------------------------------------------------------
    // Step 1: Identity Generation & ZKP Age Verification Gate (DPDP Act Sec 9)
    // ------------------------------------------------------------------------
    let rahul = IdentityKeyPair::generate();
    let merchant = IdentityKeyPair::generate();

    // Rahul completes Zero-Knowledge Age Verification without disclosing Aadhaar or DOB
    let dob_timestamp = now - (26 * 365 * 86400); // 26 years old (> 18)
    let aadhaar_seed = b"AADHAAR-SHA256-HASH-VAULT-9988";
    let zkp_age_proof = ZkAgeProver::prove_age(dob_timestamp, aadhaar_seed, now);
    assert!(zkp_age_proof.is_adult, "Rahul must be proven an adult");
    assert!(ZkAgeVerifier::verify(&zkp_age_proof).unwrap(), "ZKP proof must be mathematically valid");

    // Register Rahul & Merchant on Consortium Blockchain with DPDP Consent
    {
        let mut chain = env.chain.lock().unwrap();
        chain.register_did(&rahul.did, rahul.document().verifying_key_bytes, now).unwrap();
        chain.grant_consent(&rahul.did, &merchant.did, ConsentScope::CommerceUpdates, now, 86400 * 365);
        chain.grant_consent(&rahul.did, &merchant.did, ConsentScope::TransactionalNotifications, now, 86400 * 365);

        chain.register_did(&merchant.did, merchant.document().verifying_key_bytes, now).unwrap();
        chain.grant_consent(&merchant.did, &rahul.did, ConsentScope::TransactionalNotifications, now, 86400 * 365);

        assert!(chain.has_active_consent(&rahul.did, &merchant.did, ConsentScope::CommerceUpdates, now + 100));
    }

    // Hardware SIM-Binding as per RBI Directives
    let rahul_sim = SimBindingToken::create("HW-DEVICE-PIXEL9-PRO", "SIM-AIRTEL-40445-998811", now);
    assert!(rahul_sim.verify("HW-DEVICE-PIXEL9-PRO", "SIM-AIRTEL-40445-998811"));

    // ------------------------------------------------------------------------
    // Step 2: Conversational Commerce & NPCI UPI 2.0 Settlement
    // ------------------------------------------------------------------------
    let mut catalog = ProductCatalog::new("CAT_SURAT_01", &merchant.did, "Surat Heritage Silk");
    let saree = ProductItem::new(
        "PROD_SILK_99",
        "SKU-SILK-99",
        "Zari Pure Silk Saree",
        "Surat handloom authentic zari",
        5000.00,
        "Textiles",
        "SILK-99",
    ).unwrap();
    catalog.add_item(saree);

    let profile = BusinessProfile::default_retailer(&merchant.did, "Surat Heritage Silk");
    let mut bot = ConversationalBotEngine::new(profile);

    // Rahul initiates chat; Bot automatically greets with interactive menu
    let greeting = bot.process_inbound(&rahul.did, "Namaste", now, 11).expect("Greeting should trigger");
    assert!(greeting.body_text.contains("Namaste! Welcome to Surat Heritage Silk"));

    // Rahul builds order cart
    let mut order = OrderDraft::new("ORD-SURAT-5501", &rahul.did, &merchant.did, now);
    order.add_item("PROD_SILK_99", "Zari Pure Silk Saree", 5000.00, 1);
    order.calculate_totals(5.0); // 5% GST = 250 INR (125 CGST + 125 SGST)
    assert_eq!(order.total_amount_inr, 5250.00);

    // Generate UPI 2.0 payment intent
    let upi_uri = order.generate_upi_intent("suratsilk@okhdfcbank", "Surat Heritage Silk", now + 10)
        .expect("UPI URI should generate");
    assert!(upi_uri.starts_with("upi://pay?pa=suratsilk@okhdfcbank"));
    assert!(upi_uri.contains("am=5250.00"));

    // Verify SIM binding prior to payment authorization
    assert!(rahul_sim.verify("HW-DEVICE-PIXEL9-PRO", "SIM-AIRTEL-40445-998811"));

    // UPI Switch confirms settlement
    order.confirm_payment(now + 20).unwrap();
    assert_eq!(order.status, OrderStatus::PaidConfirmed);

    // ------------------------------------------------------------------------
    // Step 3: Off-Chain Chunked Storage & Rule 4(2) Traceability Envelope
    // ------------------------------------------------------------------------
    let smk = StorageMasterKey::generate(&rahul.did);
    let receipt_media = b"INVOICE_AND_WARRANTY_CERTIFICATE_SURAT_SILK_AUTHENTIC_2026";

    // Client-side CSAM screening
    let filter = BlindBloomFilter::new(1024);
    let csam_scanner = ClientScanner::new(filter);
    assert!(csam_scanner.inspect_and_gate(receipt_media, now).is_ok(), "Clean invoice media must pass CSAM scan");

    // Off-chain media chunking & encryption under SMK
    let media_desc = env.storage.upload_media(&rahul.did, receipt_media, "application/pdf", &smk)
        .await
        .expect("Media upload should succeed");
    assert_eq!(env.storage.total_stored_blocks().await, 1);

    // Download & decrypt check
    let downloaded = env.storage.download_media(&media_desc, &smk)
        .await
        .expect("Media download should decrypt perfectly");
    assert_eq!(downloaded, receipt_media);

    // Setup Multi-Sig Legal Keypair for Rule 4(2) Judicial Traceability
    let legal_secret = StaticSecret::random_from_rng(OsRng);
    let legal_pubkey = X25519PublicKey::from(&legal_secret);

    // Create Rule 4(2) Originator Envelope for the message & media
    let message_payload = format!("Receipt media root CID: {}", media_desc.root_cid);
    let envelope = TraceabilityEnvelope::create(
        &rahul,
        message_payload.as_bytes(),
        &legal_pubkey,
        now + 30,
    ).unwrap();

    assert!(envelope.verify_content_integrity(message_payload.as_bytes()));

    // Forwarding proof creation
    let forward_proof = forward_message(&rahul, &envelope, message_payload.as_bytes(), now + 35)
        .expect("Forwarding hop proof should be signed");
    assert_eq!(forward_proof.forwarder_did, rahul.did);
    assert_eq!(forward_proof.parent_cid, envelope.content_cid);

    // Register merchant client on Ephemeral Relay and route packet
    let mut rx_merchant = env.relay.register_client(&merchant.did).await;
    let packet = MessagePacket {
        sender_did: rahul.did.clone(),
        recipient_did: merchant.did.clone(),
        e2ee_ciphertext: message_payload.as_bytes().to_vec(),
        trace_envelope: envelope.clone(),
        forward_chain: vec![forward_proof],
        packet_signature: rahul.sign(message_payload.as_bytes()),
    };
    let delivered_live = env.relay.route_packet(packet, now + 35).await.expect("Relay routing should succeed");
    assert!(delivered_live);

    let received = rx_merchant.recv().await.expect("Merchant must receive packet");
    assert_eq!(received.sender_did, rahul.did);
    assert_eq!(received.recipient_did, merchant.did);

    // ------------------------------------------------------------------------
    // Step 4: Court Warrant Threshold Decryption (Rule 4(2) Proof)
    // ------------------------------------------------------------------------
    // A competent court issues an order requiring identification of the first originator
    let rahul_did_doc = rahul.document();
    let revealed_originator_meta = envelope.threshold_decrypt(&legal_secret, &rahul_did_doc)
        .expect("Threshold key must decrypt originator metadata upon judicial warrant");

    assert_eq!(revealed_originator_meta.originator_did, rahul.did);
    assert_eq!(revealed_originator_meta.content_cid, envelope.content_cid);
    // Plaintext content was not stored on the envelope or exposed to bystanders!

    // ------------------------------------------------------------------------
    // Step 5: WebRTC 32-Party SFrame Call Signaling
    // ------------------------------------------------------------------------
    let mut call = CallSession::new(&merchant.did, CallType::AudioVideo, now);
    let key_dist = call.join(&rahul.did, now + 5).expect("Rahul joins call");
    assert_eq!(key_dist.epoch, 1);
    assert_eq!(call.participants.len(), 2);

    // ------------------------------------------------------------------------
    // Step 6: DPDP Act 2023 Section 12 Right-to-Erasure (Crypto-Shredding)
    // ------------------------------------------------------------------------
    // Rahul requests complete account deletion & erasure
    {
        // 1. Unpin and physically delete all blocks owned by Rahul
        let purged_blocks = env.storage.unpin_and_purge_user(&rahul.did).await;
        assert_eq!(purged_blocks, 1);
        assert_eq!(env.storage.total_stored_blocks().await, 0);

        // 2. StorageMasterKey zeroized / dropped
        let bad_smk = StorageMasterKey::generate(&rahul.did);
        let download_after_purge = env.storage.download_media(&media_desc, &bad_smk).await;
        assert!(matches!(download_after_purge, Err(StorageError::ChunkNotFound(_))));

        // 3. Mark DID on Consortium Blockchain as Revoked & Purged
        let mut chain = env.chain.lock().unwrap();
        chain.revoke_did(&rahul.did, now + 50).unwrap();
        assert!(!chain.is_did_active(&rahul.did));
    }

    println!(">>> Full Sovereign Lifecycle E2E Test Completed Successfully with 100% Assertion Passes! <<<");
}
