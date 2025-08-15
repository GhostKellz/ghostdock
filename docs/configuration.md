# Configuration Guide

This guide covers all configuration options for GhostDock Registry.

## Configuration File

GhostDock uses TOML configuration files. The default location is `config.toml` in the working directory.

```toml
# config.toml
[server]
bind = "127.0.0.1"
port = 5000
web_port = 8080
workers = 4

[database]
url = "sqlite:ghostdock.db"
max_connections = 10
connect_timeout = 30

[storage]
type = "filesystem"
path = "./storage"
max_blob_size = "5GB"

[auth]
jwt_secret = "your-secret-key-here"
session_timeout = "24h"

[oauth.github]
enabled = true
client_id = "your-github-client-id"
client_secret = "your-github-client-secret"
redirect_uri = "http://localhost:8080/auth/github/callback"

[oauth.google]
enabled = true
client_id = "your-google-client-id"
client_secret = "your-google-client-secret"
redirect_uri = "http://localhost:8080/auth/google/callback"

[oauth.microsoft]
enabled = true
client_id = "your-microsoft-client-id"
client_secret = "your-microsoft-client-secret"
tenant_id = "your-tenant-id"
redirect_uri = "http://localhost:8080/auth/microsoft/callback"

[logging]
level = "info"
file = "ghostdock.log"
format = "json"

[metrics]
enabled = true
path = "/metrics"

[security]
require_auth = true
allow_anonymous_pull = false
max_upload_size = "1GB"
rate_limit = 100
```

## Environment Variables

Environment variables override configuration file settings:

### Server Configuration

- `GHOSTDOCK_BIND` - Bind address (default: `127.0.0.1`)
- `GHOSTDOCK_PORT` - Registry port (default: `5000`)
- `GHOSTDOCK_WEB_PORT` - Web UI port (default: `8080`)
- `GHOSTDOCK_WORKERS` - Number of worker threads (default: `4`)

### Database Configuration

- `DATABASE_URL` - Database connection string
- `GHOSTDOCK_DB_MAX_CONNECTIONS` - Maximum database connections (default: `10`)
- `GHOSTDOCK_DB_TIMEOUT` - Connection timeout in seconds (default: `30`)

### Storage Configuration

- `GHOSTDOCK_STORAGE_TYPE` - Storage backend type (`filesystem`, `s3`, `gcs`, `azure`)
- `GHOSTDOCK_STORAGE_PATH` - Storage path for filesystem backend
- `GHOSTDOCK_MAX_BLOB_SIZE` - Maximum blob size (default: `5GB`)

### Authentication Configuration

- `GHOSTDOCK_JWT_SECRET` - JWT signing secret (required)
- `GHOSTDOCK_SESSION_TIMEOUT` - Session timeout duration (default: `24h`)

### OAuth Configuration

#### GitHub
- `GHOSTDOCK_GITHUB_CLIENT_ID`
- `GHOSTDOCK_GITHUB_CLIENT_SECRET`
- `GHOSTDOCK_GITHUB_REDIRECT_URI`

#### Google
- `GHOSTDOCK_GOOGLE_CLIENT_ID`
- `GHOSTDOCK_GOOGLE_CLIENT_SECRET`
- `GHOSTDOCK_GOOGLE_REDIRECT_URI`

#### Microsoft
- `GHOSTDOCK_MICROSOFT_CLIENT_ID`
- `GHOSTDOCK_MICROSOFT_CLIENT_SECRET`
- `GHOSTDOCK_MICROSOFT_TENANT_ID`
- `GHOSTDOCK_MICROSOFT_REDIRECT_URI`

### Security Configuration

- `GHOSTDOCK_REQUIRE_AUTH` - Require authentication (default: `true`)
- `GHOSTDOCK_ALLOW_ANONYMOUS_PULL` - Allow anonymous pulls (default: `false`)
- `GHOSTDOCK_MAX_UPLOAD_SIZE` - Maximum upload size (default: `1GB`)
- `GHOSTDOCK_RATE_LIMIT` - Rate limit per minute (default: `100`)

### Logging Configuration

- `RUST_LOG` - Log level (`error`, `warn`, `info`, `debug`, `trace`)
- `GHOSTDOCK_LOG_FILE` - Log file path
- `GHOSTDOCK_LOG_FORMAT` - Log format (`json`, `text`)

## Storage Backends

### Filesystem Backend

```toml
[storage]
type = "filesystem"
path = "./storage"
```

The filesystem backend stores blobs in a content-addressed structure:
```
storage/
├── blobs/
│   └── sha256/
│       ├── ab/
│       │   └── abc123.../
│       │       └── data
│       └── de/
│           └── def456.../
│               └── data
└── temp/
```

### S3 Backend (Future)

```toml
[storage]
type = "s3"
bucket = "my-registry-bucket"
region = "us-west-2"
access_key = "your-access-key"
secret_key = "your-secret-key"
```

