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
    let dup_resp = gateway.handle_register_identity(
        &identity.did,
        identity.document().verifying_key_bytes,
        now + 10,
    );
    assert_eq!(dup_resp.status_code, 409);
    assert_eq!(dup_resp.body["title"], "Identity Conflict");
    assert_eq!(dup_resp.body["jurisdiction"], "INDIA_MUMBAI_BLR");
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
    assert!(http_str.contains("Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n"));
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


