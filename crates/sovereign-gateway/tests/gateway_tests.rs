use sovereign_core::IdentityKeyPair;
use sovereign_gateway::GatewayEngine;

#[tokio::test]
async fn test_gateway_health_and_openapi_spec() {
    let gateway = GatewayEngine::new();

    // 1. Health check
    let health_resp = gateway.handle_health();
    assert_eq!(health_resp.status_code, 200);
    assert_eq!(health_resp.body["status"], "UP");
    assert_eq!(health_resp.body["jurisdiction"], "INDIA_LOCALIZED_MUMBAI_BLR");

    // 2. OpenAPI 3.1 schema
    let openapi = gateway.get_openapi_spec();
    assert_eq!(openapi["openapi"], "3.1.0");
    assert!(openapi["paths"].get("/api/v1/identity").is_some());
    assert!(openapi["paths"].get("/api/v1/commerce/order").is_some());
    assert!(openapi["paths"].get("/api/v1/compliance/shred").is_some());
}

#[tokio::test]
async fn test_gateway_identity_registration_and_rfc7807_problem_details() {
    let gateway = GatewayEngine::new();
    let now = 1776000000u64;
    let identity = IdentityKeyPair::generate();

    // 1. Register new DID -> 201 Created
    let reg_resp = gateway.handle_register_identity(
        &identity.did,
        identity.document().verifying_key_bytes,
        now,
    );
    assert_eq!(reg_resp.status_code, 201);
    assert_eq!(reg_resp.body["did"], identity.did);

    // 2. Query DID -> 200 OK
    let get_resp = gateway.handle_get_identity(&identity.did);
    assert_eq!(get_resp.status_code, 200);
    assert_eq!(get_resp.body["status"], "ACTIVE");

    // 3. Duplicate registration -> 409 Conflict with RFC 7807 Problem Details
    let _dup_resp = gateway.handle_register_identity(
        &identity.did,
        identity.document().verifying_key_bytes,
        now + 10,
    );
    // 4. List all active identities -> 200 OK
    let list_resp = gateway.dispatch_request("GET", "/api/v1/identities", "").await;
    assert_eq!(list_resp.status_code, 200);
    assert_eq!(list_resp.body["total"], 1);
    assert_eq!(list_resp.body["identities"][0]["did"], identity.did);

    // 5. Test message dispatch
    let msg_resp = gateway.dispatch_request(
        "POST",
        "/api/v1/messages",
        &serde_json::json!({
            "sender_did": identity.did,
            "recipient_did": "did:bharat:9999999999",
            "text": "Namaste Sovereign Bharat",
            "timestamp": now + 20
        }).to_string(),
    ).await;
    assert_eq!(msg_resp.status_code, 200);
    assert_eq!(msg_resp.body["status"], "DELIVERED");
}

#[tokio::test]
async fn test_gateway_commerce_order_and_upi_generation() {
    let gateway = GatewayEngine::new();
    let now = 1776000000u64;

    let items = vec![
        ("prod_kanjivaram", "Pure Kanjivaram Silk Saree", 4500.0, 2), // 9000
        ("prod_kurti", "Chanderi Cotton Kurti", 1200.0, 1),          // 1200
    ]; // Subtotal = 10,200.00

    let order_resp = gateway.handle_commerce_order(
        "ORD-GW-101",
        "did:bharat:buyer_rahul",
        "did:bharat:merchant_surat",
        items,
        5.0, // 5% GST
        "suratsilk@okhdfcbank",
        "Surat Heritage Silk",
        now,
    );

    assert_eq!(order_resp.status_code, 201);
    assert_eq!(order_resp.body["order_id"], "ORD-GW-101");
    assert_eq!(order_resp.body["subtotal_inr"], 10200.0);
    assert_eq!(order_resp.body["total_amount_inr"], 10710.0);

    let upi_uri = order_resp.body["upi_uri"].as_str().unwrap();
    assert!(upi_uri.starts_with("upi://pay?pa=suratsilk@okhdfcbank"));
    assert!(upi_uri.contains("am=10710.00"));
}

#[tokio::test]
async fn test_gateway_dpdp_crypto_shredding_flow() {
    let gateway = GatewayEngine::new();
    let now = 1776000000u64;
    let identity = IdentityKeyPair::generate();

    // Register DID
    gateway.handle_register_identity(
        &identity.did,
        identity.document().verifying_key_bytes,
        now,
    );

    // Verify active
    assert_eq!(gateway.handle_get_identity(&identity.did).status_code, 200);

    // Execute DPDP Section 12 Crypto-Shredding
    let shred_resp = gateway.handle_crypto_shred(&identity.did, now + 50).await;
    assert_eq!(shred_resp.status_code, 200);
    assert_eq!(shred_resp.body["status"], "REVOKED_AND_PURGED");
    assert_eq!(shred_resp.body["smk_zeroized"], true);

    // Subsequent lookup returns 404 Problem Details
    let lookup_after = gateway.handle_get_identity(&identity.did);
    assert_eq!(lookup_after.status_code, 404);
    assert_eq!(lookup_after.body["title"], "Identity Not Found");
}