### Google Cloud Storage (Future)

```toml
[storage]
type = "gcs"
bucket = "my-registry-bucket"
project_id = "my-project"
credentials_path = "/path/to/credentials.json"
```

### Azure Blob Storage (Future)

```toml
[storage]
type = "azure"
container = "registry"
account_name = "mystorageaccount"
access_key = "your-access-key"
```

## Authentication Providers

### GitHub OAuth

1. **Create GitHub OAuth App**:
   - Go to GitHub Settings → Developer settings → OAuth Apps
   - Create new OAuth App
   - Set Authorization callback URL to `http://your-domain:8080/auth/github/callback`

2. **Configure GhostDock**:
   ```toml
   [oauth.github]
   enabled = true
   client_id = "your-client-id"
   client_secret = "your-client-secret"
   redirect_uri = "http://your-domain:8080/auth/github/callback"
   ```

### Google OAuth

1. **Create Google OAuth Client**:
   - Go to Google Cloud Console → APIs & Services → Credentials
   - Create OAuth 2.0 Client ID
   - Add authorized redirect URI: `http://your-domain:8080/auth/google/callback`

2. **Configure GhostDock**:
   ```toml
   [oauth.google]
   enabled = true
   client_id = "your-client-id"
   client_secret = "your-client-secret"
   redirect_uri = "http://your-domain:8080/auth/google/callback"
   ```

### Microsoft OAuth (Entra ID)

1. **Register Application in Azure AD**:
   - Go to Azure Portal → Azure Active Directory → App registrations
   - Create new registration
   - Add redirect URI: `http://your-domain:8080/auth/microsoft/callback`

2. **Configure GhostDock**:
   ```toml
   [oauth.microsoft]
   enabled = true
   client_id = "your-application-id"
   client_secret = "your-client-secret"
   tenant_id = "your-tenant-id"
   redirect_uri = "http://your-domain:8080/auth/microsoft/callback"
   ```

## SSL/TLS Configuration

For production deployments, use a reverse proxy like Nginx:

```nginx
server {
    listen 443 ssl http2;
    server_name your-registry.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    # Registry API
    location /v2/ {
        proxy_pass http://ghostdock:5000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        client_max_body_size 0;
        proxy_request_buffering off;
    }
    
    # Web UI
    location / {
        proxy_pass http://ghostdock:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Database Configuration

### SQLite (Default)

```toml
[database]
url = "sqlite:ghostdock.db"
max_connections = 10
```

### PostgreSQL (Future)

```toml
[database]
url = "postgresql://user:password@localhost/ghostdock"
max_connections = 20
```

### ZQLite (Future)

```toml
[database]
url = "zqlite:cluster.db"
nodes = ["node1:5432", "node2:5432", "node3:5432"]
```

## Performance Tuning

### Memory Settings

```toml
[server]
workers = 8  # Match CPU cores

[database]
max_connections = 20  # 2-3x workers

[storage]
cache_size = "1GB"
```

### Rate Limiting

```toml
[security]
rate_limit = 1000  # requests per minute
burst_limit = 100  # burst capacity
```

### Monitoring

```toml
[metrics]
enabled = true
path = "/metrics"

[logging]
level = "info"
format = "json"
```

## Security Best Practices

1. **Use HTTPS in production**
2. **Set strong JWT secret**:
   ```bash
   export GHOSTDOCK_JWT_SECRET=$(openssl rand -hex 32)
   ```
3. **Enable authentication**:
   ```toml
   [security]
   require_auth = true
   allow_anonymous_pull = false
   ```
4. **Set upload limits**:
   ```toml
   [security]
   max_upload_size = "1GB"
   ```
5. **Use rate limiting**:
   ```toml
   [security]
   rate_limit = 100
   ```

## Example Configurations

### Development

```toml
[server]
bind = "127.0.0.1"
port = 5000
web_port = 8080

[database]
url = "sqlite:dev.db"

[storage]
type = "filesystem"
path = "./dev-storage"

[auth]
jwt_secret = "dev-secret-key"

[security]
require_auth = false
allow_anonymous_pull = true
```

### Production

```toml
[server]
bind = "0.0.0.0"
port = 5000
web_port = 8080
workers = 8

[database]
url = "postgresql://ghostdock:password@db:5432/ghostdock"
max_connections = 20

[storage]
type = "s3"
bucket = "my-registry-bucket"
region = "us-west-2"

[auth]
jwt_secret = "very-secure-secret-key"

[security]
require_auth = true
allow_anonymous_pull = false
max_upload_size = "5GB"
rate_limit = 1000

[logging]
level = "info"
format = "json"
```

## Environment-Specific Overrides

Use multiple configuration files:

```bash
# Base configuration
ghostdock --config config.toml

# Override with environment-specific settings
ghostdock --config config.toml --config config.production.toml
```
