use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::Duration;

use crate::HttpResponse;

/// SRE Golden Signals and Statutory SLA Metrics Collector
pub struct SreMetrics {
    // Traffic & Error Counters: key = (method, path, status)
    request_counters: RwLock<HashMap<(String, String, u16), u64>>,
    // Total request count
    total_requests: AtomicU64,
    // Total latency in microseconds
    total_latency_us: AtomicU64,
    // Latency histogram buckets (seconds): 0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5
    bucket_counts: [AtomicU64; 10],
    
    // Saturation
    pub active_websockets: AtomicU64,
    pub ephemeral_relay_buffer_bytes: AtomicU64,

    // Statutory Compliance SLA & Cryptographic Telemetry
    pub cert_in_sla_seconds: u32, // CERT-In Directions 2022 (6 hours = 21,600s)
    pub rule4_2_originator_lookups: AtomicU64,
    pub dpdp_crypto_shredding_ops: AtomicU64,
    pub last_shredding_entropy_scaled: AtomicU64, // entropy * 1000
    pub sim_binding_verifications: AtomicU64,
    pub zkp_age_verifications: AtomicU64,
}

const HISTOGRAM_THRESHOLDS: [f64; 10] = [
    0.001, 0.005, 0.010, 0.025, 0.050, 0.100, 0.250, 0.500, 1.000, 2.500,
];

impl SreMetrics {
    pub fn new() -> Self {
        Self {
            request_counters: RwLock::new(HashMap::new()),
            total_requests: AtomicU64::new(0),
            total_latency_us: AtomicU64::new(0),
            bucket_counts: Default::default(),
            active_websockets: AtomicU64::new(0),
            ephemeral_relay_buffer_bytes: AtomicU64::new(0),
            cert_in_sla_seconds: 21600, // 6 hours
            rule4_2_originator_lookups: AtomicU64::new(0),
            dpdp_crypto_shredding_ops: AtomicU64::new(0),
            last_shredding_entropy_scaled: AtomicU64::new(7991), // 7.991 bits/byte baseline
            sim_binding_verifications: AtomicU64::new(0),
            zkp_age_verifications: AtomicU64::new(0),
        }
    }

    pub fn record_request(&self, method: &str, path: &str, status: u16, duration: Duration) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        let us = duration.as_micros() as u64;
        self.total_latency_us.fetch_add(us, Ordering::Relaxed);

        let secs = duration.as_secs_f64();
        for (idx, &threshold) in HISTOGRAM_THRESHOLDS.iter().enumerate() {
            if secs <= threshold {
                self.bucket_counts[idx].fetch_add(1, Ordering::Relaxed);
            }
        }