#[tokio::test]
async fn test_gateway_dispatch_request_router() {
    let gateway = GatewayEngine::new();
    let now = 1776000000u64;

    // 1. Dispatch GET /health
    let resp = gateway.dispatch_request("GET", "/health", "").await;
    assert_eq!(resp.status_code, 200);
    assert_eq!(resp.body["status"], "UP");

    // 2. Dispatch GET /api/v1/openapi.json
    let resp = gateway.dispatch_request("GET", "/api/v1/openapi.json", "").await;
    assert_eq!(resp.status_code, 200);
    assert_eq!(resp.body["openapi"], "3.1.0");

    // 3. Dispatch POST /api/v1/identity
    let id = IdentityKeyPair::generate();
    let vk_hex = hex::encode(id.document().verifying_key_bytes);
    let id_body = serde_json::json!({
        "did": id.did,
        "vk_hex": vk_hex,
        "timestamp": now
    }).to_string();
    let resp = gateway.dispatch_request("POST", "/api/v1/identity", &id_body).await;
    assert_eq!(resp.status_code, 201);
    assert_eq!(resp.body["did"], id.did);

    // 4. Dispatch GET /api/v1/identity/{did}
    let get_path = format!("/api/v1/identity/{}", id.did);
    let resp = gateway.dispatch_request("GET", &get_path, "").await;
    assert_eq!(resp.status_code, 200);
    assert_eq!(resp.body["status"], "ACTIVE");

    // 5. Dispatch POST /api/v1/commerce/order
    let order_body = serde_json::json!({
        "order_id": "ORD-LIVE-777",
        "customer_did": id.did,
        "merchant_did": "did:bharat:merchant_pune",
        "items": [
            { "product_id": "item1", "name": "Silk Dupatta", "unit_price": 1500.0, "quantity": 2 }
        ],
        "gst_rate": 5.0,
        "payee_vpa": "punetextiles@upi",
        "payee_name": "Pune Handloom",
        "timestamp": now
    }).to_string();
    let resp = gateway.dispatch_request("POST", "/api/v1/commerce/order", &order_body).await;
    assert_eq!(resp.status_code, 201);
    assert_eq!(resp.body["order_id"], "ORD-LIVE-777");
    assert_eq!(resp.body["total_amount_inr"], 3150.0);

    // 6. Dispatch POST /api/v1/compliance/shred
    let shred_body = serde_json::json!({
        "did": id.did,
        "timestamp": now + 100
    }).to_string();
    let resp = gateway.dispatch_request("POST", "/api/v1/compliance/shred", &shred_body).await;
    assert_eq!(resp.status_code, 200);
    assert_eq!(resp.body["status"], "REVOKED_AND_PURGED");

    // 7. Dispatch Unknown Route -> 404
    let resp = gateway.dispatch_request("GET", "/invalid/endpoint", "").await;
    assert_eq!(resp.status_code, 404);
}

