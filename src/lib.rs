use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    net::SocketAddr,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};

pub mod grok_client;

/// Thread pool for handling concurrent connections.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Deserialize)]
pub struct AuditRequest {
    pub document_text: String,
    pub analysis_depth: String, // "fast" or "deep"
}

#[derive(Serialize)]
pub struct AuditResponse {
    pub status: String,
    pub risk_score: u8,
    pub findings: Vec<String>,
    pub ai_suggestion: String,
}

pub struct ConnectionTracker {
    connections: HashMap<SocketAddr, Vec<Instant>>,
    blocked_ips: HashMap<SocketAddr, Instant>,
    max_connections_per_ip: usize,
    max_requests_per_minute: usize,
    block_duration: Duration,
}

impl ConnectionTracker {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            blocked_ips: HashMap::new(),
            max_connections_per_ip: 10,
            max_requests_per_minute: 60,
            block_duration: Duration::from_secs(300),
        }
    }

    pub fn is_ip_blocked(&mut self, ip: SocketAddr) -> bool {
        let now = Instant::now();
        // Clean up expired blocks on each check
        self.blocked_ips.retain(|_, &mut blocked_until| now < blocked_until);
        self.blocked_ips.contains_key(&ip)
    }

    pub fn should_block_ip(&mut self, ip: SocketAddr) -> bool {
        let now = Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);

        let entries = self.connections.entry(ip).or_default();
        entries.retain(|&time| time > one_minute_ago);
        entries.push(now);

        if entries.len() > self.max_connections_per_ip || entries.len() > self.max_requests_per_minute {
            self.blocked_ips.insert(ip, now + self.block_duration);
            return true;
        }
        false
    }
}

pub fn load_ssl_certs() -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let key_file = File::open("certs/key.pem")?;
    let mut key_reader = BufReader::new(key_file);
    let keys: Vec<PrivateKey> = pkcs8_private_keys(&mut key_reader)?
        .into_iter()
        .map(PrivateKey)
        .collect();

    if keys.is_empty() {
        return Err("Private key not found".into());
    }

    let cert_file = File::open("certs/cert.pem")?;
    let mut cert_reader = BufReader::new(cert_file);
    let cert_chain: Vec<Certificate> = certs(&mut cert_reader)?
        .into_iter()
        .map(Certificate)
        .collect();

    if cert_chain.is_empty() {
        return Err("Certificate not found".into());
    }

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, keys[0].clone())?;

    Ok(config)
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Self { workers, sender: Some(sender) }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        if let Some(ref sender) = self.sender {
            // In production, better to log send errors
            let _ = sender.send(job);
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let message = {
                let lock = receiver.lock().expect("Mutex poisoned");
                lock.recv()
            };

            match message {
                Ok(job) => job(),
                Err(_) => break,
            }
        });

        Self { id, thread: Some(thread) }
    }
}
