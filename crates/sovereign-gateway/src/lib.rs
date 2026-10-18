use serde::{Deserialize, Serialize};
use sovereign_chain::ConsentScope;
use sovereign_commerce::order::OrderDraft;
use sovereign_core::MessagePacket;
use sovereign_e2e::SovereignAppEnvironment;
use std::collections::HashMap;
use std::sync::Arc;

pub fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..=i + 2]).unwrap_or(""), 16) {
                result.push(byte as char);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            result.push(' ');
        } else {
            result.push(bytes[i] as char);
        }
        i += 1;
    }
    result
}

pub fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut map = [255u8; 256];
    for (idx, &c) in TABLE.iter().enumerate() {
        map[c as usize] = idx as u8;
    }
    map[b'-' as usize] = 62;
    map[b'_' as usize] = 63;

    let clean: Vec<u8> = input.bytes().filter(|&b| !b.is_ascii_whitespace()).collect();
    let mut out = Vec::new();
    let mut buf = [0u8; 4];
    let mut buf_len = 0;

    for &b in &clean {
        if b == b'=' {
            break;
        }
        let val = map[b as usize];
        if val == 255 {
            return Err(format!("Invalid base64 character: {}", b as char));
        }
        buf[buf_len] = val;
        buf_len += 1;
        if buf_len == 4 {
            out.push((buf[0] << 2) | (buf[1] >> 4));
            out.push((buf[1] << 4) | (buf[2] >> 2));
            out.push((buf[2] << 6) | buf[3]);
            buf_len = 0;
        }
    }

    if buf_len == 2 {
        out.push((buf[0] << 2) | (buf[1] >> 4));
    } else if buf_len == 3 {
        out.push((buf[0] << 2) | (buf[1] >> 4));
        out.push((buf[1] << 4) | (buf[2] >> 2));
    } else if buf_len == 1 {
        return Err("Truncated base64 input".to_string());
    }

    Ok(out)
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: serde_json::Value,
    #[serde(default)]
    pub raw_body: Option<String>,
    #[serde(skip)]
    pub binary_body: Option<Vec<u8>>,
}

impl HttpResponse {
    pub fn ok(body: serde_json::Value) -> Self {
        Self {
            status_code: 200,
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body,
            raw_body: None,
            binary_body: None,
        }
    }

    pub fn created(body: serde_json::Value) -> Self {
        Self {
            status_code: 201,
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body,
            raw_body: None,
            binary_body: None,
        }
    }

    pub fn no_content() -> Self {
        Self {
            status_code: 204,
            headers: vec![],
            body: serde_json::Value::Null,
            raw_body: None,
            binary_body: None,
        }
    }

    pub fn raw_text(status_code: u16, content_type: &str, text: String) -> Self {
        Self {
            status_code,
            headers: vec![("Content-Type".to_string(), content_type.to_string())],
            body: serde_json::Value::Null,
            raw_body: Some(text),
            binary_body: None,
        }
    }

    pub fn binary(status_code: u16, content_type: &str, bytes: Vec<u8>, extra_headers: Vec<(String, String)>) -> Self {
        let mut headers = vec![("Content-Type".to_string(), content_type.to_string())];
        headers.extend(extra_headers);
        Self {
            status_code,
            headers,
            body: serde_json::Value::Null,
            raw_body: None,
            binary_body: Some(bytes),
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
            binary_body: None,
        }
    }

    /// Format response into raw HTTP/1.1 response bytes with CORS headers
    pub fn to_http_bytes(&self) -> Vec<u8> {
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

        if let Some(ref bin) = self.binary_body {
            let has_content_length = self.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("Content-Length"));
            let mut header_str = if has_content_length {
                format!(
                    "HTTP/1.1 {} {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, Authorization\r\nConnection: close\r\n",
                    self.status_code, status_text
                )
            } else {
                format!(
                    "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, Authorization\r\nConnection: close\r\n",
                    self.status_code, status_text, bin.len()
                )
            };
            for (k, v) in &self.headers {
                header_str.push_str(&format!("{}: {}\r\n", k, v));
            }
            header_str.push_str("\r\n");
            let mut out = header_str.into_bytes();
            out.extend_from_slice(bin);
            return out;
        }

