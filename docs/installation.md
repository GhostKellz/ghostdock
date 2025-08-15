# Installation Guide

This guide covers different ways to install and run GhostDock Registry.

## Docker (Recommended)

### Using Docker Compose

1. **Download the docker-compose.yml**:
   ```bash
   wget https://raw.githubusercontent.com/ghostkellz/ghostdock/main/docker-compose.yml
   ```

2. **Create configuration directory**:
   ```bash
   mkdir -p config data logs
   wget -O config/config.toml https://raw.githubusercontent.com/ghostkellz/ghostdock/main/config/config.toml
   ```

3. **Start GhostDock**:
   ```bash
   docker-compose up -d
   ```

4. **Verify installation**:
   ```bash
   curl http://localhost:5000/v2/
   curl http://localhost:8080/
   ```

### Using Docker Run

```bash
docker run -d \
  --name ghostdock \
  -p 5000:5000 \
  -p 8080:8080 \
  -v $(pwd)/data:/var/lib/ghostdock \
  -v $(pwd)/config:/etc/ghostdock \
  ghostdock:latest
```

## Binary Installation

### From Source

1. **Install Rust** (1.75 or later):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Clone and build**:
   ```bash
   git clone https://github.com/ghostkellz/ghostdock.git
   cd ghostdock
   export DATABASE_URL="sqlite:ghostdock.db"
   cargo build --release
   ```

3. **Install binary**:
   ```bash
   sudo cp target/release/ghostdock /usr/local/bin/
   ```

4. **Create systemd service** (optional):
   ```bash
   sudo cp scripts/ghostdock.service /etc/systemd/system/
   sudo systemctl enable ghostdock
   sudo systemctl start ghostdock
   ```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GHOSTDOCK_CONFIG` | Path to config file | `config.toml` |
| `GHOSTDOCK_BIND` | Bind address | `127.0.0.1` |
| `GHOSTDOCK_PORT` | Registry port | `5000` |
| `GHOSTDOCK_WEB_PORT` | Web UI port | `8080` |
| `GHOSTDOCK_STORAGE_PATH` | Storage directory | `./storage` |
| `GHOSTDOCK_DATABASE_PATH` | Database path | `./ghostdock.db` |
| `RUST_LOG` | Log level | `info` |

### Docker Configuration

When using Docker, you can configure GhostDock using environment variables:

```yaml
version: '3.8'
services:
  ghostdock:
    image: ghostdock:latest
    ports:
      - "5000:5000"
      - "8080:8080"
    environment:
      - GHOSTDOCK_BIND=0.0.0.0
      - GHOSTDOCK_STORAGE_PATH=/var/lib/ghostdock
      - RUST_LOG=info
    volumes:
      - ghostdock_data:/var/lib/ghostdock
      - ./config:/etc/ghostdock

volumes:
  ghostdock_data:
```

## Docker Client Configuration

To use your GhostDock registry with Docker, you need to configure Docker to trust it:

### For HTTP (Development)

Add to `/etc/docker/daemon.json`:
```json
{
  "insecure-registries": ["localhost:5000", "your-registry.com:5000"]
}
```

Then restart Docker:
```bash
sudo systemctl restart docker
```

### For HTTPS (Production)

1. **Set up SSL certificates** in nginx configuration
2. **Configure Docker daemon** (optional, for self-signed certificates):
   ```json
   {
     "registry-mirrors": [],
     "insecure-registries": []
   }
   ```

## Verification

After installation, verify GhostDock is working:

```bash
# Check registry API
curl http://localhost:5000/v2/

# Check web interface
curl http://localhost:8080/

# Test push/pull
docker pull alpine:latest
docker tag alpine:latest localhost:5000/alpine:test
docker push localhost:5000/alpine:test
docker pull localhost:5000/alpine:test
```

## Troubleshooting

### Common Issues

1. **Permission denied errors**:
   ```bash
   sudo chown -R $(id -u):$(id -g) data/ logs/
   ```

2. **Port already in use**:
   ```bash
   # Check what's using the port
   sudo lsof -i :5000
   sudo lsof -i :8080
   ```

3. **Docker push fails**:
   - Ensure Docker daemon is configured to trust your registry
   - Check that GhostDock is running and accessible

4. **Storage issues**:
   - Ensure storage directory has proper permissions
   - Check available disk space

### Logs

View GhostDock logs:

```bash
# Docker Compose
docker-compose logs -f ghostdock

# Binary installation
journalctl -f -u ghostdock

# Development
tail -f ghostdock.log
```

## Next Steps

- [Configuration Guide](configuration.md) - Detailed configuration options
- [Web UI Guide](web-ui.md) - Using the web interface
- [API Documentation](api.md) - Docker Registry v2 API reference
