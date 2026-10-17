use sovereign_gateway::{GatewayEngine, HttpResponse};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = std::env::var("SOVEREIGN_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("SOVEREIGN_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);

    let listener = TcpListener::bind(&addr).await?;
    println!("============================================================");
    println!("  BharatChat Sovereign Gateway Daemon");
    println!("  Listening on: http://{}", addr);
    println!("  Jurisdiction: INDIA_LOCALIZED_MUMBAI_BLR");
    println!("  Zero-Disk Ephemeral Relay: ACTIVE");
    println!("  DPDP Act 2023 & IT Rules 2021 Rule 4(2): ACTIVE");
    println!("============================================================");

    let gateway = Arc::new(GatewayEngine::new());

    loop {
        let (mut socket, peer_addr) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("[WARN] Failed to accept connection: {}", e);
                continue;
            }
        };

        let gw = Arc::clone(&gateway);
        tokio::spawn(async move {
            let mut buf = [0u8; 65536];
            let n = match socket.read(&mut buf).await {
                Ok(n) if n > 0 => n,
                _ => return,
            };

            let req_str = String::from_utf8_lossy(&buf[..n]);
            let mut lines = req_str.lines();
            if let Some(req_line) = lines.next() {
                let parts: Vec<&str> = req_line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let method = parts[0];
                    let path = parts[1];

                    // Extract body if present (split on \r\n\r\n or \n\n)
                    let body = if let Some(idx) = req_str.find("\r\n\r\n") {
                        &req_str[idx + 4..]
                    } else if let Some(idx) = req_str.find("\n\n") {
                        &req_str[idx + 2..]
                    } else {
                        ""
                    };

                    println!("[REQ] peer={} method={} path={}", peer_addr, method, path);

                    let response = if method == "OPTIONS" {
                        HttpResponse {
                            status_code: 204,
                            headers: vec![],
                            body: serde_json::Value::Null,
                        }
                    } else {
                        gw.dispatch_request(method, path, body).await
                    };

                    let resp_bytes = response.to_http_bytes();
                    let _ = socket.write_all(&resp_bytes).await;
                    let _ = socket.flush().await;
                }
            }
        });
    }
}
