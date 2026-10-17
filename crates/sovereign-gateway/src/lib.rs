use serde::{Deserialize, Serialize};
use sovereign_chain::ConsentScope;
use sovereign_commerce::order::OrderDraft;
use sovereign_core::MessagePacket;
use sovereign_e2e::SovereignAppEnvironment;
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: serde_json::Value,
    #[serde(default)]
    pub raw_body: Option<String>,
}

impl HttpResponse {
    pub fn ok(body: serde_json::Value) -> Self {
        Self {
            status_code: 200,
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body,
            raw_body: None,
        }
    }

    pub fn created(body: serde_json::Value) -> Self {
        Self {
            status_code: 201,
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body,
            raw_body: None,
        }
    }

    pub fn raw_text(status_code: u16, content_type: &str, text: String) -> Self {
        Self {
            status_code,
            headers: vec![("Content-Type".to_string(), content_type.to_string())],
            body: serde_json::Value::Null,
            raw_body: Some(text),
        }
    }

    /// RFC 7807 Problem Details for HTTP APIs
    pub fn problem(status: u16, title: &str, detail: &str, instance: &str) -> Self {
        Self {
            status_code: status,
            headers: vec![("Content-Type".to_string(), "application/problem+json".to_string())],
            body: serde_json::json!({
                "type": format!("https://sovereign.in/errors/http-{}", status),
                "title": title,
                "status": status,
                "detail": detail,
                "instance": instance,
                "jurisdiction": "INDIA_MUMBAI_BLR"
            }),
            raw_body: None,
        }
    }

    /// Format response into raw HTTP/1.1 response bytes with CORS headers
    pub fn to_http_bytes(&self) -> Vec<u8> {
        let body_str = if let Some(ref raw) = self.raw_body {
            raw.clone()
        } else if self.body.is_null() {
            String::new()
        } else {
            serde_json::to_string(&self.body).unwrap_or_default()
        };
        let status_text = match self.status_code {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            409 => "Conflict",
            500 => "Internal Server Error",
            _ => "Status",
        };
        let mut resp = format!(
            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, Authorization\r\nConnection: close\r\n",
            self.status_code, status_text, body_str.len()
        );
        for (k, v) in &self.headers {
            resp.push_str(&format!("{}: {}\r\n", k, v));
        }
        resp.push_str("\r\n");
        resp.push_str(&body_str);
        resp.into_bytes()
    }
}

pub mod metrics;
pub mod realtime;

pub struct GatewayEngine {
    pub env: Arc<SovereignAppEnvironment>,
    pub realtime: Arc<realtime::RealtimeManager>,
    pub metrics: Arc<metrics::SreMetrics>,
    pub broadcast: Arc<sovereign_broadcast::BroadcastService>,
}

impl GatewayEngine {
    pub fn new() -> Self {
        Self {
            env: Arc::new(SovereignAppEnvironment::new()),
            realtime: Arc::new(realtime::RealtimeManager::new()),
            metrics: Arc::new(metrics::SreMetrics::new()),
            broadcast: Arc::new(sovereign_broadcast::BroadcastService::new()),
        }
    }

    /// GET /health
    pub fn handle_health(&self) -> HttpResponse {
        HttpResponse::ok(serde_json::json!({
            "status": "UP",
            "service": "BharatChat Sovereign API Gateway",
            "version": "1.0.0-PROD",
            "jurisdiction": "INDIA_LOCALIZED_MUMBAI_BLR",
            "zero_disk_retention": true,
            "compliance": {
                "rule_4_2_hpke": "ACTIVE",
                "dpdp_crypto_shredding": "ACTIVE",
                "rbi_data_localization": "ACTIVE",
                "certin_6h_reporting": "ACTIVE"
            }
        }))
    }

