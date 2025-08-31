// Импортируем необходимые модули для многопоточности и синхронизации
use std::{
    sync::{Arc, Mutex, mpsc}, // arc для разделения между потоками, mutex для синхронизации, mpsc для каналов
    thread, // Для работы с потоками
    collections::HashMap,
    net::SocketAddr,
    time::{Duration, Instant},
    fs::File,
    io::BufReader,
};

// HTTPS модули
use rustls::{ServerConfig, PrivateKey, Certificate};
use rustls_pemfile::{certs, pkcs8_private_keys};

/// пул потоков для обработки http-запросов
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>, // option для корректного завершения
}

///замыкание, которое можно выполнить один раз
type Job = Box<dyn FnOnce() + Send + 'static>;

/// структура для отслеживания соединений с IP и защиты от DDoS
pub struct ConnectionTracker {
    connections: HashMap<SocketAddr, Vec<Instant>>,
    blocked_ips: HashMap<SocketAddr, Instant>,
    max_connections_per_ip: usize,
    max_requests_per_minute: usize,
    block_duration: Duration,
}

impl ConnectionTracker {
    /// создает новый трекер соединений
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            blocked_ips: HashMap::new(),
            max_connections_per_ip: 10, // максимум 10 соединений с одного IP
            max_requests_per_minute: 60, // максимум 60 запросов в минуту
            block_duration: Duration::from_secs(300), // блокировка на 5 минут
        }
    }

    /// проверяет, заблокирован ли IP
    pub fn is_ip_blocked(&mut self, ip: SocketAddr) -> bool {
        // очищаем устаревшие блокировки
        let now = Instant::now();
        self.blocked_ips.retain(|_, blocked_until| now < *blocked_until);
        
        self.blocked_ips.contains_key(&ip)
    }

    /// проверяет, нужно ли заблокировать IP
    pub fn should_block_ip(&mut self, ip: SocketAddr) -> bool {
        let now = Instant::now();
        
        // очищаем старые соединения (старше 1 минуты)
        let one_minute_ago = now - Duration::from_secs(60);
        
        if let Some(connections) = self.connections.get_mut(&ip) {
            connections.retain(|&time| time > one_minute_ago);
            
            // проверяем количество соединений
            if connections.len() > self.max_connections_per_ip {
                self.blocked_ips.insert(ip, now + self.block_duration);
                return true;
            }
            
            // проверяем rate limiting
            if connections.len() > self.max_requests_per_minute {
                self.blocked_ips.insert(ip, now + self.block_duration);
                return true;
            }
        }
        
        // добавляем новое соединение
        self.connections.entry(ip).or_insert_with(Vec::new).push(now);
        false
    }
}

/// загружает SSL сертификаты из файлов
pub fn load_ssl_certs() -> Result<ServerConfig, Box<dyn std::error::Error>> {
    // загружаем приватный ключ
    let key_file = File::open("certs/key.pem")?;
    let mut key_reader = BufReader::new(key_file);
    let keys: Vec<PrivateKey> = pkcs8_private_keys(&mut key_reader)?
        .into_iter()
        .map(|key| PrivateKey(key))
        .collect();
    
    if keys.is_empty() {
        return Err("Не найден приватный ключ".into());
    }
    
    // загружаем сертификат
    let cert_file = File::open("certs/cert.pem")?;
    let mut cert_reader = BufReader::new(cert_file);
    let certs: Vec<Certificate> = certs(&mut cert_reader)?
        .into_iter()
        .map(|cert| Certificate(cert))
        .collect();
    
    if certs.is_empty() {
        return Err("Не найден сертификат".into());
    }
    
    // создаем конфигурацию сервера
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, keys[0].clone())?;
    
    Ok(config)
}

impl ThreadPool {
    /// создает новый пул потоков
    ///
    /// # паника
    /// вызовет панику, если размер равен нулю
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        // оборачиваем в arc<mutex> для безопасного разделения между потоками
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// выполняет задачу в пуле потоков
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        // unwrap безопасен, так как sender всегда some
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    /// корректно завершает работу пула потоков
    fn drop(&mut self) {
        // убираем отправителя, что приведет к закрытию канала
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

/// рабочий поток в пуле
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>, // option для корректного завершения
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");
                        job();
                    }
                    Err(_) => {
                        // канал закрыт - завершаем поток
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}