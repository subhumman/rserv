use rserver::{ThreadPool, ConnectionTracker, load_ssl_certs};
use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
    env,
};

fn main() {
    // проверяет аргументы командной строки
    let args: Vec<String> = env::args().collect();
    let use_https = args.contains(&"--https".to_string()) || args.contains(&"-s".to_string());
    if use_https {
        run_https_server();
    }else{
        println!("а вот http прописывать снова мне как то стало не очень вкусно, вот вам вместо этого кайф: ПУСТУЕТ ТРОН,
        Я ВЫГЛЯЖУ КАК САМЫЙ ОТВРАТИТЕЛЬНЫЙ КОРОЛЬ,
        НА ТВОЕЙ ПАРЕ КРОВЬ, ЭТО САМЫЙ УЖАСНЫЙ СОН,
        САМЫЙ УЖАСНЫЙ ЗАМОК В КОТОРОМ Я ОДИНОК,
        РЫЙАРИ НЕ ЧУВСТВУЮТ БОЛЬ Я БОЮСЬ СМОТРЕТЬ В КОРИДОР");
    }
}

/// запускает https сервер
fn run_https_server() {
    println!("запуск https сервера...");
    
    // загружает ssl сертификаты
    let ssl_config = match load_ssl_certs() {
        Ok(config) => {
            println!("ssl сертификаты загружены успешно");
            config
        }
        Err(e) => {
            eprintln!("ошибка загрузки ssl сертификатов: {}", e);
            eprintln!("убедитесь, что файлы certs/key.pem и certs/cert.pem существуют");
            eprintln!("запустите generate_cert.ps1 (windows) или generate_cert.sh (linux/macos)");
            return;
        }
    };
    
    let listener = TcpListener::bind("127.0.0.1:8443").unwrap();
    
    // создает пул из 4 потоков для обработки запросов
    let pool = ThreadPool::new(4);
    
    // создает трекер соединений для защиты от ddos
    let connection_tracker = Arc::new(Mutex::new(ConnectionTracker::new()));

    println!("https сервер запущен на https://127.0.0.1:8443");
    println!("защита от ddos активна");
    println!("ssl/tls шифрование включено");

    // обрабатывает входящие соединения
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let peer_addr = stream.peer_addr().unwrap();
        
        // клонирует трекер и ssl конфигурацию для передачи в поток
        let tracker = Arc::clone(&connection_tracker);
        let ssl_config = ssl_config.clone();
        
        pool.execute(move || {
            // проверяет, не заблокирован ли ip
            if {
                let mut tracker = tracker.lock().unwrap();
                tracker.is_ip_blocked(peer_addr)
            } {
                // отправляет ответ о блокировке
                send_blocked_response(stream);
                return;
            }
            
            // проверяет, не превышает ли ip лимиты
            if {
                let mut tracker = tracker.lock().unwrap();
                tracker.should_block_ip(peer_addr)
            } {
                // отправляет ответ о превышении лимитов
                send_rate_limit_response(stream);
                return;
            }
            
            // обрабатывает https соединение
            handle_https_connection(stream, ssl_config);
        });
    }

    println!("shutting down https server.");
}

/// обрабатывает https соединение
fn handle_https_connection(mut stream: TcpStream, ssl_config: rustls::ServerConfig) {
    use rustls::ServerConnection;
    use std::io::Write;
    use std::sync::Arc;

    let config = Arc::new(ssl_config);
    
    // создает tls соединение
    let mut tls_conn = match ServerConnection::new(config) {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("ошибка создания tls соединения: {}", e);
            return;
        }
    };

    // правильно создает tls stream - передает mutable reference
    let mut tls_stream = rustls::Stream::new(&mut tls_conn, &mut stream);

    // читает запрос через tls
    let mut reader = BufReader::new(&mut tls_stream);
    let mut request_line = String::new();
    
    if let Err(e) = reader.read_line(&mut request_line) {
        eprintln!("ошибка чтения tls потока: {}", e);
        return;
    }

    // обрабатывает запрос
    let (status_line, filename) = match request_line.trim() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "darkprince.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "darkprince.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n".to_string()
    });
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    // пишет ответ через tls
    if let Err(e) = tls_stream.write_all(response.as_bytes()) {
        eprintln!("ошибка записи в tls поток: {}", e);
    }
    
    // явно закрывает соединение
    let _ = tls_stream.flush();
}

/// отправляет ответ о блокировке ip
fn send_blocked_response(mut stream: TcpStream) {
    let response = "HTTP/1.1 429 Too Many Requests\r\n\
                   Content-Length: 0\r\n\
                   Retry-After: 300\r\n\
                   \r\n";
    let _ = stream.write_all(response.as_bytes());
}

/// отправляет ответ о превышении лимитов
fn send_rate_limit_response(mut stream: TcpStream) {
    let response = "HTTP/1.1 429 Too Many Requests\r\n\
                   Content-Length: 0\r\n\
                   Retry-After: 60\r\n\
                   \r\n";
    let _ = stream.write_all(response.as_bytes());
}