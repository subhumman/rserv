LexiGuard AI: MVP Legal Audit Service in Rust
High-performance web server with TLS 1.3 support, designed for automated analysis of legal documents using AI logic. The project demonstrates skills in systems programming, network security, and containerization.

Main Technology Stack
Backend: Rust (stable branch 2026).

Networking: TCP/IP, TLS 1.3 (rustls library).

Data: JSON API (serde library).

Infrastructure: Docker (Multi-stage build), PowerShell/Bash (automation).

Frontend: HTML5/CSS3 (Modern Dark Minimalist), Vanilla JS (Fetch API).

Key Features and Implemented Tasks
1. System Architecture and Performance
Custom Thread Pool: Implemented a custom thread pool for parallel processing of incoming connections. This allows the server to remain responsive under load without creating a new thread for each request.

Graceful Shutdown: Safe server shutdown with proper stopping of workflows. 2. Network Security HTTPS/TLS 1.3: All traffic is encrypted. Logic for loading and parsing X.509 certificates and PKCS#8 private keys is implemented. Connection Tracker (Anti-DDoS): A system to prevent request brute-forcing (Rate Limiting). The server monitors the number of connections from a single IP address and temporarily blocks suspicious activity (status 429 Too Many Requests). 3. API and Processing Logic JSON API: A POST /api/audit endpoint is implemented that accepts document text and returns a structured risk report. Frontend Integration: Asynchronous interaction without page reload. 4. Automation and DevOps Cross-Platform: Automation scripts for Windows (PowerShell) and Linux (Bash) are prepared for rapid generation of the required SSL certificates. Docker Containerization: Multi-stage build is used. The final image contains only the binary file and minimal dependencies (Debian Slim)., что минимизирует размер образа и поверхность атаки.

How to Run the Project
Quick Start via Docker
The project is fully self-contained and generates certificates inside the container:

After starting, the service will be available at: https://localhost:8443

Local Development
Generate certificates: .generate_cert.ps1 (for Windows) or ./generate_cert.sh (for Linux).

Run the server: cargo run -- --https

Project Structure
src/main.rs — entry point, HTTP handling logic and routing.

src/lib.rs — system core: ThreadPool, ConnectionTracker, working with certificates.

index.html — user interface (audit panel).

certs/ — directory for storing keys (generated automatically).
