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

    let persist_path = std::env::var("SOVEREIGN_PERSISTENCE_DIR")
        .unwrap_or_else(|_| "data/persistence".to_string());
    println!("  Data Persistence Storage: {}", persist_path);
    let gateway = Arc::new(GatewayEngine::new_with_persistence(&persist_path));

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

                    // Check for WebSocket upgrade
                    let is_ws = req_str.lines().any(|l| {
                        let lower = l.to_ascii_lowercase();
                        lower.starts_with("upgrade:") && lower.contains("websocket")
                    });
                    let ws_key = req_str.lines().find_map(|l| {
                        if l.to_ascii_lowercase().starts_with("sec-websocket-key:") {
                            l.split(':').nth(1).map(|v| v.trim().to_string())
                        } else {
                            None
                        }
                    });

                    if is_ws && ws_key.is_some() {
                        let key = ws_key.unwrap();
                        let accept = sovereign_gateway::realtime::compute_ws_accept(&key);
                        let handshake = format!(
                            "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n",
                            accept
                        );
                        if socket.write_all(handshake.as_bytes()).await.is_ok() && socket.flush().await.is_ok() {
                            gw.metrics.active_websockets.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            println!("[WS] Established WebSocket session for peer={}", peer_addr);

                            let mut ws_buf = [0u8; 4096];
                            loop {
                                let n = match socket.read(&mut ws_buf).await {
                                    Ok(n) if n > 0 => n,
                                    _ => break,
                                };
                                if let Some((opcode, payload)) = sovereign_gateway::realtime::parse_ws_frame(&ws_buf[..n]) {
                                    if opcode == 0x8 {
                                        // Close frame
                                        let close_frame = [0x88, 0x00];
                                        let _ = socket.write_all(&close_frame).await;
                                        break;
                                    } else if opcode == 0x9 {
                                        // Ping -> Pong
                                        let pong = [0x8A, 0x00];
                                        let _ = socket.write_all(&pong).await;
                                    } else if opcode == 0x1 {
                                        let text = String::from_utf8_lossy(&payload);
                                        let ack = serde_json::json!({
                                            "type": "ACK",
                                            "received": text,
                                            "status": "PROCESSED",
                                            "jurisdiction": "INDIA_LOCALIZED_MUMBAI_BLR"
                                        }).to_string();
                                        let frame = sovereign_gateway::realtime::encode_ws_text_frame(&ack);
                                        let _ = socket.write_all(&frame).await;
                                    }
                                }
                            }

                            gw.metrics.active_websockets.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                            println!("[WS] Terminated WebSocket session for peer={}", peer_addr);
                            return;
                        }
                    }

                    let body_slice = if received.len() >= h_end + content_length {
                        &received[h_end..h_end + content_length]
                    } else {
                        &received[h_end..]
                    };
                    let body = String::from_utf8_lossy(body_slice);

                    println!("[REQ] peer={} method={} path={}", peer_addr, method, path);

                    let response = if method == "OPTIONS" {
                        HttpResponse::no_content()
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
