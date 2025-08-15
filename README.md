# GhostDock Registry

A modern, secure, and feature-rich Docker Registry v2 implementation written in Rust. GhostDock provides enterprise-grade authentication, a beautiful web interface, and production-ready deployment options.

<div align="center">

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://www.docker.com)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](#)

</div>

## ✨ Features

### 🐳 **Docker Registry v2 API**
- Full compatibility with Docker CLI and container tools
- Manifest v2 schema support
- Chunked upload support for large images
- Content-addressable blob storage

### 🔐 **Enterprise Authentication**  
- **OAuth Integration**: GitHub, Google, Microsoft/Azure AD
- **JWT-based Sessions**: Secure token authentication
- **Role-based Access Control**: Admin, User, Guest roles
- **Personal Access Tokens**: API access for CI/CD

### 🌐 **Modern Web Interface**
- Beautiful, responsive design
- Repository and image management
- User administration dashboard  
- Real-time metrics and monitoring
- Mobile-friendly interface

### 🚀 **High Performance**
- Written in Rust with async I/O
- Multi-threaded request handling
- Efficient blob deduplication
- Connection pooling and caching

### 🛡️ **Security First**
- Secure by default configuration
- Rate limiting and DDoS protection
- Comprehensive audit logging
- Content scanning integration (planned)

### 📊 **Monitoring & Observability**
- Prometheus metrics endpoint
- Structured JSON logging
- Health check endpoints
- Performance dashboards

### ☁️ **Cloud Native**
- Container-first design
- Kubernetes ready
- Docker Compose deployment
- Multiple storage backends

## 🚀 Quick Start

### Docker Compose (Recommended)

Get started in under 60 seconds:

```bash
# Download configuration
curl -O https://raw.githubusercontent.com/ghostkellz/ghostdock/main/docker-compose.yml
mkdir -p config data logs
curl -o config/config.toml https://raw.githubusercontent.com/ghostkellz/ghostdock/main/config/config.toml

# Start GhostDock
docker-compose up -d

# Verify installation
curl http://localhost:5000/v2/     # Registry API
curl http://localhost:8080/        # Web UI
```

### Binary Installation

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/ghostkellz/ghostdock.git
cd ghostdock
export DATABASE_URL="sqlite:ghostdock.db"
cargo build --release

# Run
./target/release/ghostdock --config config.toml
```

### Production Install Script

```bash
# Automated installation for Linux
curl -sSL https://raw.githubusercontent.com/ghostkellz/ghostdock/main/scripts/install.sh | sudo bash
```

## 📋 Quick Usage

### Configure Docker Client

```bash
# For development (HTTP)
echo '{"insecure-registries": ["localhost:5000"]}' | sudo tee /etc/docker/daemon.json
sudo systemctl restart docker

# Test registry
docker pull alpine:latest
docker tag alpine:latest localhost:5000/test/alpine:latest
docker push localhost:5000/test/alpine:latest
```

### Web Interface

1. Open http://localhost:8080 in your browser
2. Sign in with OAuth (GitHub/Google/Microsoft) or create local account
3. Manage repositories, users, and settings through the web UI

## 📖 Documentation

| Document | Description |
|----------|-------------|
| [Installation Guide](docs/installation.md) | Detailed installation instructions |
| [Configuration Guide](docs/configuration.md) | Complete configuration reference |
| [Web UI Guide](docs/web-ui.md) | Using the web interface |
| [API Documentation](docs/api.md) | Docker Registry v2 & Management APIs |

### Quick Links

- **🛠 Installation**: [Docker](docs/installation.md#docker-recommended) • [Binary](docs/installation.md#binary-installation) • [Script](docs/installation.md#production-install-script)
- **⚙️ Configuration**: [OAuth Setup](docs/configuration.md#authentication-providers) • [Storage Backends](docs/configuration.md#storage-backends) • [SSL/TLS](docs/configuration.md#ssltls-configuration)
- **🖥 Web UI**: [Authentication](docs/web-ui.md#authentication) • [Repository Management](docs/web-ui.md#repository-management) • [User Administration](docs/web-ui.md#user-management)
- **🔧 API**: [Docker Registry v2](docs/api.md#docker-registry-v2-api) • [Management API](docs/api.md#ghostdock-management-api) • [Client Libraries](docs/api.md#client-libraries)

## 🏗 Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Docker CLI    │    │    Web Browser   │    │   CI/CD Tools   │
│                 │    │                  │    │                 │
└─────────┬───────┘    └─────────┬────────┘    └─────────┬───────┘
          │                      │                        │
          │ Registry v2 API      │ Web UI                 │ API
          │                      │                        │
      ┌───▼──────────────────────▼────────────────────────▼───┐
      │                GhostDock Registry                     │
      │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │
      │  │    Auth     │  │   Storage   │  │  Database   │   │
      │  │   (OAuth)   │  │ (FS/S3/GCS) │  │ (SQLite/PG) │   │
      │  └─────────────┘  └─────────────┘  └─────────────┘   │
      └───────────────────────────────────────────────────────┘
```