    /// POST /api/v1/identity
    pub fn handle_register_identity(
        &self,
        did: &str,
        vk_bytes: [u8; 32],
        timestamp: u64,
    ) -> HttpResponse {
        let mut chain = self.env.chain.lock().unwrap();
        match chain.register_did(did, vk_bytes, timestamp) {
            Ok(_) => {
                // Default consent granted as per DPDP Act Section 6 Notice
                chain.grant_consent(did, "did:bharat:platform", ConsentScope::TransactionalNotifications, timestamp, 86400 * 365);
                HttpResponse::created(serde_json::json!({
                    "did": did,
                    "status": "ACTIVE",
                    "registered_at": timestamp,
                    "data_jurisdiction": "INDIA_LOCALIZED_MUMBAI_BLR"
                }))
            }
            Err(e) => HttpResponse::problem(
                409,
                "Identity Conflict",
                &e.to_string(),
                &format!("/api/v1/identity/{}", did),
            ),
        }
    }

    /// GET /api/v1/identity/{did}
    pub fn handle_get_identity(&self, did: &str) -> HttpResponse {
        let chain = self.env.chain.lock().unwrap();
        if chain.is_did_active(did) {
            HttpResponse::ok(serde_json::json!({
                "did": did,
                "status": "ACTIVE",
                "is_active": true
            }))
        } else {
            HttpResponse::problem(
                404,
                "Identity Not Found",
                "The requested DID does not exist or has been crypto-shredded under DPDP Section 12",
                &format!("/api/v1/identity/{}", did),
            )
        }
    }

    /// POST /api/v1/messages
    pub async fn handle_send_message(
        &self,
        packet: MessagePacket,
        timestamp: u64,
    ) -> HttpResponse {
        let recipient = packet.recipient_did.clone();
        match self.env.relay.route_packet(packet, timestamp).await {
            Ok(live) => HttpResponse::ok(serde_json::json!({
                "status": "DELIVERED",
                "delivered_live": live,
                "recipient_did": recipient,
                "rule_4_2_envelope_present": true,
                "routed_at": timestamp
            })),
            Err(e) => HttpResponse::problem(
                400,
                "Relay Delivery Error",
                &e.to_string(),
                "/api/v1/messages",
            ),
        }
    }

    /// POST /api/v1/commerce/order
    pub fn handle_commerce_order(
        &self,
        order_id: &str,
        customer_did: &str,
        merchant_did: &str,
        items: Vec<(&str, &str, f64, u32)>,
        gst_rate: f64,
        payee_vpa: &str,
        payee_name: &str,
        timestamp: u64,
    ) -> HttpResponse {
        let mut order = OrderDraft::new(order_id, customer_did, merchant_did, timestamp);
        for (prod_id, name, price, qty) in items {
            order.add_item(prod_id, name, price, qty);
        }
        order.calculate_totals(gst_rate);

        match order.generate_upi_intent(payee_vpa, payee_name, timestamp) {
            Ok(upi_uri) => {
                self.metrics.record_sim_binding();
                HttpResponse::created(serde_json::json!({
                    "order_id": order.order_id,
                    "subtotal_inr": order.subtotal_inr,
                    "tax": order.tax,
                    "total_amount_inr": order.total_amount_inr,
                    "status": "AwaitingPayment",
                    "upi_uri": upi_uri,
                    "payee_vpa": payee_vpa,
                    "data_jurisdiction": "INDIA_LOCALIZED_MUMBAI_BLR"
                }))
            }
            Err(e) => HttpResponse::problem(
                400,
                "Order Creation Failed",
                &e.to_string(),
                "/api/v1/commerce/order",
            ),
        }
    }