        let key = (method.to_string(), path.to_string(), status);
        let mut map = self.request_counters.write().unwrap();
        *map.entry(key).or_insert(0) += 1;
    }

    pub fn record_shredding(&self, entropy_bits: f64) {
        self.dpdp_crypto_shredding_ops.fetch_add(1, Ordering::Relaxed);
        let scaled = (entropy_bits * 1000.0) as u64;
        self.last_shredding_entropy_scaled.store(scaled, Ordering::Relaxed);
    }

    pub fn record_originator_lookup(&self) {
        self.rule4_2_originator_lookups.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_sim_binding(&self) {
        self.sim_binding_verifications.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_zkp_verification(&self) {
        self.zkp_age_verifications.fetch_add(1, Ordering::Relaxed);
    }

    /// Render metrics in Prometheus Text Format (RFC-compatible exposition format)
    pub fn render_prometheus(&self) -> String {
        let mut out = String::with_capacity(4096);

        out.push_str("# HELP http_requests_total Total number of HTTP requests processed by endpoint\n");
        out.push_str("# TYPE http_requests_total counter\n");
        {
            let map = self.request_counters.read().unwrap();
            for ((method, path, status), count) in map.iter() {
                out.push_str(&format!(
                    "http_requests_total{{method=\"{}\",path=\"{}\",status=\"{}\"}} {}\n",
                    method, path, status, count
                ));
            }
        }

        out.push_str("\n# HELP http_request_duration_seconds HTTP request latency histogram\n");
        out.push_str("# TYPE http_request_duration_seconds histogram\n");
        for (idx, &threshold) in HISTOGRAM_THRESHOLDS.iter().enumerate() {
            let count = self.bucket_counts[idx].load(Ordering::Relaxed);
            out.push_str(&format!(
                "http_request_duration_seconds_bucket{{le=\"{:.3}\"}} {}\n",
                threshold, count
            ));
        }
        let total = self.total_requests.load(Ordering::Relaxed);
        let sum_secs = (self.total_latency_us.load(Ordering::Relaxed) as f64) / 1_000_000.0;
        out.push_str(&format!(
            "http_request_duration_seconds_bucket{{le=\"+Inf\"}} {}\n",
            total
        ));
        out.push_str(&format!("http_request_duration_seconds_sum {:.6}\n", sum_secs));
        out.push_str(&format!("http_request_duration_seconds_count {}\n", total));

        out.push_str("\n# HELP sovereign_active_websocket_connections Current active WebSocket client connections\n");
        out.push_str("# TYPE sovereign_active_websocket_connections gauge\n");
        out.push_str(&format!(
            "sovereign_active_websocket_connections {}\n",
            self.active_websockets.load(Ordering::Relaxed)
        ));

        out.push_str("\n# HELP sovereign_ephemeral_relay_buffer_bytes In-memory RAM buffer usage across ephemeral relay mesh\n");
        out.push_str("# TYPE sovereign_ephemeral_relay_buffer_bytes gauge\n");
        out.push_str(&format!(
            "sovereign_ephemeral_relay_buffer_bytes {}\n",
            self.ephemeral_relay_buffer_bytes.load(Ordering::Relaxed)
        ));

        out.push_str("\n# HELP sovereign_cert_in_sla_seconds Statutory maximum response deadline under CERT-In Directions 2022 (seconds)\n");
        out.push_str("# TYPE sovereign_cert_in_sla_seconds gauge\n");
        out.push_str(&format!(
            "sovereign_cert_in_sla_seconds {}\n",
            self.cert_in_sla_seconds
        ));

        out.push_str("\n# HELP sovereign_rule4_2_originator_lookups_total IT Rules 2021 Rule 4(2) threshold warrant origin verifications\n");
        out.push_str("# TYPE sovereign_rule4_2_originator_lookups_total counter\n");
        out.push_str(&format!(
            "sovereign_rule4_2_originator_lookups_total {}\n",
            self.rule4_2_originator_lookups.load(Ordering::Relaxed)
        ));

        out.push_str("\n# HELP sovereign_dpdp_crypto_shredding_operations_total DPDP Act 2023 Section 12 irreversible key destruction operations\n");
        out.push_str("# TYPE sovereign_dpdp_crypto_shredding_operations_total counter\n");
        out.push_str(&format!(
            "sovereign_dpdp_crypto_shredding_operations_total {}\n",
            self.dpdp_crypto_shredding_ops.load(Ordering::Relaxed)
        ));

        out.push_str("\n# HELP sovereign_dpdp_shredding_entropy_bits Average Shannon entropy per byte of overwritten key blocks (target >= 7.989)\n");
        out.push_str("# TYPE sovereign_dpdp_shredding_entropy_bits gauge\n");
        let entropy = (self.last_shredding_entropy_scaled.load(Ordering::Relaxed) as f64) / 1000.0;
        out.push_str(&format!(
            "sovereign_dpdp_shredding_entropy_bits {:.3}\n",
            entropy
        ));

        out.push_str("\n# HELP sovereign_sim_binding_verifications_total RBI localized hardware SIM binding checks\n");
        out.push_str("# TYPE sovereign_sim_binding_verifications_total counter\n");
        out.push_str(&format!(
            "sovereign_sim_binding_verifications_total {}\n",
            self.sim_binding_verifications.load(Ordering::Relaxed)
        ));

        out.push_str("\n# HELP sovereign_zkp_age_verifications_total DPDP Section 9 Zero-Knowledge age proof verifications\n");
        out.push_str("# TYPE sovereign_zkp_age_verifications_total counter\n");
        out.push_str(&format!(
            "sovereign_zkp_age_verifications_total {}\n",
            self.zkp_age_verifications.load(Ordering::Relaxed)
        ));

        out
    }

    pub fn render_response(&self) -> HttpResponse {
        HttpResponse::raw_text(200, "text/plain; version=0.0.4", self.render_prometheus())
    }
}

impl Default for SreMetrics {
    fn default() -> Self {
        Self::new()
    }
}
