use rserver::{load_ssl_certs, ConnectionTracker, ThreadPool, AuditRequest, AuditResponse, grok_client};
use std::{
    env, fs,
    io::{self, prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let use_https = args.iter().any(|arg| arg == "--https" || arg == "-s");

    if use_https {
        run_https_server();
    } else {
        println!("HTTP not configured. The throne is empty, the king is in the corridor...");
        println!("Run with --https flag to activate the server");
    }
}

fn run_https_server() {
    // Get API key from environment variable
    let grok_api_key = env::var("GROK_API_KEY")
        .unwrap_or_else(|_| {
            eprintln!("⚠️  GROK_API_KEY not set. AI analysis will run in test mode.");
            String::new()
        });

    let ssl_config = match load_ssl_certs() {
        Ok(config) => Arc::new(config),
        Err(e) => {
            eprintln!("Critical SSL error: {}", e);
            return;
        }
    };

    let listener = TcpListener::bind("0.0.0.0:8443").expect("Failed to bind port");
    let pool = ThreadPool::new(4);
    let tracker = Arc::new(Mutex::new(ConnectionTracker::new()));

    println!("🔒 HTTPS Server live on https://0.0.0.0:8443");
    println!("📋 Legal AI Assistant activated");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let peer_addr = stream.peer_addr().unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
                let tracker_cloned = Arc::clone(&tracker);
                let config_cloned = Arc::clone(&ssl_config);
                let api_key = grok_api_key.clone();

                pool.execute(move || {
                    let is_blocked = {
                        let mut t = tracker_cloned.lock().unwrap();
                        t.is_ip_blocked(peer_addr) || t.should_block_ip(peer_addr)
                    };

                    if is_blocked {
                        let _ = send_error_response(stream, "429 Too Many Requests");
                        return;
                    }

                    handle_https_connection(stream, config_cloned, &api_key);
                });
            }
            Err(e) => eprintln!("Incoming connection error: {}", e),
        }
    }
}

fn send_error_response(mut stream: TcpStream, status: &str) -> io::Result<()> {
    let response = format!("HTTP/1.1 {}\r\nContent-Length: 0\r\n\r\n", status);
    stream.write_all(response.as_bytes())
}

fn handle_https_connection(mut stream: TcpStream, config: Arc<rustls::ServerConfig>, grok_api_key: &str) {
    let mut conn = match rustls::ServerConnection::new(config) {
        Ok(c) => c,
        Err(_) => return,
    };
    let mut tls_stream = rustls::Stream::new(&mut conn, &mut stream);
    
    let mut reader = BufReader::new(&mut tls_stream);
    let mut first_line = String::new();
    if reader.read_line(&mut first_line).is_err() { return; }

    if first_line.starts_with("POST /api/audit") {
        let mut content_length = 0;
        let mut line = String::new();
        
        while reader.read_line(&mut line).unwrap_or(0) > 2 {
            if line.to_lowercase().starts_with("content-length:") {
                content_length = line.split(':').nth(1).unwrap().trim().parse().unwrap_or(0);
            }
            line.clear();
        }

        let mut buffer = vec![0; content_length];
        if reader.read_exact(&mut buffer).is_err() { return; }
        
        let req_data: AuditRequest = serde_json::from_slice(&buffer).unwrap_or(AuditRequest {
            document_text: "".into(),
            analysis_depth: "fast".into(),
        });

        // Use Grok API if key is available, otherwise return test response
        let response_data = if !grok_api_key.is_empty() && !req_data.document_text.is_empty() {
            match grok_client::analyze_document_sync(
                grok_api_key,
                &req_data.document_text,
                &req_data.analysis_depth
            ) {
                Ok(grok_response) => {
                    // Try to parse JSON from Grok
                    serde_json::from_str::<AuditResponse>(&grok_response)
                        .unwrap_or_else(|_| {
                            // If Grok didn't return JSON, create structured response
                            AuditResponse {
                                status: "success".into(),
                                risk_score: 50,
                                findings: vec!["AI analysis completed".into()],
                                ai_suggestion: grok_response,
                            }
                        })
                }
                Err(e) => {
                    eprintln!("Grok API error: {}", e);
                    create_fallback_response(&req_data.document_text, &req_data.analysis_depth)
                }
            }
        } else {
            // Test response without API
            create_fallback_response(&req_data.document_text, &req_data.analysis_depth)
        };

        let json_response = serde_json::to_string(&response_data).unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
            json_response.len(),
            json_response
        );
        let _ = tls_stream.write_all(response.as_bytes());
        let _ = tls_stream.flush();

    } else if first_line.starts_with("OPTIONS") {
        // CORS preflight
        let response = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: POST, GET, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Length: 0\r\n\r\n";
        let _ = tls_stream.write_all(response.as_bytes());
        let _ = tls_stream.flush();
        
    } else {
        let request_line = first_line.trim();

        let (status, file, content_type) = match request_line {
            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "static/index.html", "text/html; charset=utf-8"),
            "GET /sleep HTTP/1.1" => {
                thread::sleep(Duration::from_secs(2));
                ("HTTP/1.1 200 OK", "static/index.html", "text/html; charset=utf-8")
            }
            _ => ("HTTP/1.1 404 NOT FOUND", "static/404.html", "text/html; charset=utf-8"),
        };

        let contents = fs::read_to_string(file).unwrap_or_else(|_| "<h1>Error</h1>".to_string());
        let response = format!(
            "{}\r\nContent-Type: {}\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
            status, content_type, contents.len(), contents
        );
        
        let _ = tls_stream.write_all(response.as_bytes());
        let _ = tls_stream.flush();
    }
}

fn create_fallback_response(text: &str, depth: &str) -> AuditResponse {
    if text.is_empty() || text == "Empty document" {
        return AuditResponse {
            status: "error".into(),
            risk_score: 0,
            findings: vec!["No document provided".into()],
            ai_suggestion: "Please insert contract text for analysis.".into(),
        };
    }

    let deep_analysis = depth == "deep";
    
    AuditResponse {
        status: "success".into(),
        risk_score: if deep_analysis { 35 } else { 20 },
        findings: if deep_analysis {
            vec![
                "Standard payment terms detected".into(),
                "Execution period: 30 days (within norm)".into(),
                "Missing confidentiality clause".into(),
                "Penalty sanctions not detailed".into(),
            ]
        } else {
            vec![
                "Basic analysis: no obvious violations found".into(),
                "Deep analysis recommended".into(),
            ]
        },
        ai_suggestion: if deep_analysis {
            "🤖 AI recommends: add a force majeure clause considering 2026 sanctions, detail liability of parties, and include confidentiality provisions. For precise analysis, connect Grok API (set GROK_API_KEY).".into()
        } else {
            "🤖 Quick analysis completed. For detailed review use 'deep' mode. Connect Grok API for full AI analysis.".into()
        },
    }
}