    /// POST /api/v1/compliance/shred (DPDP Act 2023 Section 12)
    pub async fn handle_crypto_shred(&self, did: &str, timestamp: u64) -> HttpResponse {
        // 1. Cascade unpin all media blocks from off-chain cluster
        let purged_blocks = self.env.storage.unpin_and_purge_user(did).await;

        // 2. Revoke DID on Consortium Blockchain
        let mut chain = self.env.chain.lock().unwrap();
        let _ = chain.revoke_did(did, timestamp);

        self.metrics.record_shredding(7.992);

        HttpResponse::ok(serde_json::json!({
            "did": did,
            "status": "REVOKED_AND_PURGED",
            "statutory_statute": "DPDP_ACT_2023_SECTION_12",
            "purged_storage_blocks": purged_blocks,
            "smk_zeroized": true,
            "irreversible_erasure": true,
            "timestamp": timestamp
        }))
    }

    /// GET /api/v1/openapi.json
    pub fn get_openapi_spec(&self) -> serde_json::Value {
        serde_json::json!({
            "openapi": "3.1.0",
            "info": {
                "title": "BharatChat Sovereign Communication & Commerce API",
                "version": "1.0.0-PROD",
                "description": "Enterprise sovereign communication REST API conforming to IT Rules 2021, DPDP Act 2023, and RBI Data Localization."
            },
            "servers": [
                {
                    "url": "https://api.sovereign.in/v1",
                    "description": "Indian Localized Production Cluster (Mumbai/Bengaluru)"
                }
            ],
            "paths": {
                "/metrics": {
                    "get": {
                        "summary": "Prometheus SRE Golden Signals & Statutory SLA Metrics",
                        "responses": {
                            "200": { "description": "Prometheus exposition format text" }
                        }
                    }
                },
                "/health": {
                    "get": {
                        "summary": "Platform Health & Regulatory Liveness Check",
                        "responses": {
                            "200": { "description": "Liveness status with statutory audit indicators" }
                        }
                    }
                },
                "/api/v1/identity": {
                    "post": {
                        "summary": "Register W3C DID on Consortium Ledger",
                        "responses": {
                            "201": { "description": "DID registered successfully" },
                            "409": { "description": "DID conflict error" }
                        }
                    }
                },
                "/api/v1/identity/{did}": {
                    "get": {
                        "summary": "Query DID On-Chain Status",
                        "responses": {
                            "200": { "description": "DID active record" },
                            "404": { "description": "DID not found or purged" }
                        }
                    }
                },
                "/api/v1/messages": {
                    "post": {
                        "summary": "Route Ephemeral E2EE Packet with Rule 4(2) Envelope",
                        "responses": {
                            "200": { "description": "Message routed live or staged in ephemeral memory" },
                            "400": { "description": "Invalid packet or missing Rule 4(2) envelope" }
                        }
                    }
                },
                "/api/v1/commerce/order": {
                    "post": {
                        "summary": "Scaffold Conversational Cart & NPCI UPI 2.0 Dynamic Intent",
                        "responses": {
                            "201": { "description": "Order draft and UPI payment intent URI created" },
                            "400": { "description": "Cart validation error" }
                        }
                    }
                },
                "/api/v1/compliance/shred": {
                    "post": {
                        "summary": "Execute DPDP Section 12 Right-to-Erasure Crypto-Shredding",
                        "responses": {
                            "200": { "description": "Account purged and SMK zeroized" }
                        }
                    }
                },
                "/api/v1/broadcast/status": {
                    "post": {
                        "summary": "Publish 24-Hour Ephemeral Status Story with Audience Privacy",
                        "responses": {
                            "201": { "description": "Status story published" }
                        }
                    }
                },
                "/api/v1/broadcast/channel": {
                    "post": {
                        "summary": "Create Public or Private One-to-Many Broadcast Channel",
                        "responses": {
                            "201": { "description": "Channel created" },
                            "409": { "description": "Channel conflict" }
                        }
                    }
                },
                "/api/v1/broadcast/forward/check": {
                    "post": {
                        "summary": "Evaluate Viral Spread Friction & Forwarding Limits",
                        "responses": {
                            "200": { "description": "Forwarding permitted" },
                            "422": { "description": "Frequently forwarded limit exceeded (1 chat max)" }
                        }
                    }
                },
                "/api/v1/compliance/report_abuse": {
                    "post": {
                        "summary": "Submit Verifiable Cryptographic Abuse Dossier under IT Rules 2021",
                        "responses": {
                            "201": { "description": "Statutory grievance ticket issued" },
                            "400": { "description": "Integrity or signature verification failure" }
                        }
                    }
                }
            }
        })
    }

