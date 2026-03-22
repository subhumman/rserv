use rserver::{load_ssl_certs, ConnectionTracker, ThreadPool, AuditRequest, AuditResponse};
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
        println!("HTTP не настроен. Трон пустует, король в коридоре...");
    }
}

fn run_https_server() {
    let ssl_config = match load_ssl_certs() {
        Ok(config) => Arc::new(config),
        Err(e) => {
            eprintln!("Критическая ошибка SSL: {}", e);
            return;
        }
    };

    let listener = TcpListener::bind("0.0.0.0:8443").expect("Не удалось забиндить порт");
    let pool = ThreadPool::new(4);
    let tracker = Arc::new(Mutex::new(ConnectionTracker::new()));

    println!("HTTPS Server live on https://0.0.0.0:8443");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let peer_addr = stream.peer_addr().unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
                let tracker_cloned: Arc<Mutex<ConnectionTracker>> = Arc::clone(&tracker);
                let config_cloned = Arc::clone(&ssl_config);

                pool.execute(move || {
                    let is_blocked = {
                        let mut t = tracker_cloned.lock().unwrap();
                        t.is_ip_blocked(peer_addr) || t.should_block_ip(peer_addr)
                    };

                    if is_blocked {
                        let _ = send_error_response(stream, "429 Too Many Requests");
                        return;
                    }

                    handle_https_connection(stream, config_cloned);
                });
            }
            Err(e) => eprintln!("Ошибка входящего соединения: {}", e),
        }
    }
}

fn send_error_response(mut stream: TcpStream, status: &str) -> io::Result<()> {
    let response = format!("HTTP/1.1 {}\r\nContent-Length: 0\r\n\r\n", status);
    stream.write_all(response.as_bytes())
}

fn handle_https_connection(mut stream: TcpStream, config: Arc<rustls::ServerConfig>) {
    let mut conn = rustls::ServerConnection::new(config).unwrap();
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
        reader.read_exact(&mut buffer).unwrap();
        
        let req_data: AuditRequest = serde_json::from_slice(&buffer).unwrap_or(AuditRequest {
            document_text: "".into(),
            analysis_depth: "fast".into(),
        });

        // имитация AI-ответа
        let response_data = AuditResponse { 
            status: "success".into(),
            risk_score: 15,
            findings: vec![
                "Найдена опечатка в реквизитах".into(),
                "Срок оплаты превышает 30 дней".into(),
            ],
            ai_suggestion: "Рекомендуется добавить пункт о форс-мажоре в условиях санкций 2026 года.".into(),
        };

        let json_response = serde_json::to_string(&response_data).unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            json_response.len(),
            json_response
        );
        let _ = tls_stream.write_all(response.as_bytes());
        let _ = tls_stream.flush();   

    } else {
        
        let request_line = first_line.trim();

        let (status, file) = match request_line {
            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "index.html"),
            "GET /sleep HTTP/1.1" => {
                thread::sleep(Duration::from_secs(2));
                ("HTTP/1.1 200 OK", "index.html")
            }
            _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
        };

        let contents = fs::read_to_string(file).unwrap_or_else(|_| "Error".to_string());
        let response = format!("{}\r\nContent-Length: {}\r\n\r\n{}", status, contents.len(), contents);
        
        let _ = tls_stream.write_all(response.as_bytes());
        let _ = tls_stream.flush();
    }  
}