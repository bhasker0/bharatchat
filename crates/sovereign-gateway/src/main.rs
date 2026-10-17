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
            let mut received = Vec::new();
            let mut temp = [0u8; 8192];
            let mut header_end = None;
            let mut content_length: usize = 0;

            loop {
                let n = match socket.read(&mut temp).await {
                    Ok(n) if n > 0 => n,
                    _ => break,
                };
                received.extend_from_slice(&temp[..n]);

                if header_end.is_none() {
                    if let Some(pos) = received.windows(4).position(|w| w == b"\r\n\r\n") {
                        header_end = Some(pos + 4);
                        let header_str = String::from_utf8_lossy(&received[..pos]);
                        for line in header_str.lines() {
                            if line.to_ascii_lowercase().starts_with("content-length:") {
                                if let Some(val_str) = line.split(':').nth(1) {
                                    content_length = val_str.trim().parse::<usize>().unwrap_or(0);
                                }
                            }
                        }
                    } else if let Some(pos) = received.windows(2).position(|w| w == b"\n\n") {
                        header_end = Some(pos + 2);
                    }
                }

                if let Some(h_end) = header_end {
                    if received.len() >= h_end + content_length {
                        break;
                    }
                }
            }

            let h_end = match header_end {
                Some(pos) => pos,
                None => return,
            };

            let req_str = String::from_utf8_lossy(&received[..h_end]);
            let mut lines = req_str.lines();
            if let Some(req_line) = lines.next() {
                let parts: Vec<&str> = req_line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let method = parts[0];
                    let path = parts[1];

                    let body_slice = if received.len() >= h_end + content_length {
                        &received[h_end..h_end + content_length]
                    } else {
                        &received[h_end..]
                    };
                    let body = String::from_utf8_lossy(body_slice);

                    println!("[REQ] peer={} method={} path={}", peer_addr, method, path);

                    let response = if method == "OPTIONS" {
                        HttpResponse {
                            status_code: 204,
                            headers: vec![],
                            body: serde_json::Value::Null,
                        }
                    } else {
                        gw.dispatch_request(method, path, &body).await
                    };

                    let resp_bytes = response.to_http_bytes();
                    let _ = socket.write_all(&resp_bytes).await;
                    let _ = socket.flush().await;
                }
            }
        });
    }
}