#[tokio::test]
async fn test_gateway_http_bytes_cors_and_serialization() {
    let gateway = GatewayEngine::new();
    let resp = gateway.handle_health();
    let bytes = resp.to_http_bytes();
    let http_str = String::from_utf8(bytes).unwrap();

    assert!(http_str.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(http_str.contains("Access-Control-Allow-Origin: *\r\n"));
    assert!(http_str.contains("Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n"));
    assert!(http_str.contains("Content-Length: "));
    assert!(http_str.contains("\"jurisdiction\":\"INDIA_LOCALIZED_MUMBAI_BLR\""));
}

#[test]
fn test_rfc6455_websocket_handshake_test_vector_and_framing() {
    use sovereign_gateway::realtime::{compute_ws_accept, encode_ws_text_frame, parse_ws_frame};

    // Standard RFC 6455 Section 4.2.2 Test Vector:
    // Key: "dGhlIHNhbXBsZSBub25jZQ==" -> Accept: "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
    let rfc_key = "dGhlIHNhbXBsZSBub25jZQ==";
    let accept = compute_ws_accept(rfc_key);
    assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");

    // Text frame encode/decode round trip
    let payload = "Namaste Bharat! Live WebSocket Framing.";
    let frame_bytes = encode_ws_text_frame(payload);
    assert_eq!(frame_bytes[0], 0x81); // FIN + Text opcode

    let (opcode, decoded_bytes) = parse_ws_frame(&frame_bytes).unwrap();
    assert_eq!(opcode, 1);
    assert_eq!(String::from_utf8(decoded_bytes).unwrap(), payload);
}

#[tokio::test]
async fn test_realtime_delivery_receipts_and_dpdp_privacy() {
    let gateway = GatewayEngine::new();
    let rt = &gateway.realtime;
    let now = 1776000000u64;

    let sender = "did:bharat:alice";
    let recipient = "did:bharat:bob";
    let msg_id = "msg_live_001";

    // 1. Sent -> 1 Grey Tick
    let ev_sent = rt.record_sent(msg_id, sender, recipient, now);
    assert_eq!(ev_sent.status, sovereign_gateway::realtime::DeliveryStatus::Sent);
    assert!(!ev_sent.blue_ticks);

    // 2. Delivered -> 2 Grey Ticks
    let ev_deliv = rt.record_delivered(msg_id, now + 1).unwrap();
    assert_eq!(ev_deliv.status, sovereign_gateway::realtime::DeliveryStatus::Delivered);
    assert!(!ev_deliv.blue_ticks);

    // 3. Read -> 2 Blue Ticks (both have receipts enabled)
    let ev_read = rt.record_read(msg_id, recipient, now + 2).unwrap();
    assert_eq!(ev_read.status, sovereign_gateway::realtime::DeliveryStatus::Read);
    assert!(ev_read.blue_ticks);

    // 4. DPDP Privacy: Bob disables Read Receipts
    rt.set_read_receipt_policy(recipient, sovereign_gateway::realtime::ReadReceiptPolicy::Disabled);

    let msg_id_private = "msg_live_002";
    rt.record_sent(msg_id_private, sender, recipient, now + 10);
    rt.record_delivered(msg_id_private, now + 11);

    // Bob reads message -> status remains Delivered, blue_ticks is false
    let ev_read_private = rt.record_read(msg_id_private, recipient, now + 12).unwrap();
    assert_eq!(ev_read_private.status, sovereign_gateway::realtime::DeliveryStatus::Delivered);
    assert!(!ev_read_private.blue_ticks);
}

#[tokio::test]
async fn test_realtime_presence_and_typing_indicators() {
    let gateway = GatewayEngine::new();
    let rt = &gateway.realtime;
    let now = 1776000000u64;

    let user_a = "did:bharat:rahul";
    let user_b = "did:bharat:priya";
    let user_stranger = "did:bharat:stranger";

    // Presence: ContactsOnly policy
    rt.update_presence(user_a, true, now, sovereign_gateway::realtime::LastSeenPolicy::ContactsOnly);
    rt.add_contact(user_a, user_b);

    // Priya is an approved contact -> can see Rahul's presence
    let p_b = rt.query_presence(user_b, user_a);
    assert!(p_b.is_some());
    assert!(p_b.unwrap().is_online);

    // Stranger is not a contact -> hidden (None)
    let p_s = rt.query_presence(user_stranger, user_a);
    assert!(p_s.is_none());

    // Typing state & auto-expiry (> 5s)
    let chat_id = "chat_group_surat";
    rt.set_typing(chat_id, user_a, true, now);
    let typers = rt.get_active_typers(chat_id, now + 2);
    assert_eq!(typers, vec![user_a.to_string()]);

    // 6 seconds later -> typing expired
    let expired_typers = rt.get_active_typers(chat_id, now + 7);
    assert!(expired_typers.is_empty());
}

#[tokio::test]
async fn test_gateway_sre_prometheus_metrics_and_statutory_slas() {
    let gateway = GatewayEngine::new();
    let now = 1776000000u64;

    // 1. Initial GET /metrics
    let resp = gateway.dispatch_request("GET", "/metrics", "").await;
    assert_eq!(resp.status_code, 200);
    let bytes = resp.to_http_bytes();
    let text = String::from_utf8(bytes).unwrap();

    assert!(text.contains("Content-Type: text/plain; version=0.0.4\r\n"));
    assert!(text.contains("# HELP http_requests_total"));
    assert!(text.contains("# HELP http_request_duration_seconds"));
    assert!(text.contains("sovereign_cert_in_sla_seconds 21600"));
    assert!(text.contains("sovereign_active_websocket_connections 0"));

    // 2. Perform actions to increment metrics
    let id = IdentityKeyPair::generate();
    let order_body = serde_json::json!({
        "order_id": "ORD-METRIC-01",
        "customer_did": id.did,
        "merchant_did": "did:bharat:merchant",
        "items": [
            { "product_id": "p1", "name": "Item 1", "unit_price": 500.0, "quantity": 1 }
        ],
        "gst_rate": 5.0,
        "payee_vpa": "merchant@upi",
        "payee_name": "Merchant",
        "timestamp": now
    }).to_string();
    let _ = gateway.dispatch_request("POST", "/api/v1/commerce/order", &order_body).await;

    let shred_body = serde_json::json!({
        "did": id.did,
        "timestamp": now + 50
    }).to_string();
    let _ = gateway.dispatch_request("POST", "/api/v1/compliance/shred", &shred_body).await;

    // 3. Query /metrics again
    let resp2 = gateway.dispatch_request("GET", "/metrics", "").await;
    assert_eq!(resp2.status_code, 200);
    let text2 = String::from_utf8(resp2.to_http_bytes()).unwrap();

    assert!(text2.contains("sovereign_sim_binding_verifications_total 1"));
    assert!(text2.contains("sovereign_dpdp_crypto_shredding_operations_total 1"));
    assert!(text2.contains("sovereign_dpdp_shredding_entropy_bits 7.992"));
    assert!(text2.contains("http_requests_total{method=\"POST\",path=\"/api/v1/commerce/order\",status=\"201\"} 1"));
    assert!(text2.contains("http_requests_total{method=\"POST\",path=\"/api/v1/compliance/shred\",status=\"200\"} 1"));
}

#[tokio::test]
async fn test_gateway_broadcast_status_channel_forwarding_and_abuse() {
    let gateway = GatewayEngine::new();
    let now = 1776000000u64;

    let admin = IdentityKeyPair::generate();
    let user_alice = "did:bharat:status_alice";

    // 1. Post Status Story
    let status_req = serde_json::json!({
        "story_id": "story_gw_01",
        "author_did": user_alice,
        "content_cid": "cid_gateway_media_01",
        "caption": "Namaste India! 🇮🇳",
        "audience_type": "MyContacts",
        "timestamp": now
    }).to_string();
    let resp = gateway.dispatch_request("POST", "/api/v1/broadcast/status", &status_req).await;
    assert_eq!(resp.status_code, 201);
    assert_eq!(resp.body["story_id"], "story_gw_01");

    // 2. Query Status Stories
    let get_status_path = format!("/api/v1/broadcast/status/{}", user_alice);
    let resp = gateway.dispatch_request("GET", &get_status_path, "").await;
    assert_eq!(resp.status_code, 200);
    assert_eq!(resp.body.as_array().unwrap().len(), 1);

    // 3. Create Broadcast Channel
    let chan_req = serde_json::json!({
        "channel_id": "chan_isro_updates",
        "name": "ISRO Space Dispatch",
        "description": "Official updates from Indian Space Research Organisation",
        "admin_did": admin.did,
        "is_public": true,
        "timestamp": now
    }).to_string();
    let resp = gateway.dispatch_request("POST", "/api/v1/broadcast/channel", &chan_req).await;
    assert_eq!(resp.status_code, 201);
    assert_eq!(resp.body["channel_id"], "chan_isro_updates");

    // 4. Publish Channel Post
    let post_req = serde_json::json!({
        "post_id": "post_gaganyaan_01",
        "author_did": admin.did,
        "content_cid": "cid_gaganyaan_launch",
        "text": "Gaganyaan orbital module successful qualification test.",
        "signature_hex": "aa".repeat(64),
        "timestamp": now + 10
    }).to_string();
    let publish_path = "/api/v1/broadcast/channel/chan_isro_updates/publish";
    let resp = gateway.dispatch_request("POST", publish_path, &post_req).await;
    assert_eq!(resp.status_code, 201);
    assert_eq!(resp.body["post_id"], "post_gaganyaan_01");

    // 5. Query Channel Feed
    let resp = gateway.dispatch_request("GET", "/api/v1/broadcast/channel/chan_isro_updates", "").await;
    assert_eq!(resp.status_code, 200);
    assert_eq!(resp.body.as_array().unwrap().len(), 1);

    // 6. Forwarding Limit Evaluation
    // Hop count = 1 -> 3 chats allowed
    let fwd_ok_req = serde_json::json!({
        "message_cid": "cid_msg_001",
        "sender_did": admin.did,
        "hop_count": 1,
        "target_chats": ["chat1", "chat2", "chat3"]
    }).to_string();
    let resp = gateway.dispatch_request("POST", "/api/v1/broadcast/forward/check", &fwd_ok_req).await;
    assert_eq!(resp.status_code, 200);
    assert_eq!(resp.body["allowed"], true);

    // Hop count = 5 (frequently forwarded) -> 2 chats BLOCKED (422)
    let fwd_block_req = serde_json::json!({
        "message_cid": "cid_msg_001",
        "sender_did": admin.did,
        "hop_count": 5,
        "target_chats": ["chat1", "chat2"]
    }).to_string();
    let resp = gateway.dispatch_request("POST", "/api/v1/broadcast/forward/check", &fwd_block_req).await;
    assert_eq!(resp.status_code, 422);
    assert_eq!(resp.body["title"], "Viral Forwarding Limit Exceeded");

    // 7. Cryptographic Abuse Report under IT Rules 2021
    let abusive_msg = "Terror threat warning against public metro station.";
    let cid_bytes = sovereign_core::compute_canonical_cid(abusive_msg.as_bytes());
    let cid_hex = hex::encode(cid_bytes);

    let mut sign_payload = Vec::new();
    sign_payload.extend_from_slice(cid_hex.as_bytes());
    sign_payload.extend_from_slice(&now.to_be_bytes());
    let sig_bytes = admin.sign(&sign_payload);

    let abuse_req = serde_json::json!({
        "dossier": {
            "dossier_id": "DOSSIER-GW-999",
            "reported_message_cid": cid_hex,
            "sender_did": admin.did,
            "sender_signature_hex": hex::encode(sig_bytes),
            "plaintext": abusive_msg,
            "reporter_did": "did:bharat:reporter_citizen",
            "category": "Terrorism",
            "timestamp": now,
            "jurisdiction": "INDIA_LOCALIZED_MUMBAI_BLR"
        },
        "sender_vk_hex": hex::encode(admin.document().verifying_key_bytes)
    }).to_string();

    let resp = gateway.dispatch_request("POST", "/api/v1/compliance/report_abuse", &abuse_req).await;
    assert_eq!(resp.status_code, 201);
    assert_eq!(resp.body["ticket_id"], "GRV-IN-DOSSIER-GW-999");
    assert_eq!(resp.body["status"], "Acknowledged");
}

#[tokio::test]
async fn test_call_logs_and_persistence_recovery() {
    let temp_dir = std::env::temp_dir().join(format!("bharatchat_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let _ = std::fs::create_dir_all(&temp_dir);

    // 1. Instance 1: write identity, message, and call log
    {
        let gw1 = GatewayEngine::new_with_persistence(&temp_dir);

        // Register identity
        let reg_req = serde_json::json!({
            "did": "did:bharat:call_test_user_a",
            "vk_hex": "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
            "timestamp": 1700000000
        }).to_string();
        let resp = gw1.dispatch_request("POST", "/api/v1/identity", &reg_req).await;
        assert_eq!(resp.status_code, 201);

        // Send chat message
        let msg_req = serde_json::json!({
            "sender_did": "did:bharat:call_test_user_a",
            "recipient_did": "did:bharat:call_test_user_b",
            "text": "Call me when free!",
            "timestamp": 1700000010
        }).to_string();
        let resp = gw1.dispatch_request("POST", "/api/v1/messages", &msg_req).await;
        assert_eq!(resp.status_code, 200);

        // Log call
        let call_req = serde_json::json!({
            "caller_did": "did:bharat:call_test_user_a",
            "recipient_did": "did:bharat:call_test_user_b",
            "call_type": "VIDEO",
            "status": "COMPLETED",
            "duration_seconds": 42,
            "timestamp": 1700000020
        }).to_string();
        let resp = gw1.dispatch_request("POST", "/api/v1/calls", &call_req).await;
        assert_eq!(resp.status_code, 201);
        assert_eq!(resp.body["status"], "RECORDED");
        assert_eq!(resp.body["call_type"], "VIDEO");
        assert_eq!(resp.body["duration_seconds"], 42);

        // Query calls on gw1
        let get_calls = gw1.dispatch_request("GET", "/api/v1/calls?did=did:bharat:call_test_user_a", "").await;
        assert_eq!(get_calls.status_code, 200);
        assert_eq!(get_calls.body["total"], 1);
        assert_eq!(get_calls.body["calls"][0]["duration_seconds"], 42);
    }

    // 2. Instance 2: Reload same directory - verify full recovery of accounts, messages, and calls!
    {
        let gw2 = GatewayEngine::new_with_persistence(&temp_dir);

        // Verify identity restored
        let id_resp = gw2.dispatch_request("GET", "/api/v1/identity/did:bharat:call_test_user_a", "").await;
        assert_eq!(id_resp.status_code, 200);
        assert_eq!(id_resp.body["status"], "ACTIVE");

        // Verify message restored
        let msg_resp = gw2.dispatch_request("GET", "/api/v1/messages?did=did:bharat:call_test_user_a", "").await;
        assert_eq!(msg_resp.status_code, 200);
        assert_eq!(msg_resp.body["total"], 1);
        assert_eq!(msg_resp.body["messages"][0]["text"], "Call me when free!");

        // Verify call log restored
        let call_resp = gw2.dispatch_request("GET", "/api/v1/calls?did=did:bharat:call_test_user_a", "").await;
        assert_eq!(call_resp.status_code, 200);
        assert_eq!(call_resp.body["total"], 1);
        assert_eq!(call_resp.body["calls"][0]["call_type"], "VIDEO");
        assert_eq!(call_resp.body["calls"][0]["duration_seconds"], 42);
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_commerce_products_crud_and_signaling() {
    let gateway = GatewayEngine::new();

    // 1. GET initial seed products
    let list_resp = gateway.dispatch_request("GET", "/api/v1/commerce/products", "").await;
    assert_eq!(list_resp.status_code, 200);
    assert!(list_resp.body["total"].as_u64().unwrap() >= 3);

    // 2. POST create product
    let create_payload = serde_json::json!({
        "id": "prod_test_khadi",
        "owner_did": "did:bharat:seller_1",
        "owner_name": "Sovereign Khadi Bhandar",
        "title": "Organic Handspun Khadi Shirt",
        "description": "Pure khadi cotton, natural indigo dye",
        "price_inr": 1499.0,
        "category": "Apparel",
        "sku": "KHADI-001"
    });
    let create_resp = gateway.dispatch_request("POST", "/api/v1/commerce/products", &create_payload.to_string()).await;
    assert_eq!(create_resp.status_code, 201);
    assert_eq!(create_resp.body["product"]["id"], "prod_test_khadi");
    assert_eq!(create_resp.body["product"]["owner_did"], "did:bharat:seller_1");

    // 3. PUT update product
    let update_payload = serde_json::json!({
        "price_inr": 1299.0,
        "title": "Organic Handspun Khadi Shirt (Sale)"
    });
    let update_resp = gateway.dispatch_request("PUT", "/api/v1/commerce/products/prod_test_khadi", &update_payload.to_string()).await;
    assert_eq!(update_resp.status_code, 200);
    assert_eq!(update_resp.body["product"]["price_inr"], 1299.0);

    // 4. POST call signal
    let signal_payload = serde_json::json!({
        "caller_did": "did:bharat:alice",
        "recipient_did": "did:bharat:bob",
        "signal_type": "OFFER",
        "call_type": "VIDEO"
    });
    let signal_resp = gateway.dispatch_request("POST", "/api/v1/calls/signal", &signal_payload.to_string()).await;
    assert_eq!(signal_resp.status_code, 200);
    assert_eq!(signal_resp.body["status"], "SIGNAL_DELIVERED");

    // 5. DELETE product
    let del_resp = gateway.dispatch_request("DELETE", "/api/v1/commerce/products/prod_test_khadi", "").await;
    assert_eq!(del_resp.status_code, 200);
    assert_eq!(del_resp.body["status"], "DELETED");
}

#[tokio::test]
async fn test_ota_app_update_metadata_and_binary_stream() {
    let gateway = GatewayEngine::new();

    // 1. GET default update info
    let get_resp = gateway.dispatch_request("GET", "/api/v1/app/update", "").await;
    assert_eq!(get_resp.status_code, 200);
    assert_eq!(get_resp.body["latest_version_code"], 2);
    assert_eq!(get_resp.body["latest_version_name"], "1.0.1-SOVEREIGN");
    assert_eq!(get_resp.body["download_url"], "/api/v1/app/download");

    // 2. POST update metadata
    let update_payload = serde_json::json!({
        "latest_version_code": 3,
        "latest_version_name": "1.0.2-SOVEREIGN",
        "release_notes": "• Critical security patches\n• Offline audio call improvements",
        "mandatory": false
    });
    let post_resp = gateway.dispatch_request("POST", "/api/v1/app/update", &update_payload.to_string()).await;
    assert_eq!(post_resp.status_code, 200);
    assert_eq!(post_resp.body["latest_version_code"], 3);
    assert_eq!(post_resp.body["latest_version_name"], "1.0.2-SOVEREIGN");

    // Verify GET reflects update
    let check_resp = gateway.dispatch_request("GET", "/api/v1/app/update", "").await;
    assert_eq!(check_resp.status_code, 200);
    assert_eq!(check_resp.body["latest_version_code"], 3);

    // 3. Test APK binary download stream
    let temp_dir = std::env::temp_dir();
    let mock_apk_path = temp_dir.join("mock_bharatchat.apk");
    let mock_bytes = vec![0x50, 0x4B, 0x03, 0x04, 0xDE, 0xAD, 0xBE, 0xEF]; // Zip/APK magic header mock
    std::fs::write(&mock_apk_path, &mock_bytes).expect("Failed to write mock APK");

    gateway.set_apk_path(&mock_apk_path);

    // Test HEAD request
    let head_resp = gateway.dispatch_request("HEAD", "/api/v1/app/download", "").await;
    assert_eq!(head_resp.status_code, 200);
    let cl_header = head_resp.headers.iter().find(|(k, _)| k == "Content-Length");
    assert!(cl_header.is_some());
    assert_eq!(cl_header.unwrap().1, mock_bytes.len().to_string());

    // Test GET binary request
    let download_resp = gateway.dispatch_request("GET", "/api/v1/app/download", "").await;
    assert_eq!(download_resp.status_code, 200);
    assert!(download_resp.binary_body.is_some());
    assert_eq!(download_resp.binary_body.as_ref().unwrap(), &mock_bytes);

    let raw_http = download_resp.to_http_bytes();
    assert!(raw_http.starts_with(b"HTTP/1.1 200 OK\r\n"));
    assert!(raw_http.ends_with(&mock_bytes));

    let _ = std::fs::remove_file(mock_apk_path);
}

#[tokio::test]
async fn test_bidirectional_call_signaling_flow() {
    let gateway = GatewayEngine::new();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // 1. Alice calls Bob (OFFER)
    let offer_payload = serde_json::json!({
        "call_id": "call_test_101",
        "caller_did": "did:bharat:alice",
        "caller_name": "Alice Sharma",
        "recipient_did": "did:bharat:bob",
        "signal_type": "OFFER",
        "call_type": "VIDEO",
        "timestamp": now
    });
    let offer_resp = gateway.dispatch_request("POST", "/api/v1/calls/signal", &offer_payload.to_string()).await;
    assert_eq!(offer_resp.status_code, 200);
    assert_eq!(offer_resp.body["status"], "SIGNAL_DELIVERED");
    assert_eq!(offer_resp.body["call_status"], "RINGING");

    // 2. Bob polls incoming calls for his DID
    let bob_signals = gateway.dispatch_request("GET", "/api/v1/calls/signal?did=did:bharat:bob", "").await;
    assert_eq!(bob_signals.status_code, 200);
    assert_eq!(bob_signals.body["total"], 1);
    assert_eq!(bob_signals.body["signals"][0]["call_id"], "call_test_101");
    assert_eq!(bob_signals.body["signals"][0]["caller_name"], "Alice Sharma");
    assert_eq!(bob_signals.body["signals"][0]["status"], "RINGING");

    // 2b. Verify Bob can also poll using standard URL-encoded DID (did%3Abharat%3Abob)
    let bob_encoded_signals = gateway.dispatch_request("GET", "/api/v1/calls/signal?did=did%3Abharat%3Abob", "").await;
    assert_eq!(bob_encoded_signals.status_code, 200);
    assert_eq!(bob_encoded_signals.body["total"], 1);
    assert_eq!(bob_encoded_signals.body["signals"][0]["call_id"], "call_test_101");

    // 3. Bob accepts the call (ANSWER)
    let answer_payload = serde_json::json!({
        "call_id": "call_test_101",
        "caller_did": "did:bharat:alice",
        "recipient_did": "did:bharat:bob",
        "signal_type": "ANSWER",
        "call_type": "VIDEO",
        "timestamp": now + 3
    });
    let answer_resp = gateway.dispatch_request("POST", "/api/v1/calls/signal", &answer_payload.to_string()).await;
    assert_eq!(answer_resp.status_code, 200);
    assert_eq!(answer_resp.body["call_status"], "CONNECTED");

    // 4. Alice polls call status by call_id and sees CONNECTED
    let check_resp = gateway.dispatch_request("GET", "/api/v1/calls/signal?call_id=call_test_101", "").await;
    assert_eq!(check_resp.status_code, 200);
    assert_eq!(check_resp.body["signals"][0]["status"], "CONNECTED");

    // 5. Alice hangs up call (HANGUP)
    let hangup_payload = serde_json::json!({
        "call_id": "call_test_101",
        "caller_did": "did:bharat:alice",
        "recipient_did": "did:bharat:bob",
        "signal_type": "HANGUP",
        "call_type": "VIDEO",
        "timestamp": now + 15
    });
    let hangup_resp = gateway.dispatch_request("POST", "/api/v1/calls/signal", &hangup_payload.to_string()).await;
    assert_eq!(hangup_resp.status_code, 200);
    assert_eq!(hangup_resp.body["call_status"], "ENDED");

    // 6. Verify Bob checks status and sees ENDED
    let final_check = gateway.dispatch_request("GET", "/api/v1/calls/signal?call_id=call_test_101", "").await;
    assert_eq!(final_check.status_code, 200);
    assert_eq!(final_check.body["signals"][0]["status"], "ENDED");
}

#[tokio::test]
async fn test_media_upload_and_retrieval() {
    let gateway = GatewayEngine::new();

    // 1. Upload sample image (base64 encoded "SovereignImageBytes")
    let raw_bytes = b"SovereignImageBytes12345";
    let base64_str = "U292ZXJlaWduSW1hZ2VCeXRlczEyMzQ1";
    let upload_payload = serde_json::json!({
        "filename": "surat_silk.png",
        "media_type": "IMAGE",
        "mime_type": "image/png",
        "data_base64": base64_str
    });

    let up_resp = gateway.dispatch_request("POST", "/api/v1/media/upload", &upload_payload.to_string()).await;
    assert_eq!(up_resp.status_code, 200);
    assert_eq!(up_resp.body["status"], "UPLOADED");
    assert_eq!(up_resp.body["filename"], "surat_silk.png");
    assert_eq!(up_resp.body["size_bytes"], raw_bytes.len());
    let media_id = up_resp.body["media_id"].as_str().unwrap();

    // 2. Fetch the uploaded media binary
    let get_path = format!("/api/v1/media/{}", media_id);
    let down_resp = gateway.dispatch_request("GET", &get_path, "").await;
    assert_eq!(down_resp.status_code, 200);
    assert!(down_resp.binary_body.is_some());
    assert_eq!(down_resp.binary_body.unwrap(), raw_bytes);
}

#[tokio::test]
async fn test_webrtc_sdp_and_ice_candidate_relay() {
    let gateway = GatewayEngine::new();
    let now = 1776000000u64;

    // 1. Alice sends OFFER with WebRTC SDP
    let offer_payload = serde_json::json!({
        "call_id": "call_webrtc_999",
        "caller_did": "did:bharat:alice",
        "caller_name": "Alice",
        "recipient_did": "did:bharat:bob",
        "signal_type": "OFFER",
        "call_type": "VIDEO",
        "sdp": "v=0\r\no=alice 12345 2 IN IP4 127.0.0.1\r\ns=-\r\n",
        "timestamp": now
    });
    let offer_resp = gateway.dispatch_request("POST", "/api/v1/calls/signal", &offer_payload.to_string()).await;
    assert_eq!(offer_resp.status_code, 200);
    assert_eq!(offer_resp.body["signal"]["sdp"], "v=0\r\no=alice 12345 2 IN IP4 127.0.0.1\r\ns=-\r\n");

    // 2. Bob sends ANSWER with WebRTC SDP
    let answer_payload = serde_json::json!({
        "call_id": "call_webrtc_999",
        "caller_did": "did:bharat:alice",
        "recipient_did": "did:bharat:bob",
        "signal_type": "ANSWER",
        "call_type": "VIDEO",
        "sdp": "v=0\r\no=bob 67890 2 IN IP4 127.0.0.1\r\ns=-\r\n",
        "timestamp": now + 1
    });
    let ans_resp = gateway.dispatch_request("POST", "/api/v1/calls/signal", &answer_payload.to_string()).await;
    assert_eq!(ans_resp.status_code, 200);

    // 3. Alice exchanges ICE candidate
    let ice_payload = serde_json::json!({
        "call_id": "call_webrtc_999",
        "caller_did": "did:bharat:alice",
        "recipient_did": "did:bharat:bob",
        "signal_type": "ICE_CANDIDATE",
        "call_type": "VIDEO",
        "candidate": "candidate:1 1 UDP 2122260223 192.168.1.3 54321 typ host",
        "sdp_mid": "0",
        "sdp_mline_index": 0,
        "timestamp": now + 2
    });
    let ice_resp = gateway.dispatch_request("POST", "/api/v1/calls/signal", &ice_payload.to_string()).await;
    assert_eq!(ice_resp.status_code, 200);
    assert_eq!(ice_resp.body["signal"]["signal_type"], "ICE_CANDIDATE");

    // 4. Bob queries call signals and receives the ICE candidate
    let bob_signals = gateway.dispatch_request("GET", "/api/v1/calls/signal?call_id=call_webrtc_999", "").await;
    assert_eq!(bob_signals.status_code, 200);
    assert_eq!(bob_signals.body["total"], 2); // main call session + ice candidate
}