## 📊 Configuration Examples

### Development
```toml
[server]
bind = "127.0.0.1"
port = 5000

[auth]
jwt_secret = "dev-secret"

[security]
require_auth = false
allow_anonymous_pull = true
```

### Production
```toml
[server]
bind = "0.0.0.0"
port = 5000
workers = 8

[auth]
jwt_secret = "secure-random-key"

[oauth.github]
enabled = true
client_id = "your-github-client-id"
client_secret = "your-github-client-secret"

[security]
require_auth = true
allow_anonymous_pull = false
rate_limit = 1000
```

See [Configuration Guide](docs/configuration.md) for complete options.

## 🔧 Development

### Prerequisites

- Rust 1.75+
- SQLite 3
- Docker (for testing)

### Development Setup

```bash
# Clone repository
git clone https://github.com/ghostkellz/ghostdock.git
cd ghostdock

# Set up environment
export DATABASE_URL="sqlite:dev.db"
cp examples/config.dev.toml config.toml

# Build and run
make dev

# Run tests
make test

# Check code quality  
make lint
```

### Available Make Targets

```bash
make build          # Build release binary
make dev            # Run in development mode
make test           # Run test suite
make lint           # Run linting
make docker         # Build Docker image
make clean          # Clean build artifacts
```

## 🤝 Contributing

We welcome contributions! Here's how to get started:

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes**: Follow our coding standards
4. **Add tests**: Ensure your changes are tested
5. **Submit a PR**: We'll review and merge

### Development Guidelines

- Follow Rust best practices and idioms
- Add tests for new functionality
- Update documentation as needed
- Use conventional commit messages
- Ensure all CI checks pass

## � Roadmap

### Short Term (v1.1)
- [ ] Content vulnerability scanning
- [ ] Repository mirroring and sync
- [ ] S3/GCS/Azure storage backends
- [ ] Advanced RBAC permissions

### Medium Term (v1.2)
- [ ] ZQLite database backend
- [ ] GraphQL API
- [ ] Webhook system
- [ ] Image signing verification

### Long Term (v2.0)
- [ ] Multi-registry federation
- [ ] AI-powered image optimization
- [ ] Advanced analytics dashboard
- [ ] Kubernetes operator

## 📦 Examples

Check the [examples/](examples/) directory for:
- [Python client](examples/python_client.py) - Programmatic registry access
- [Test scripts](examples/test_registry.sh) - Automated testing
- [Config templates](examples/) - Production configurations
- [Docker Compose](examples/docker-compose.production.yml) - Production deployment

## � License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<div align="center">

**[Documentation](docs/) • [Examples](examples/) • [Issues](https://github.com/ghostkellz/ghostdock/issues) • [Discussions](https://github.com/ghostkellz/ghostdock/discussions)**

Made with ❤️ by the GhostDock team

</div>
