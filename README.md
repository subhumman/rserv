#LexiGuard AI: High-Performance Legal Document Analysis Server
A production-grade HTTPS server written in Rust with TLS 1.3 encryption, designed for automated AI-powered analysis of legal documents. The system provides real-time contract risk assessment through a clean web interface, demonstrating advanced systems programming, network security implementation, and modern DevOps practices.

#Technical Architecture
##Core Stack
Backend: Rust (stable, edition 2024). Networking: Native TCP/IP with TLS 1.3 via rustls. Data Layer: Structured JSON API with serde serialization. AI Integration: xAI Grok API for intelligent document analysis. Infrastructure: Docker with multi-stage builds, cross-platform automation scripts. Frontend: Vanilla HTML5/CSS3 with asynchronous Fetch API (zero JavaScript dependencies).

##Key Features
###1. Custom Concurrency Engine
The server implements a custom-built thread pool with configurable worker count that efficiently distributes incoming connections across threads, avoiding the overhead of thread-per-connection models while maintaining responsiveness under load. Channel-based job distribution uses MPSC (multiple producer, single consumer) channels for safe, lock-free communication between the main acceptor thread and worker threads. The graceful shutdown mechanism executes a proper cleanup sequence that drops the sender first, then joins all worker threads, ensuring no connections are forcefully terminated during server shutdown.

###2. Enterprise Network Security
Full HTTPS support with TLS 1.3 encryption and modern cipher suites. The server configuration includes safe defaults with no client authentication required for the public-facing API. The custom certificate loading system parses PEM-encoded certificates and PKCS#8 private keys, with comprehensive error handling for missing or malformed certificate files. The ConnectionTracker module monitors connection patterns per IP address, enforcing configurable limits on connections and requests per minute. Violators receive HTTP 429 responses and are temporarily blocked for 5 minutes. Expired IP blocks are cleaned up during normal operation, preventing memory leaks from stale entries.

###3. Intelligent Document Analysis API
The POST /api/audit endpoint accepts JSON payloads containing document text and analysis depth parameter (fast or deep), returning structured risk assessment reports. The system operates in dual mode: production mode routes document analysis through xAI's Grok model with specialized legal system prompts for comprehensive contract review when the GROK_API_KEY environment variable is set; development and testing mode falls back to deterministic test responses when no API key is configured, enabling development without external dependencies. The structured response format returns AuditResponse with risk score from 0 to 100, specific findings list, and AI-generated recommendations. CORS support handles OPTIONS preflight requests for cross-origin API access.

###4. Frontend Interface
Dark minimalist design uses CSS custom properties for consistent theming with accent colors and smooth transitions. Vanilla JavaScript with Fetch API enables non-blocking document submission and result display. Visual status indicators include animated pulse effect showing live server connection status and dynamic risk score coloring. Flexbox-based responsive layout adapts to different viewport sizes without external frameworks.

###5. DevOps and Automation
Multi-stage Docker build compiles the Rust binary with all optimizations in the build stage, while the final stage uses minimal Debian Slim base image containing only the compiled binary, static assets, and certificate directory. This approach minimizes the attack surface and reduces image size. Cross-platform certificate generation scripts include generate_cert.ps1 for Windows PowerShell with OpenSSL detection and fallback instructions, and generate_cert.sh for Linux and Bash environments. Environment-based configuration manages API keys and runtime settings through environment variables, following twelve-factor app principles.

##Quick Start
``` Docker
Build the container image with docker build -t lexiguard-ai . then run with Grok API integration using docker run -p 8443:8443 -e GROK_API_KEY="xai-your-key-here" lexiguard-ai or run in test mode without external API using docker run -p 8443:8443 lexiguard-ai. The container automatically generates SSL certificates on first launch. Access the service at https://localhost:8443.
```
```Local Development
Generate SSL certificates once using ./generate_cert.sh on Linux/Mac or .\generate_cert.ps1 on Windows PowerShell. Set the Grok API key optionally with export GROK_API_KEY="xai-your-key-here" for AI analysis. Build and run with HTTPS using cargo run -- --https.
```

##API Reference
POST /api/audit
Analyzes a legal document and returns a risk assessment. The request body requires document_text as a string containing the complete text of the document to analyze, and analysis_depth as a string set to either "fast" for quick review or "deep" for comprehensive analysis. A successful response with status 200 returns JSON containing status as "success", risk_score as a number from 0 to 100, findings as an array of strings describing detected issues, and ai_suggestion as a string with recommendations. Example success response: {"status": "success", "risk_score": 35, "findings": ["Standard payment terms detected", "Missing confidentiality clause", "Penalty sanctions not detailed"], "ai_suggestion": "Add force majeure clause considering current sanctions, detail liability provisions, include confidentiality agreement."}. An error response returns: {"status": "error", "risk_score": 0, "findings": ["No document provided"], "ai_suggestion": "Please insert contract text for analysis."}.

##Rate Limiting
The server enforces the following limits per IP address: maximum 10 concurrent connections, maximum 60 requests per minute. Violations result in HTTP 429 with a 5-minute temporary block.

##Dependencies
Runtime dependencies include rustls 0.21 for TLS 1.3 implementation, rustls-pemfile 1.0 for PEM certificate parsing, serde and serde_json 1.0 for JSON serialization, reqwest 0.11 as HTTP client for Grok API integration, and tokio 1.0 as async runtime for API calls. Development requirements include the Rust toolchain stable version, OpenSSL for local certificate generation, and Docker for containerized deployment.

##Security Considerations
All traffic is encrypted with TLS 1.3 using modern cipher suites and perfect forward secrecy. Development certificates are self-signed, generated locally, and not trusted by browsers, requiring a security exception for local testing. For production deployment, replace self-signed certificates with CA-signed certificates from Let's Encrypt or a commercial CA. The Grok API key is passed via environment variable, never hardcoded or committed to version control. Built-in rate limiting provides protection against brute-force attempts and DoS attacks. The minimal Docker container image contains only the compiled binary, reducing potential attack vectors. Rust's ownership model eliminates entire classes of memory-related vulnerabilities.