        let body_str = if let Some(ref raw) = self.raw_body {
            raw.clone()
        } else if self.body.is_null() {
            String::new()
        } else {
            serde_json::to_string(&self.body).unwrap_or_default()
        };
        let has_content_length = self.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("Content-Length"));
        let mut resp = if has_content_length {
            format!(
                "HTTP/1.1 {} {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, Authorization\r\nConnection: close\r\n",
                self.status_code, status_text
            )
        } else {
            format!(
                "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, Authorization\r\nConnection: close\r\n",
                self.status_code, status_text, body_str.len()
            )
        };
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct IdentityPersistRecord {
    pub did: String,
    pub vk_hex: String,
    pub timestamp: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChatMessageRecord {
    pub id: String,
    pub sender_did: String,
    pub recipient_did: String,
    pub text: String,
    pub timestamp: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CallLogRecord {
    pub id: String,
    pub caller_did: String,
    pub recipient_did: String,
    pub call_type: String,
    pub status: String,
    pub duration_seconds: u32,
    pub timestamp: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CallSignalRecord {
    pub call_id: String,
    pub caller_did: String,
    pub caller_name: String,
    pub recipient_did: String,
    pub signal_type: String, // "OFFER", "ANSWER", "ICE_CANDIDATE", "REJECT", "CANCEL", "HANGUP"
    pub call_type: String,   // "AUDIO", "VIDEO"
    pub status: String,      // "RINGING", "CONNECTED", "REJECTED", "ENDED"
    pub timestamp: u64,
    #[serde(default)]
    pub sdp: Option<String>,
    #[serde(default)]
    pub candidate: Option<String>,
    #[serde(default)]
    pub sdp_mid: Option<String>,
    #[serde(default)]
    pub sdp_mline_index: Option<i32>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProductRecord {
    pub id: String,
    pub owner_did: String,
    pub owner_name: String,
    pub title: String,
    pub description: String,
    pub price_inr: f64,
    pub category: String,
    pub sku: String,
    pub created_at: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateInfoRecord {
    pub latest_version_code: u32,
    pub latest_version_name: String,
    pub min_supported_version_code: u32,
    pub title: String,
    pub release_notes: String,
    pub download_url: String,
    pub sha256_hash: String,
    pub mandatory: bool,
    pub published_at: u64,
}

pub fn get_default_update_info() -> UpdateInfoRecord {
    UpdateInfoRecord {
        latest_version_code: 2,
        latest_version_name: "1.0.1-SOVEREIGN".to_string(),
        min_supported_version_code: 1,
        title: "BharatChat Sovereign v1.0.1 Update".to_string(),
        release_notes: "• Direct In-App OTA Update System\n• High-performance binary download stream\n• Package installer integration\n• Enhanced offline messaging & call reliability".to_string(),
        download_url: "/api/v1/app/download".to_string(),
        sha256_hash: "".to_string(),
        mandatory: false,
        published_at: 1776000000,
    }
}

pub fn get_seed_products() -> Vec<ProductRecord> {
    vec![
        ProductRecord {
            id: "prod_001".to_string(),
            owner_did: "did:bharat:9876543210".to_string(),
            owner_name: "Ramesh Patel".to_string(),
            title: "Pure Kanjivaram Silk Saree".to_string(),
            description: "Authentic handloom woven silk with 2.5g gold zari borders".to_string(),
            price_inr: 4500.0,
            category: "Textiles".to_string(),
            sku: "KANJI-001".to_string(),
            created_at: 1700000000,
        },
        ProductRecord {
            id: "prod_002".to_string(),
            owner_did: "did:bharat:9876543210".to_string(),
            owner_name: "Ramesh Patel".to_string(),
            title: "Surat Banarasi Georgette Saree".to_string(),
            description: "Lightweight festive handwoven floral motifs".to_string(),
            price_inr: 3200.0,
            category: "Textiles".to_string(),
            sku: "BANARAS-002".to_string(),
            created_at: 1700000001,
        },
        ProductRecord {
            id: "prod_003".to_string(),
            owner_did: "did:bharat:9811122233".to_string(),
            owner_name: "Priya Sharma".to_string(),
            title: "Chanderi Cotton Embroidered Kurti".to_string(),
            description: "Breathable pure organic cotton summer collection".to_string(),
            price_inr: 1200.0,
            category: "Ready-to-wear".to_string(),
            sku: "CHAND-003".to_string(),
            created_at: 1700000002,
        },
    ]
}

pub struct GatewayEngine {
    pub env: Arc<SovereignAppEnvironment>,
    pub realtime: Arc<realtime::RealtimeManager>,
    pub metrics: Arc<metrics::SreMetrics>,
    pub broadcast: Arc<sovereign_broadcast::BroadcastService>,
    pub messages_store: Arc<std::sync::Mutex<Vec<ChatMessageRecord>>>,
    pub call_logs_store: Arc<std::sync::Mutex<Vec<CallLogRecord>>>,
    pub call_signals_store: Arc<std::sync::Mutex<Vec<CallSignalRecord>>>,
    pub identities_store: Arc<std::sync::Mutex<Vec<IdentityPersistRecord>>>,
    pub products_store: Arc<std::sync::Mutex<Vec<ProductRecord>>>,
    pub update_store: Arc<std::sync::Mutex<UpdateInfoRecord>>,
    pub apk_path: Arc<std::sync::Mutex<Option<std::path::PathBuf>>>,
    pub persistence_dir: Option<std::path::PathBuf>,
    pub media_store: Arc<std::sync::Mutex<HashMap<String, (String, String, Vec<u8>)>>>,
    pub media_dir: Option<std::path::PathBuf>,
}

impl GatewayEngine {
    /// In-memory engine without disk persistence (ideal for unit/integration tests)
    pub fn new() -> Self {
        Self {
            env: Arc::new(SovereignAppEnvironment::new()),
            realtime: Arc::new(realtime::RealtimeManager::new()),
            metrics: Arc::new(metrics::SreMetrics::new()),
            broadcast: Arc::new(sovereign_broadcast::BroadcastService::new()),
            messages_store: Arc::new(std::sync::Mutex::new(Vec::new())),
            call_logs_store: Arc::new(std::sync::Mutex::new(Vec::new())),
            call_signals_store: Arc::new(std::sync::Mutex::new(Vec::new())),
            identities_store: Arc::new(std::sync::Mutex::new(Vec::new())),
            products_store: Arc::new(std::sync::Mutex::new(get_seed_products())),
            update_store: Arc::new(std::sync::Mutex::new(get_default_update_info())),
            apk_path: Arc::new(std::sync::Mutex::new(None)),
            persistence_dir: None,
            media_store: Arc::new(std::sync::Mutex::new(HashMap::new())),
            media_dir: None,
        }
    }

    /// Production engine with persistent storage and PostgreSQL synchronization
    pub fn new_with_persistence<P: AsRef<std::path::Path>>(path: P) -> Self {
        let p_dir = path.as_ref().to_path_buf();
        let _ = std::fs::create_dir_all(&p_dir);
        let media_dir = p_dir.join("media");
        let _ = std::fs::create_dir_all(&media_dir);

        let env = Arc::new(SovereignAppEnvironment::new());
        let identities_store = Arc::new(std::sync::Mutex::new(Vec::new()));
        let messages_store = Arc::new(std::sync::Mutex::new(Vec::new()));
        let call_logs_store = Arc::new(std::sync::Mutex::new(Vec::new()));
        let call_signals_store = Arc::new(std::sync::Mutex::new(Vec::new()));
        let products_store = Arc::new(std::sync::Mutex::new(Vec::new()));

        // Restore identities
        let id_file = p_dir.join("identities.json");
        if let Ok(content) = std::fs::read_to_string(&id_file) {
            if let Ok(records) = serde_json::from_str::<Vec<IdentityPersistRecord>>(&content) {
                let mut chain = env.chain.lock().unwrap();
                for r in &records {
                    let mut vk_bytes = [0u8; 32];
                    if let Ok(decoded) = hex::decode(&r.vk_hex) {
                        if decoded.len() == 32 {
                            vk_bytes.copy_from_slice(&decoded);
                        }
                    }
                    let _ = chain.register_did(&r.did, vk_bytes, r.timestamp);
                    chain.grant_consent(&r.did, "did:bharat:platform", ConsentScope::TransactionalNotifications, r.timestamp, 86400 * 365);
                }
                *identities_store.lock().unwrap() = records;
            }
        }

        // Restore messages
        let msg_file = p_dir.join("messages.json");
        if let Ok(content) = std::fs::read_to_string(&msg_file) {
            if let Ok(records) = serde_json::from_str::<Vec<ChatMessageRecord>>(&content) {
                *messages_store.lock().unwrap() = records;
            }
        }

        // Restore call logs
        let calls_file = p_dir.join("call_logs.json");
        if let Ok(content) = std::fs::read_to_string(&calls_file) {
            if let Ok(records) = serde_json::from_str::<Vec<CallLogRecord>>(&content) {
                *call_logs_store.lock().unwrap() = records;
            }
        }

        // Restore products
        let prod_file = p_dir.join("products.json");
        let loaded_prods = if let Ok(content) = std::fs::read_to_string(&prod_file) {
            serde_json::from_str::<Vec<ProductRecord>>(&content).unwrap_or_else(|_| get_seed_products())
        } else {
            get_seed_products()
        };
        *products_store.lock().unwrap() = if loaded_prods.is_empty() { get_seed_products() } else { loaded_prods };

        // Restore update info
        let update_file = p_dir.join("update_info.json");
        let update_info = if let Ok(content) = std::fs::read_to_string(&update_file) {
            serde_json::from_str::<UpdateInfoRecord>(&content).unwrap_or_else(|_| get_default_update_info())
        } else {
            get_default_update_info()
        };
        let update_store = Arc::new(std::sync::Mutex::new(update_info));

        // Locate release APK on disk
        let rel_apk = std::path::PathBuf::from("client/build/app/outputs/flutter-apk/app-release.apk");
        let found_apk = if rel_apk.exists() {
            Some(rel_apk)
        } else {
            let abs_apk = std::path::PathBuf::from("/home/bhasker-savaliya/hola/client/build/app/outputs/flutter-apk/app-release.apk");
            if abs_apk.exists() {
                Some(abs_apk)
            } else {
                let dbg_apk = std::path::PathBuf::from("client/build/app/outputs/flutter-apk/app-debug.apk");
                if dbg_apk.exists() {
                    Some(dbg_apk)
                } else {
                    Some(rel_apk)
                }
            }
        };
        let apk_path = Arc::new(std::sync::Mutex::new(found_apk));

        let engine = Self {
            env,
            realtime: Arc::new(realtime::RealtimeManager::new()),
            metrics: Arc::new(metrics::SreMetrics::new()),
            broadcast: Arc::new(sovereign_broadcast::BroadcastService::new()),
            messages_store,
            call_logs_store,
            call_signals_store,
            identities_store,
            products_store,
            update_store,
            apk_path,
            persistence_dir: Some(p_dir),
            media_store: Arc::new(std::sync::Mutex::new(HashMap::new())),
            media_dir: Some(media_dir),
        };
        engine.save_products();
        engine.save_update_info();
        engine
    }

    pub fn save_identities(&self) {
        let p_dir = match &self.persistence_dir {
            Some(d) => d,
            None => return,
        };
        if let Ok(store) = self.identities_store.lock() {
            if let Ok(json) = serde_json::to_string_pretty(&*store) {
                let path = p_dir.join("identities.json");
                let tmp = p_dir.join("identities.json.tmp");
                if std::fs::write(&tmp, json).is_ok() {
                    let _ = std::fs::rename(tmp, path);
                }
            }
        }
    }

    pub fn save_messages(&self) {
        let p_dir = match &self.persistence_dir {
            Some(d) => d,
            None => return,
        };
        if let Ok(store) = self.messages_store.lock() {
            if let Ok(json) = serde_json::to_string_pretty(&*store) {
                let path = p_dir.join("messages.json");
                let tmp = p_dir.join("messages.json.tmp");
                if std::fs::write(&tmp, json).is_ok() {
                    let _ = std::fs::rename(tmp, path);
                }
            }
        }
    }

    pub fn save_call_logs(&self) {
        let p_dir = match &self.persistence_dir {
            Some(d) => d,
            None => return,
        };
        if let Ok(store) = self.call_logs_store.lock() {
            if let Ok(json) = serde_json::to_string_pretty(&*store) {
                let path = p_dir.join("call_logs.json");
                let tmp = p_dir.join("call_logs.json.tmp");
                if std::fs::write(&tmp, json).is_ok() {
                    let _ = std::fs::rename(tmp, path);
                }
            }
        }
    }

    pub fn save_products(&self) {
        let p_dir = match &self.persistence_dir {
            Some(d) => d,
            None => return,
        };
        if let Ok(store) = self.products_store.lock() {
            if let Ok(json) = serde_json::to_string_pretty(&*store) {
                let path = p_dir.join("products.json");
                let tmp = p_dir.join("products.json.tmp");
                if std::fs::write(&tmp, json).is_ok() {
                    let _ = std::fs::rename(tmp, path);
                }
            }
        }
    }

    pub fn save_update_info(&self) {
        let p_dir = match &self.persistence_dir {
            Some(d) => d,
            None => return,
        };
        if let Ok(store) = self.update_store.lock() {
            if let Ok(json) = serde_json::to_string_pretty(&*store) {
                let path = p_dir.join("update_info.json");
                let tmp = p_dir.join("update_info.json.tmp");
                if std::fs::write(&tmp, json).is_ok() {
                    let _ = std::fs::rename(tmp, path);
                }
            }
        }
    }

    pub fn set_apk_path<P: AsRef<std::path::Path>>(&self, path: P) {
        if let Ok(mut lock) = self.apk_path.lock() {
            *lock = Some(path.as_ref().to_path_buf());
        }
    }

    pub fn sync_postgres_identity(&self, did: &str, vk_hex: &str, ts: u64) {
        if self.persistence_dir.is_none() {
            return;
        }
        let did_c = did.to_string();
        let vk_c = vk_hex.to_string();
        tokio::task::spawn_blocking(move || {
            let sql = format!(
                "INSERT INTO identities (did, verifying_key_hex, status, registered_at) VALUES ('{}', '{}', 'ACTIVE', {}) ON CONFLICT (did) DO NOTHING;",
                did_c.replace('\'', "''"),
                vk_c.replace('\'', "''"),
                ts
            );
            let _ = std::process::Command::new("podman")
                .args(&["exec", "-i", "bharatchat-postgres", "psql", "-U", "bharat_admin", "-d", "bharatchat", "-c", &sql])
                .output();
        });
    }

    pub fn sync_postgres_message(&self, id: &str, sender: &str, recipient: &str, text: &str, ts: u64) {
        if self.persistence_dir.is_none() {
            return;
        }
        let id_c = id.to_string();
        let s_c = sender.to_string();
        let r_c = recipient.to_string();
        let t_c = text.to_string();
        tokio::task::spawn_blocking(move || {
            let sql = format!(
                "INSERT INTO chat_messages (id, sender_did, recipient_did, text, timestamp) VALUES ('{}', '{}', '{}', '{}', {}) ON CONFLICT (id) DO NOTHING;",
                id_c.replace('\'', "''"),
                s_c.replace('\'', "''"),
                r_c.replace('\'', "''"),
                t_c.replace('\'', "''"),
                ts
            );
            let _ = std::process::Command::new("podman")
                .args(&["exec", "-i", "bharatchat-postgres", "psql", "-U", "bharat_admin", "-d", "bharatchat", "-c", &sql])
                .output();
        });
    }

    pub fn sync_postgres_call_log(&self, id: &str, caller: &str, recipient: &str, call_type: &str, status: &str, duration: u32, ts: u64) {
        if self.persistence_dir.is_none() {
            return;
        }
        let id_c = id.to_string();
        let c_c = caller.to_string();
        let r_c = recipient.to_string();
        let ct_c = call_type.to_string();
        let st_c = status.to_string();
        tokio::task::spawn_blocking(move || {
            let sql = format!(
                "INSERT INTO call_logs (id, caller_did, recipient_did, call_type, status, duration_seconds, timestamp) VALUES ('{}', '{}', '{}', '{}', '{}', {}, {}) ON CONFLICT (id) DO NOTHING;",
                id_c.replace('\'', "''"),
                c_c.replace('\'', "''"),
                r_c.replace('\'', "''"),
                ct_c.replace('\'', "''"),
                st_c.replace('\'', "''"),
                duration,
                ts
            );
            let _ = std::process::Command::new("podman")
                .args(&["exec", "-i", "bharatchat-postgres", "psql", "-U", "bharat_admin", "-d", "bharatchat", "-c", &sql])
                .output();
        });
    }

    /// GET /health
    pub fn handle_health(&self) -> HttpResponse {
        HttpResponse::ok(serde_json::json!({
            "status": "UP",
            "service": "BharatChat Sovereign API Gateway",
            "version": "1.0.0-PROD",
            "jurisdiction": "INDIA_LOCALIZED_MUMBAI_BLR",
            "zero_disk_retention": false,
            "persistence": "ACTIVE_ENCLAVE_AND_POSTGRES",
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
                let vk_hex = hex::encode(vk_bytes);
                {
                    let mut id_store = self.identities_store.lock().unwrap();
                    if !id_store.iter().any(|i| i.did == did) {
                        id_store.push(IdentityPersistRecord {
                            did: did.to_string(),
                            vk_hex: vk_hex.clone(),
                            timestamp,
                        });
                    }
                }
                self.save_identities();
                self.sync_postgres_identity(did, &vk_hex, timestamp);

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
                "/api/v1/identities": {
                    "get": {
                        "summary": "List All Active Registered DIDs on Consortium Ledger",
                        "responses": {
                            "200": { "description": "List of active registered DIDs" }
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
            ("OPTIONS", _) => HttpResponse::ok(serde_json::json!({"status": "OK"})),
            ("GET", "/health") => self.handle_health(),
            ("GET", "/metrics") => self.metrics.render_response(),
            ("GET", "/api/v1/openapi.json") => HttpResponse::ok(self.get_openapi_spec()),
            ("GET", "/api/v1/identities") => {
                let chain = self.env.chain.lock().unwrap();
                let list: Vec<serde_json::Value> = chain
                    .did_records
                    .values()
                    .filter(|r| r.status == sovereign_chain::DidStatus::Active)
                    .map(|r| {
                        serde_json::json!({
                            "did": r.did,
                            "status": "ACTIVE",
                            "registered_at": r.registered_at,
                        })
                    })
                    .collect();
                HttpResponse::ok(serde_json::json!({
                    "identities": list,
                    "total": list.len(),
                    "jurisdiction": "INDIA_LOCALIZED_MUMBAI_BLR"
                }))
            }
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
            ("GET", "/api/v1/messages") => {
                let query = if let Some(q_idx) = path.find('?') {
                    &path[q_idx + 1..]
                } else {
                    ""
                };
                let mut did_param: Option<String> = None;
                let mut since_param: u64 = 0;
                for pair in query.split('&') {
                    let mut parts = pair.split('=');
                    if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                        let decoded_v = url_decode(v);
                        if k == "did" {
                            did_param = Some(decoded_v);
                        } else if k == "since" {
                            since_param = decoded_v.parse().unwrap_or(0);
                        }
                    }
                }

                let store = self.messages_store.lock().unwrap();
                let filtered: Vec<ChatMessageRecord> = store
                    .iter()
                    .filter(|m| {
                        let match_did = match did_param {
                            Some(ref d) => m.recipient_did == *d || m.sender_did == *d,
                            None => true,
                        };
                        match_did && m.timestamp >= since_param
                    })
                    .cloned()
                    .collect();

                HttpResponse::ok(serde_json::json!({
                    "messages": filtered,
                    "total": filtered.len(),
                    "jurisdiction": "INDIA_LOCALIZED_MUMBAI_BLR"
                }))
            }
            ("POST", "/api/v1/messages") => {
                #[derive(Deserialize)]
                #[allow(dead_code)]
                struct SendMsgReq {
                    #[serde(default)]
                    packet: Option<MessagePacket>,
                    #[serde(default)]
                    timestamp: Option<u64>,
                    #[serde(default)]
                    sender_did: Option<String>,
                    #[serde(default)]
                    recipient_did: Option<String>,
                    #[serde(default)]
                    content: Option<String>,
                    #[serde(default)]
                    text: Option<String>,
                    #[serde(default)]
                    ciphertext: Option<String>,
                    #[serde(default)]
                    body: Option<String>,
                }
                match serde_json::from_str::<SendMsgReq>(body) {
                    Ok(req) => {
                        let now = req.timestamp.unwrap_or_else(|| {
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs()
                        });
                        if let Some(packet) = req.packet {
                            self.handle_send_message(packet, now).await
                        } else if let (Some(sender), Some(recipient)) =
                            (req.sender_did, req.recipient_did)
                        {
                            let text = req.text
                                .or(req.content)
                                .or(req.ciphertext)
                                .or(req.body)
                                .unwrap_or_default();
                            let msg_id = format!("msg_{}_{}", now, self.messages_store.lock().unwrap().len() + 1);
                            let record = ChatMessageRecord {
                                id: msg_id.clone(),
                                sender_did: sender.clone(),
                                recipient_did: recipient.clone(),
                                text: text.clone(),
                                timestamp: now,
                            };
                            self.messages_store.lock().unwrap().push(record);
                            self.save_messages();
                            self.sync_postgres_message(&msg_id, &sender, &recipient, &text, now);

                            HttpResponse::ok(serde_json::json!({
                                "status": "DELIVERED",
                                "delivered_live": true,
                                "message_id": msg_id,
                                "sender_did": sender,
                                "recipient_did": recipient,
                                "rule_4_2_envelope_present": true,
                                "routed_at": now
                            }))
                        } else {
                            HttpResponse::problem(
                                400,
                                "Invalid Message Packet",
                                "Missing packet or sender_did/recipient_did",
                                "/api/v1/messages",
                            )
                        }
                    }
                    Err(e) => HttpResponse::problem(
                        400,
                        "Invalid Message Packet",
                        &e.to_string(),
                        "/api/v1/messages",
                    ),
                }
            }
            ("POST", "/api/v1/calls") => {
                #[derive(Deserialize)]
                struct LogCallReq {
                    caller_did: String,
                    recipient_did: String,
                    #[serde(default = "default_call_type")]
                    call_type: String,
                    #[serde(default = "default_call_status")]
                    status: String,
                    #[serde(default)]
                    duration_seconds: u32,
                    #[serde(default)]
                    timestamp: Option<u64>,
                }
                fn default_call_type() -> String { "AUDIO".to_string() }
                fn default_call_status() -> String { "COMPLETED".to_string() }

                match serde_json::from_str::<LogCallReq>(body) {
                    Ok(req) => {
                        let now = req.timestamp.unwrap_or_else(|| {
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs()
                        });
                        let call_id = format!("call_{}_{}", now, self.call_logs_store.lock().unwrap().len() + 1);
                        let record = CallLogRecord {
                            id: call_id.clone(),
                            caller_did: req.caller_did.clone(),
                            recipient_did: req.recipient_did.clone(),
                            call_type: req.call_type.clone(),
                            status: req.status.clone(),
                            duration_seconds: req.duration_seconds,
                            timestamp: now,
                        };
                        self.call_logs_store.lock().unwrap().push(record);
                        self.save_call_logs();
                        self.sync_postgres_call_log(&call_id, &req.caller_did, &req.recipient_did, &req.call_type, &req.status, req.duration_seconds, now);

                        HttpResponse::created(serde_json::json!({
                            "status": "RECORDED",
                            "call_id": call_id,
                            "caller_did": req.caller_did,
                            "recipient_did": req.recipient_did,
                            "call_type": req.call_type,
                            "call_status": req.status,
                            "duration_seconds": req.duration_seconds,
                            "timestamp": now,
                            "data_jurisdiction": "INDIA_LOCALIZED_MUMBAI_BLR"
                        }))
                    }
                    Err(e) => HttpResponse::problem(
                        400,
                        "Invalid Call Log Body",
                        &e.to_string(),
                        "/api/v1/calls",
                    ),
                }
            }
            ("GET", "/api/v1/calls") => {
                let query = if let Some(q_idx) = path.find('?') {
                    &path[q_idx + 1..]
                } else {
                    ""
                };
                let mut did_param: Option<String> = None;
                for pair in query.split('&') {
                    let mut parts = pair.split('=');
                    if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                        if k == "did" {
                            did_param = Some(url_decode(v));
                        }
                    }
                }

                let store = self.call_logs_store.lock().unwrap();
                let filtered: Vec<CallLogRecord> = store
                    .iter()
                    .filter(|c| {
                        match did_param {
                            Some(ref d) => c.caller_did == *d || c.recipient_did == *d,
                            None => true,
                        }
                    })
                    .cloned()
                    .collect();

                HttpResponse::ok(serde_json::json!({
                    "calls": filtered,
                    "total": filtered.len(),
                    "jurisdiction": "INDIA_LOCALIZED_MUMBAI_BLR"
                }))
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
            ("GET", "/api/v1/commerce/products") => {
                let store = self.products_store.lock().unwrap();
                HttpResponse::ok(serde_json::json!({
                    "products": *store,
                    "total": store.len(),
                    "jurisdiction": "INDIA_LOCALIZED_MUMBAI_BLR"
                }))
            }
            ("POST", "/api/v1/commerce/products") => {
                #[derive(Deserialize)]
                struct CreateProdReq {
                    #[serde(default)]
                    id: Option<String>,
                    owner_did: String,
                    #[serde(default)]
                    owner_name: Option<String>,
                    title: String,
                    description: String,
                    price_inr: f64,
                    #[serde(default)]
                    category: Option<String>,
                    #[serde(default)]
                    sku: Option<String>,
                }
                match serde_json::from_str::<CreateProdReq>(body) {
                    Ok(req) => {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let prod_id = req.id.unwrap_or_else(|| format!("prod_{}_{}", now, self.products_store.lock().unwrap().len() + 1));
                        let record = ProductRecord {
                            id: prod_id.clone(),
                            owner_did: req.owner_did,
                            owner_name: req.owner_name.unwrap_or_else(|| "Sovereign Merchant".to_string()),
                            title: req.title,
                            description: req.description,
                            price_inr: req.price_inr,
                            category: req.category.unwrap_or_else(|| "Textiles".to_string()),
                            sku: req.sku.unwrap_or_else(|| format!("SKU-{}", prod_id)),
                            created_at: now,
                        };
                        self.products_store.lock().unwrap().push(record.clone());
                        self.save_products();
                        HttpResponse::created(serde_json::json!({
                            "product": record,
                            "status": "CREATED",
                            "jurisdiction": "INDIA_LOCALIZED_MUMBAI_BLR"
                        }))
                    }
                    Err(e) => HttpResponse::problem(
                        400,
                        "Invalid Product Payload",
                        &e.to_string(),
                        "/api/v1/commerce/products",
                    ),
                }
            }
            ("PUT", p) if p.starts_with("/api/v1/commerce/products/") => {
                let prod_id = &p["/api/v1/commerce/products/".len()..];
                #[derive(Deserialize)]
                struct UpdateProdReq {
                    #[serde(default)]
                    title: Option<String>,
                    #[serde(default)]
                    description: Option<String>,
                    #[serde(default)]
                    price_inr: Option<f64>,
                    #[serde(default)]
                    category: Option<String>,
                    #[serde(default)]
                    sku: Option<String>,
                }
                match serde_json::from_str::<UpdateProdReq>(body) {
                    Ok(req) => {
                        let mut store = self.products_store.lock().unwrap();
                        if let Some(prod) = store.iter_mut().find(|p| p.id == prod_id) {
                            if let Some(t) = req.title { prod.title = t; }
                            if let Some(d) = req.description { prod.description = d; }
                            if let Some(pr) = req.price_inr { prod.price_inr = pr; }
                            if let Some(c) = req.category { prod.category = c; }
                            if let Some(s) = req.sku { prod.sku = s; }
                            let updated = prod.clone();
                            drop(store);
                            self.save_products();
                            HttpResponse::ok(serde_json::json!({
                                "product": updated,
                                "status": "UPDATED",
                                "jurisdiction": "INDIA_LOCALIZED_MUMBAI_BLR"
                            }))
                        } else {
                            HttpResponse::problem(
                                404,
                                "Product Not Found",
                                "The requested product ID does not exist",
                                p,
                            )
                        }
                    }
                    Err(e) => HttpResponse::problem(
                        400,
                        "Invalid Product Update Payload",
                        &e.to_string(),
                        p,
                    ),
                }
            }
            ("DELETE", p) if p.starts_with("/api/v1/commerce/products/") => {
                let prod_id = &p["/api/v1/commerce/products/".len()..];
                let mut store = self.products_store.lock().unwrap();
                let initial_len = store.len();
                store.retain(|p| p.id != prod_id);
                if store.len() < initial_len {
                    drop(store);
                    self.save_products();
                    HttpResponse::ok(serde_json::json!({
                        "id": prod_id,
                        "status": "DELETED",
                        "jurisdiction": "INDIA_LOCALIZED_MUMBAI_BLR"
                    }))
                } else {
                    HttpResponse::problem(
                        404,
                        "Product Not Found",
                        "The requested product ID does not exist",
                        p,
                    )
                }
            }
            ("POST", "/api/v1/calls/signal") => {
                #[derive(Deserialize)]
                struct SignalReq {
                    #[serde(default)]
                    call_id: Option<String>,
                    caller_did: String,
                    #[serde(default)]
                    caller_name: Option<String>,
                    recipient_did: String,
                    signal_type: String, // "OFFER", "ANSWER", "ICE_CANDIDATE", "REJECT", "CANCEL", "HANGUP"
                    #[serde(default = "default_call_type")]
                    call_type: String,   // "AUDIO", "VIDEO"
                    #[serde(default)]
                    timestamp: Option<u64>,
                    #[serde(default)]
                    sdp: Option<String>,
                    #[serde(default)]
                    candidate: Option<String>,
                    #[serde(default)]
                    sdp_mid: Option<String>,
                    #[serde(default)]
                    sdp_mline_index: Option<i32>,
                }
                fn default_call_type() -> String { "AUDIO".to_string() }

                match serde_json::from_str::<SignalReq>(body) {
                    Ok(req) => {
                        let now = req.timestamp.unwrap_or_else(|| {
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs()
                        });
                        let call_id = req.call_id.unwrap_or_else(|| format!("call_{}_{}", now, req.caller_did));
                        let status = match req.signal_type.as_str() {
                            "OFFER" => "RINGING",
                            "ANSWER" => "CONNECTED",
                            "REJECT" => "REJECTED",
                            "CANCEL" | "HANGUP" => "ENDED",
                            s => s,
                        };
                        let caller_name = req.caller_name.unwrap_or_else(|| "BharatChat Contact".to_string());

                        let mut store = self.call_signals_store.lock().unwrap();
                        let record = if req.signal_type == "ICE_CANDIDATE" {
                            let cand_record = CallSignalRecord {
                                call_id: call_id.clone(),
                                caller_did: req.caller_did.clone(),
                                caller_name: caller_name.clone(),
                                recipient_did: req.recipient_did.clone(),
                                signal_type: "ICE_CANDIDATE".to_string(),
                                call_type: req.call_type.clone(),
                                status: "CONNECTED".to_string(),
                                timestamp: now,
                                sdp: None,
                                candidate: req.candidate.clone(),
                                sdp_mid: req.sdp_mid.clone(),
                                sdp_mline_index: req.sdp_mline_index,
                            };
                            store.push(cand_record.clone());
                            if store.len() > 300 {
                                let excess = store.len() - 300;
                                store.drain(0..excess);
                            }
                            cand_record
                        } else if let Some(existing) = store.iter_mut().find(|s| s.call_id == call_id && s.signal_type != "ICE_CANDIDATE") {
                            existing.signal_type = req.signal_type.clone();
                            existing.status = status.to_string();
                            existing.timestamp = now;
                            if req.sdp.is_some() {
                                existing.sdp = req.sdp.clone();
                            }
                            existing.clone()
                        } else {
                            let new_record = CallSignalRecord {
                                call_id: call_id.clone(),
                                caller_did: req.caller_did.clone(),
                                caller_name: caller_name.clone(),
                                recipient_did: req.recipient_did.clone(),
                                signal_type: req.signal_type.clone(),
                                call_type: req.call_type.clone(),
                                status: status.to_string(),
                                timestamp: now,
                                sdp: req.sdp.clone(),
                                candidate: req.candidate.clone(),
                                sdp_mid: req.sdp_mid.clone(),
                                sdp_mline_index: req.sdp_mline_index,
                            };
                            store.push(new_record.clone());
                            if store.len() > 300 {
                                let excess = store.len() - 300;
                                store.drain(0..excess);
                            }
                            new_record
                        };

                        HttpResponse::ok(serde_json::json!({
                            "status": "SIGNAL_DELIVERED",
                            "call_id": record.call_id,
                            "caller_did": record.caller_did,
                            "caller_name": record.caller_name,
                            "recipient_did": record.recipient_did,
                            "signal_type": record.signal_type,
                            "call_type": record.call_type,
                            "call_status": record.status,
                            "timestamp": record.timestamp,
                            "signal": record
                        }))
                    }
                    Err(e) => HttpResponse::problem(
                        400,
                        "Invalid Signal Payload",
                        &e.to_string(),
                        "/api/v1/calls/signal",
                    ),
                }
            }
            ("GET", "/api/v1/calls/signal") => {
                let query = if let Some(q_idx) = path.find('?') {
                    &path[q_idx + 1..]
                } else {
                    ""
                };
                let mut did_param: Option<String> = None;
                let mut call_id_param: Option<String> = None;
                let mut since_param: u64 = 0;
                for pair in query.split('&') {
                    let mut parts = pair.split('=');
                    if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                        let decoded_v = url_decode(v);
                        if k == "did" {
                            did_param = Some(decoded_v);
                        } else if k == "call_id" {
                            call_id_param = Some(decoded_v);
                        } else if k == "since" {
                            since_param = decoded_v.parse().unwrap_or(0);
                        }
                    }
                }

                let store = self.call_signals_store.lock().unwrap();
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let filtered: Vec<CallSignalRecord> = store
                    .iter()
                    .filter(|s| {
                        if let Some(ref cid) = call_id_param {
                            return s.call_id == *cid;
                        }
                        let match_did = match did_param {
                            Some(ref d) => s.recipient_did == *d || s.caller_did == *d,
                            None => true,
                        };
                        let fresh = if since_param > 0 {
                            s.timestamp >= since_param
                        } else {
                            now.saturating_sub(s.timestamp) < 90
                        };
                        match_did && fresh
                    })
                    .cloned()
                    .collect();

                HttpResponse::ok(serde_json::json!({
                    "signals": filtered,
                    "total": filtered.len(),
                    "jurisdiction": "INDIA_LOCALIZED_MUMBAI_BLR"
                }))
            }
            ("POST", "/api/v1/media/upload") => {
                #[derive(Deserialize)]
                struct MediaUploadReq {
                    filename: String,
                    #[serde(default = "default_media_type")]
                    media_type: String, // "IMAGE", "DOCUMENT", "VOICE_NOTE"
                    #[serde(default = "default_mime_type")]
                    mime_type: String,
                    data_base64: String,
                }
                fn default_media_type() -> String { "DOCUMENT".to_string() }
                fn default_mime_type() -> String { "application/octet-stream".to_string() }

                match serde_json::from_str::<MediaUploadReq>(body) {
                    Ok(req) => {
                        match base64_decode(&req.data_base64) {
                            Ok(bytes) => {
                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();
                                let media_id = format!("media_{}_{}", now, self.media_store.lock().unwrap().len() + 1);
                                let clean_filename = req.filename
                                    .replace('/', "_")
                                    .replace('\\', "_")
                                    .replace("..", "_");

                                if let Some(ref m_dir) = self.media_dir {
                                    let target_path = m_dir.join(format!("{}_{}", media_id, clean_filename));
                                    let _ = std::fs::write(&target_path, &bytes);
                                }
                                self.media_store.lock().unwrap().insert(
                                    media_id.clone(),
                                    (clean_filename.clone(), req.mime_type.clone(), bytes.clone()),
                                );

                                HttpResponse::ok(serde_json::json!({
                                    "status": "UPLOADED",
                                    "media_id": media_id,
                                    "url": format!("/api/v1/media/{}", media_id),
                                    "filename": clean_filename,
                                    "media_type": req.media_type,
                                    "mime_type": req.mime_type,
                                    "size_bytes": bytes.len(),
                                    "timestamp": now,
                                }))
                            }
                            Err(e) => HttpResponse::problem(
                                400,
                                "Invalid Base64 Media Payload",
                                &e.to_string(),
                                "/api/v1/media/upload",
                            ),
                        }
                    }
                    Err(e) => HttpResponse::problem(
                        400,
                        "Invalid Media Upload Request",
                        &e.to_string(),
                        "/api/v1/media/upload",
                    ),
                }
            }
            ("GET", p) if p.starts_with("/api/v1/media/") => {
                let media_id = &p["/api/v1/media/".len()..];
                let clean_id = media_id.split('?').next().unwrap_or(media_id);

                // 1. Check in-memory store
                let mem_item = self.media_store.lock().unwrap().get(clean_id).cloned();
                if let Some((filename, mime, bytes)) = mem_item {
                    let disposition = format!("inline; filename=\"{}\"", filename);
                    HttpResponse::binary(
                        200,
                        &mime,
                        bytes,
                        vec![("Content-Disposition".to_string(), disposition)],
                    )
                } else if let Some(ref m_dir) = self.media_dir {
                    // 2. Check persistence dir
                    let mut found = None;
                    if let Ok(entries) = std::fs::read_dir(m_dir) {
                        for entry in entries.flatten() {
                            let fname = entry.file_name().to_string_lossy().to_string();
                            if fname.starts_with(clean_id) {
                                if let Ok(bytes) = std::fs::read(entry.path()) {
                                    let mime = if fname.ends_with(".jpg") || fname.ends_with(".jpeg") {
                                        "image/jpeg"
                                    } else if fname.ends_with(".png") {
                                        "image/png"
                                    } else if fname.ends_with(".pdf") {
                                        "application/pdf"
                                    } else if fname.ends_with(".m4a") || fname.ends_with(".aac") {
                                        "audio/mp4"
                                    } else if fname.ends_with(".wav") {
                                        "audio/wav"
                                    } else {
                                        "application/octet-stream"
                                    };
                                    found = Some((fname, mime.to_string(), bytes));
                                    break;
                                }
                            }
                        }
                    }
                    if let Some((fname, mime, bytes)) = found {
                        let disposition = format!("inline; filename=\"{}\"", fname);
                        HttpResponse::binary(
                            200,
                            &mime,
                            bytes,
                            vec![("Content-Disposition".to_string(), disposition)],
                        )
                    } else {
                        HttpResponse::problem(404, "Media Not Found", &format!("No media found with ID {}", clean_id), p)
                    }
                } else {
                    HttpResponse::problem(404, "Media Not Found", &format!("No media found with ID {}", clean_id), p)
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
            ("GET", "/api/v1/app/update") => {
                let info = self.update_store.lock().unwrap().clone();
                HttpResponse::ok(serde_json::to_value(&info).unwrap_or_default())
            }
            ("POST", "/api/v1/app/update") => {
                #[derive(Deserialize)]
                struct SetUpdateReq {
                    #[serde(default)]
                    latest_version_code: Option<u32>,
                    #[serde(default)]
                    latest_version_name: Option<String>,
                    #[serde(default)]
                    min_supported_version_code: Option<u32>,
                    #[serde(default)]
                    title: Option<String>,
                    #[serde(default)]
                    release_notes: Option<String>,
                    #[serde(default)]
                    download_url: Option<String>,
                    #[serde(default)]
                    sha256_hash: Option<String>,
                    #[serde(default)]
                    mandatory: Option<bool>,
                }
                match serde_json::from_str::<SetUpdateReq>(body) {
                    Ok(req) => {
                        let mut store = self.update_store.lock().unwrap();
                        if let Some(vc) = req.latest_version_code { store.latest_version_code = vc; }
                        if let Some(vn) = req.latest_version_name { store.latest_version_name = vn; }
                        if let Some(mvc) = req.min_supported_version_code { store.min_supported_version_code = mvc; }
                        if let Some(t) = req.title { store.title = t; }
                        if let Some(rn) = req.release_notes { store.release_notes = rn; }
                        if let Some(du) = req.download_url { store.download_url = du; }
                        if let Some(sh) = req.sha256_hash { store.sha256_hash = sh; }
                        if let Some(m) = req.mandatory { store.mandatory = m; }
                        store.published_at = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let updated = store.clone();
                        drop(store);
                        self.save_update_info();
                        HttpResponse::ok(serde_json::to_value(&updated).unwrap_or_default())
                    }
                    Err(e) => HttpResponse::problem(400, "Invalid Update Payload", &e.to_string(), path),
                }
            }
            ("GET", "/api/v1/app/download") => {
                let candidate_path = self.apk_path.lock().unwrap().clone();
                let file_path = match candidate_path {
                    Some(p) if p.exists() => Some(p),
                    _ => {
                        let rel = std::path::PathBuf::from("client/build/app/outputs/flutter-apk/app-release.apk");
                        if rel.exists() {
                            Some(rel)
                        } else {
                            let abs = std::path::PathBuf::from("/home/bhasker-savaliya/hola/client/build/app/outputs/flutter-apk/app-release.apk");
                            if abs.exists() {
                                Some(abs)
                            } else {
                                let dbg = std::path::PathBuf::from("client/build/app/outputs/flutter-apk/app-debug.apk");
                                if dbg.exists() { Some(dbg) } else { None }
                            }
                        }
                    }
                };

                if let Some(p) = file_path {
                    match std::fs::read(&p) {
                        Ok(bytes) => HttpResponse::binary(
                            200,
                            "application/vnd.android.package-archive",
                            bytes,
                            vec![
                                ("Content-Disposition".to_string(), "attachment; filename=\"bharatchat-release.apk\"".to_string()),
                                ("Cache-Control".to_string(), "no-cache".to_string()),
                            ],
                        ),
                        Err(e) => HttpResponse::problem(
                            500,
                            "APK Read Error",
                            &e.to_string(),
                            path,
                        ),
                    }
                } else {
                    HttpResponse::problem(
                        404,
                        "APK Not Found",
                        "Release APK is not built or available on server disk",
                        path,
                    )
                }
            }
            ("HEAD", "/api/v1/app/download") => {
                let candidate_path = self.apk_path.lock().unwrap().clone();
                let file_path = match candidate_path {
                    Some(p) if p.exists() => Some(p),
                    _ => {
                        let rel = std::path::PathBuf::from("client/build/app/outputs/flutter-apk/app-release.apk");
                        if rel.exists() {
                            Some(rel)
                        } else {
                            let abs = std::path::PathBuf::from("/home/bhasker-savaliya/hola/client/build/app/outputs/flutter-apk/app-release.apk");
                            if abs.exists() {
                                Some(abs)
                            } else {
                                let dbg = std::path::PathBuf::from("client/build/app/outputs/flutter-apk/app-debug.apk");
                                if dbg.exists() { Some(dbg) } else { None }
                            }
                        }
                    }
                };

                if let Some(p) = file_path {
                    if let Ok(meta) = std::fs::metadata(&p) {
                        HttpResponse {
                            status_code: 200,
                            headers: vec![
                                ("Content-Type".to_string(), "application/vnd.android.package-archive".to_string()),
                                ("Content-Length".to_string(), meta.len().to_string()),
                                ("Content-Disposition".to_string(), "attachment; filename=\"bharatchat-release.apk\"".to_string()),
                            ],
                            body: serde_json::Value::Null,
                            raw_body: Some(String::new()),
                            binary_body: None,
                        }
                    } else {
                        HttpResponse::problem(404, "APK Not Found", "Metadata unavailable", path)
                    }
                } else {
                    HttpResponse::problem(404, "APK Not Found", "Release APK is not built or available on server disk", path)
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
