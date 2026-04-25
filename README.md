# LexiGuard AI: High-Performance Legal Document Analysis Server

A production-grade HTTPS server written in Rust with TLS 1.3 encryption, designed for automated AI-powered analysis of legal documents. The system provides real-time contract risk assessment through a clean web interface, demonstrating advanced systems programming, network security implementation, and modern DevOps practices.

## Technical Architecture

### Core Stack
- **Backend**: Rust (stable, edition 2024)
- **Networking**: Native TCP/IP with TLS 1.3 via `rustls`
- **Data Layer**: Structured JSON API with `serde` serialization
- **AI Integration**: xAI Grok API for intelligent document analysis
- **Infrastructure**: Docker with multi-stage builds, cross-platform automation scripts
- **Frontend**: Vanilla HTML5/CSS3 with asynchronous Fetch API (zero dependencies)

## Key Features

### 1. Custom Concurrency Engine
- **Thread Pool Implementation**: Custom-built thread pool with configurable worker count that efficiently distributes incoming connections across threads, avoiding the overhead of thread-per-connection models while maintaining responsiveness under load.
- **Channel-Based Job Distribution**: Uses MPSC (multiple producer, single consumer) channels for safe, lock-free communication between the main acceptor thread and worker threads.
- **Graceful Shutdown Mechanism**: Proper cleanup sequence that drops the sender first, then joins all worker threads, ensuring no connections are forcefully terminated during server shutdown.

### 2. Enterprise Network Security
- **TLS 1.3 Encryption**: Full HTTPS support with modern cipher suites. Server configuration includes safe defaults with no client authentication required for the public-facing API.
- **X.509 Certificate Management**: Custom certificate loading system that parses PEM-encoded certificates and PKCS#8 private keys, with comprehensive error handling for missing or malformed certificate files.
- **Rate Limiting & Anti-Abuse**: `ConnectionTracker` module monitors connection patterns per IP address, enforcing configurable limits on connections and requests per minute. Violators receive HTTP 429 responses and are temporarily blocked for 5 minutes.
- **Automatic Block Expiry**: Expired IP blocks are cleaned up during normal operation, preventing memory leaks from stale entries.

### 3. Intelligent Document Analysis API
- **POST /api/audit Endpoint**: Accepts JSON payloads containing document text and analysis depth parameter (`fast` or `deep`). Returns structured risk assessment reports.
- **Dual-Mode Operation**: 
  - **Production Mode**: When `GROK_API_KEY` environment variable is set, routes document analysis through xAI's Grok model with specialized legal system prompts for comprehensive contract review.
  - **Development/Testing Mode**: Falls back to deterministic test responses when no API key is configured, enabling development without external dependencies.
- **Structured Response Format**: Returns `AuditResponse` with risk score (0-100), specific findings list, and AI-generated recommendations.
- **CORS Support**: Handles OPTIONS preflight requests for cross-origin API access.

### 4. Frontend Interface
- **Dark Minimalist Design**: CSS custom properties for consistent theming with accent colors and smooth transitions.
- **Async API Integration**: Vanilla JavaScript using Fetch API for non-blocking document submission and result display.
- **Visual Status Indicators**: Animated pulse effect showing live server connection status, dynamic risk score coloring.
- **Responsive Layout**: Flexbox-based design that adapts to different viewport sizes without external frameworks.

### 5. DevOps & Automation
- **Multi-Stage Docker Build**: Build stage compiles the Rust binary with all optimizations, final stage uses minimal Debian Slim base image containing only the compiled binary, static assets, and certificate directory. This approach minimizes the attack surface and reduces image size.
- **Cross-Platform Certificate Generation**: 
  - `generate_cert.ps1` for Windows PowerShell with OpenSSL detection and fallback instructions
  - `generate_cert.sh` for Linux/Bash environments
- **Environment-Based Configuration**: API keys and runtime settings managed through environment variables, following twelve-factor app principles.


## Quick Start

### Docker Deployment
```bash
# Build the container image
docker build -t lexiguard-ai .

# Run with Grok API integration
docker run -p 8443:8443 -e GROK_API_KEY="xai-your-key-here" lexiguard-ai

# Run in test mode without external API
docker run -p 8443:8443 lexiguard-ai
```

# Generate SSL certificates (required once)
# On Linux/Mac:
./generate_cert.sh
# On Windows PowerShell:
.\generate_cert.ps1

# Set Grok API key (optional, for AI analysis)
export GROK_API_KEY="xai-your-key-here"

# Build and run with HTTPS
cargo run -- --https