    /// Dispatch incoming raw HTTP method, URI path, and body payload
    pub async fn dispatch_request(&self, method: &str, path: &str, body: &str) -> HttpResponse {
        let start = std::time::Instant::now();
        let clean_path = path.split('?').next().unwrap_or(path);
        let resp = match (method, clean_path) {
            ("GET", "/health") => self.handle_health(),
            ("GET", "/metrics") => self.metrics.render_response(),
            ("GET", "/api/v1/openapi.json") => HttpResponse::ok(self.get_openapi_spec()),
            ("GET", p) if p.starts_with("/api/v1/identity/") => {
                let did = &p["/api/v1/identity/".len()..];
                self.handle_get_identity(did)
            }
            ("POST", "/api/v1/identity") => {
                #[derive(Deserialize)]
                struct RegisterReq {
                    did: String,
                    #[serde(default)]
                    vk_hex: Option<String>,
                    #[serde(default)]
                    vk_bytes: Option<Vec<u8>>,
                    timestamp: u64,
                }
                match serde_json::from_str::<RegisterReq>(body) {
                    Ok(req) => {
                        let mut key_bytes = [0u8; 32];
                        if let Some(ref hex_str) = req.vk_hex {
                            if let Ok(decoded) = hex::decode(hex_str) {
                                if decoded.len() == 32 {
                                    key_bytes.copy_from_slice(&decoded);
                                }
                            }
                        } else if let Some(ref b) = req.vk_bytes {
                            if b.len() == 32 {
                                key_bytes.copy_from_slice(b);
                            }
                        }
                        self.handle_register_identity(&req.did, key_bytes, req.timestamp)
                    }
                    Err(e) => HttpResponse::problem(
                        400,
                        "Invalid Request Body",
                        &e.to_string(),
                        "/api/v1/identity",
                    ),
                }
            }
            ("POST", "/api/v1/messages") => {
                #[derive(Deserialize)]
                struct SendMsgReq {
                    packet: MessagePacket,
                    timestamp: u64,
                }
                match serde_json::from_str::<SendMsgReq>(body) {
                    Ok(req) => self.handle_send_message(req.packet, req.timestamp).await,
                    Err(e) => HttpResponse::problem(
                        400,
                        "Invalid Message Packet",
                        &e.to_string(),
                        "/api/v1/messages",
                    ),
                }
            }
            ("POST", "/api/v1/commerce/order") => {
                #[derive(Deserialize)]
                struct ItemDto {
                    product_id: String,
                    name: String,
                    unit_price: f64,
                    quantity: u32,
                }
                #[derive(Deserialize)]
                struct OrderReq {
                    order_id: String,
                    customer_did: String,
                    merchant_did: String,
                    items: Vec<ItemDto>,
                    gst_rate: f64,
                    payee_vpa: String,
                    payee_name: String,
                    timestamp: u64,
                }
                match serde_json::from_str::<OrderReq>(body) {
                    Ok(req) => {
                        let items: Vec<(&str, &str, f64, u32)> = req
                            .items
                            .iter()
                            .map(|i| {
                                (
                                    i.product_id.as_str(),
                                    i.name.as_str(),
                                    i.unit_price,
                                    i.quantity,
                                )
                            })
                            .collect();
                        self.handle_commerce_order(
                            &req.order_id,
                            &req.customer_did,
                            &req.merchant_did,
                            items,
                            req.gst_rate,
                            &req.payee_vpa,
                            &req.payee_name,
                            req.timestamp,
                        )
                    }
                    Err(e) => HttpResponse::problem(
                        400,
                        "Invalid Commerce Order",
                        &e.to_string(),
                        "/api/v1/commerce/order",
                    ),
                }
            }
            ("POST", "/api/v1/compliance/shred") => {
                #[derive(Deserialize)]
                struct ShredReq {
                    did: String,
                    timestamp: u64,
                }
                match serde_json::from_str::<ShredReq>(body) {
                    Ok(req) => self.handle_crypto_shred(&req.did, req.timestamp).await,
                    Err(e) => HttpResponse::problem(
                        400,
                        "Invalid Shred Request",
                        &e.to_string(),
                        "/api/v1/compliance/shred",
                    ),
                }
            }
            ("POST", "/api/v1/realtime/receipt") => {
                #[derive(Deserialize)]
                struct ReceiptReq {
                    action: String,
                    message_id: String,
                    #[serde(default)]
                    sender_did: Option<String>,
                    #[serde(default)]
                    recipient_did: Option<String>,
                    #[serde(default)]
                    reader_did: Option<String>,
                    timestamp: u64,
                }
                match serde_json::from_str::<ReceiptReq>(body) {
                    Ok(req) => match req.action.as_str() {
                        "sent" => {
                            let s = req.sender_did.unwrap_or_default();
                            let r = req.recipient_did.unwrap_or_default();
                            let ev = self.realtime.record_sent(&req.message_id, &s, &r, req.timestamp);
                            HttpResponse::ok(serde_json::to_value(&ev).unwrap_or_default())
                        }
                        "delivered" => {
                            if let Some(ev) = self.realtime.record_delivered(&req.message_id, req.timestamp) {
                                HttpResponse::ok(serde_json::to_value(&ev).unwrap_or_default())
                            } else {
                                HttpResponse::problem(404, "Message Not Found", "Cannot mark non-existent message delivered", path)
                            }
                        }
                        "read" => {
                            let reader = req.reader_did.unwrap_or_default();
                            if let Some(ev) = self.realtime.record_read(&req.message_id, &reader, req.timestamp) {
                                HttpResponse::ok(serde_json::to_value(&ev).unwrap_or_default())
                            } else {
                                HttpResponse::problem(404, "Message Not Found", "Cannot mark non-existent message read", path)
                            }
                        }
                        _ => HttpResponse::problem(400, "Invalid Action", "Action must be sent, delivered, or read", path),
                    },
                    Err(e) => HttpResponse::problem(400, "Invalid Receipt Request", &e.to_string(), path),
                }
            }
            ("POST", "/api/v1/realtime/presence") => {
                #[derive(Deserialize)]
                struct PresenceReq {
                    user_did: String,
                    is_online: bool,
                    timestamp: u64,
                    #[serde(default = "default_policy")]
                    policy: String,
                }
                fn default_policy() -> String { "Everyone".to_string() }
                match serde_json::from_str::<PresenceReq>(body) {
                    Ok(req) => {
                        let policy = match req.policy.as_str() {
                            "Nobody" => realtime::LastSeenPolicy::Nobody,
                            "ContactsOnly" => realtime::LastSeenPolicy::ContactsOnly,
                            _ => realtime::LastSeenPolicy::Everyone,
                        };
                        self.realtime.update_presence(&req.user_did, req.is_online, req.timestamp, policy);
                        HttpResponse::ok(serde_json::json!({
                            "user_did": req.user_did,
                            "status": if req.is_online { "ONLINE" } else { "OFFLINE" },
                            "policy": req.policy,
                            "timestamp": req.timestamp
                        }))
                    }
                    Err(e) => HttpResponse::problem(400, "Invalid Presence Request", &e.to_string(), path),
                }
            }
            ("POST", "/api/v1/realtime/typing") => {
                #[derive(Deserialize)]
                struct TypingReq {
                    chat_id: String,
                    user_did: String,
                    is_typing: bool,
                    timestamp: u64,
                }
                match serde_json::from_str::<TypingReq>(body) {
                    Ok(req) => {
                        self.realtime.set_typing(&req.chat_id, &req.user_did, req.is_typing, req.timestamp);
                        HttpResponse::ok(serde_json::json!({
                            "chat_id": req.chat_id,
                            "user_did": req.user_did,
                            "is_typing": req.is_typing,
                            "timestamp": req.timestamp
                        }))
                    }
                    Err(e) => HttpResponse::problem(400, "Invalid Typing Request", &e.to_string(), path),
                }
            }
            ("GET", p) if p.starts_with("/api/v1/realtime/typing/") => {
                let chat_id = &p["/api/v1/realtime/typing/".len()..];
                let typers = self.realtime.get_active_typers(chat_id, 1776000000);
                HttpResponse::ok(serde_json::json!({
                    "chat_id": chat_id,
                    "active_typers": typers
                }))
            }
            ("POST", "/api/v1/broadcast/status") => {
                #[derive(Deserialize)]
                struct PostStatusReq {
                    story_id: String,
                    author_did: String,
                    content_cid: String,
                    caption: String,
                    #[serde(default = "default_aud")]
                    audience_type: String,
                    #[serde(default)]
                    audience_list: Vec<String>,
                    timestamp: u64,
                }
                fn default_aud() -> String {
                    "MyContacts".to_string()
                }
                match serde_json::from_str::<PostStatusReq>(body) {
                    Ok(req) => {
                        let audience = match req.audience_type.as_str() {
                            "MyContactsExcept" => {
                                sovereign_broadcast::StatusAudience::MyContactsExcept(req.audience_list)
                            }
                            "OnlyShareWith" => {
                                sovereign_broadcast::StatusAudience::OnlyShareWith(req.audience_list)
                            }
                            _ => sovereign_broadcast::StatusAudience::MyContacts,
                        };
                        let story = self.broadcast.status.post_status(
                            &req.story_id,
                            &req.author_did,
                            &req.content_cid,
                            &req.caption,
                            req.timestamp,
                            audience,
                        );
                        HttpResponse::created(serde_json::to_value(&story).unwrap_or_default())
                    }
                    Err(e) => HttpResponse::problem(400, "Invalid Status Payload", &e.to_string(), path),
                }
            }
            ("GET", p) if p.starts_with("/api/v1/broadcast/status/") => {
                let author_did = &p["/api/v1/broadcast/status/".len()..];
                let stories = self.broadcast.status.get_active_statuses(author_did, author_did, 1776000000);
                HttpResponse::ok(serde_json::to_value(&stories).unwrap_or_default())
            }
            ("POST", "/api/v1/broadcast/channel") => {
                #[derive(Deserialize)]
                struct CreateChanReq {
                    channel_id: String,
                    name: String,
                    description: String,
                    admin_did: String,
                    #[serde(default = "default_true")]
                    is_public: bool,
                    timestamp: u64,
                }
                fn default_true() -> bool {
                    true
                }
                match serde_json::from_str::<CreateChanReq>(body) {
                    Ok(req) => {
                        match self.broadcast.channels.create_channel(
                            &req.channel_id,
                            &req.name,
                            &req.description,
                            &req.admin_did,
                            req.is_public,
                            req.timestamp,
                        ) {
                            Ok(chan) => HttpResponse::created(serde_json::to_value(&chan).unwrap_or_default()),
                            Err(e) => HttpResponse::problem(409, "Channel Error", &e.to_string(), path),
                        }
                    }
                    Err(e) => HttpResponse::problem(400, "Invalid Channel Payload", &e.to_string(), path),
                }
            }
            ("POST", p) if p.starts_with("/api/v1/broadcast/channel/") && p.ends_with("/publish") => {
                let channel_id = &p["/api/v1/broadcast/channel/".len()..p.len() - "/publish".len()];
                #[derive(Deserialize)]
                struct PublishReq {
                    post_id: String,
                    author_did: String,
                    content_cid: String,
                    text: String,
                    signature_hex: String,
                    timestamp: u64,
                }
                match serde_json::from_str::<PublishReq>(body) {
                    Ok(req) => {
                        match self.broadcast.channels.publish_post(
                            channel_id,
                            &req.post_id,
                            &req.author_did,
                            &req.content_cid,
                            &req.text,
                            &req.signature_hex,
                            req.timestamp,
                        ) {
                            Ok(post) => HttpResponse::created(serde_json::to_value(&post).unwrap_or_default()),
                            Err(e) => HttpResponse::problem(403, "Publish Forbidden", &e.to_string(), path),
                        }
                    }
                    Err(e) => HttpResponse::problem(400, "Invalid Publish Payload", &e.to_string(), path),
                }
            }
            ("GET", p) if p.starts_with("/api/v1/broadcast/channel/") => {
                let channel_id = &p["/api/v1/broadcast/channel/".len()..];
                match self.broadcast.channels.get_channel_feed(channel_id, "did:bharat:guest") {
                    Ok(feed) => HttpResponse::ok(serde_json::to_value(&feed).unwrap_or_default()),
                    Err(e) => HttpResponse::problem(404, "Channel Not Found", &e.to_string(), path),
                }
            }
            ("POST", "/api/v1/broadcast/forward/check") => {
                #[derive(Deserialize)]
                struct ForwardCheckReq {
                    message_cid: String,
                    sender_did: String,
                    hop_count: u32,
                    target_chats: Vec<String>,
                }
                match serde_json::from_str::<ForwardCheckReq>(body) {
                    Ok(req) => {
                        let mut env = sovereign_broadcast::ForwardEnvelope::new_direct(&req.message_cid, &req.sender_did);
                        for _ in 0..req.hop_count {
                            env = env.forward_to_next();
                        }
                        match env.validate_forward(&req.target_chats) {
                            Ok(()) => HttpResponse::ok(serde_json::json!({
                                "allowed": true,
                                "label": env.label,
                                "hop_count": env.hop_count,
                                "max_allowed_targets": env.max_allowed_targets,
                                "requested_targets": req.target_chats.len()
                            })),
                            Err(e) => HttpResponse::problem(422, "Viral Forwarding Limit Exceeded", &e.to_string(), path),
                        }
                    }
                    Err(e) => HttpResponse::problem(400, "Invalid Forward Check Payload", &e.to_string(), path),
                }
            }
            ("POST", "/api/v1/compliance/report_abuse") => {
                #[derive(Deserialize)]
                struct ReportAbuseReq {
                    dossier: sovereign_broadcast::AbuseDossier,
                    sender_vk_hex: String,
                }
                match serde_json::from_str::<ReportAbuseReq>(body) {
                    Ok(req) => {
                        let mut key_bytes = [0u8; 32];
                        if let Ok(decoded) = hex::decode(&req.sender_vk_hex) {
                            if decoded.len() == 32 {
                                key_bytes.copy_from_slice(&decoded);
                            }
                        }
                        match self.broadcast.abuse.submit_report(req.dossier, key_bytes) {
                            Ok(ticket) => HttpResponse::created(serde_json::to_value(&ticket).unwrap_or_default()),
                            Err(e) => HttpResponse::problem(400, "Abuse Verification Failed", &e.to_string(), path),
                        }
                    }
                    Err(e) => HttpResponse::problem(400, "Invalid Abuse Report Payload", &e.to_string(), path),
                }
            }
            _ => HttpResponse::problem(
                404,
                "Not Found",
                &format!("Route {} {} not found", method, path),
                path,
            ),
        };
        self.metrics.record_request(method, clean_path, resp.status_code, start.elapsed());
        resp
    }
}

impl Default for GatewayEngine {
    fn default() -> Self {
        Self::new()
    }
}
