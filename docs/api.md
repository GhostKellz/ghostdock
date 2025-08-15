# API Documentation

GhostDock Registry implements the Docker Registry v2 API specification with additional management APIs.

## Docker Registry v2 API

### Base URL

```
https://your-registry.com/v2/
```

### Authentication

All API endpoints require authentication except for the base endpoint.

#### Bearer Token Authentication

```bash
# Get auth challenge
curl -I https://your-registry.com/v2/

# Authenticate and get token
curl -X POST https://your-registry.com/auth \
  -H "Content-Type: application/json" \
  -d '{"username":"user","password":"pass"}'

# Use token in requests
curl -H "Authorization: Bearer $TOKEN" \
  https://your-registry.com/v2/_catalog
```

### Core Endpoints

#### Check API Version

```http
GET /v2/
```

Returns API version and supported features.

**Response:**
```json
{
  "registry": "GhostDock Registry",
  "version": "1.0.0",
  "features": ["oauth", "webhooks", "scanning"]
}
```

#### List Repositories

```http
GET /v2/_catalog
```

**Parameters:**
- `n` (int): Maximum number of repositories to return
- `last` (string): Last repository name for pagination

**Response:**
```json
{
  "repositories": [
    "library/nginx",
    "myapp/frontend",
    "myapp/backend"
  ]
}
```

#### List Tags

```http
GET /v2/{repository}/tags/list
```

**Parameters:**
- `n` (int): Maximum number of tags to return  
- `last` (string): Last tag for pagination

**Response:**
```json
{
  "name": "myapp/frontend",
  "tags": [
    "latest",
    "v1.0.0",
    "v1.1.0"
  ]
}
```

### Manifest Operations

#### Get Manifest

```http
GET /v2/{repository}/manifests/{tag_or_digest}
```

**Headers:**
- `Accept: application/vnd.docker.distribution.manifest.v2+json`

**Response:**
```json
{
  "schemaVersion": 2,
  "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
  "config": {
    "mediaType": "application/vnd.docker.container.image.v1+json",
    "size": 7023,
    "digest": "sha256:b5b2b2c..."
  },
  "layers": [
    {
      "mediaType": "application/vnd.docker.image.rootfs.diff.tar.gzip",
      "size": 32654,
      "digest": "sha256:e692418..."
    }
  ]
}
```

#### Put Manifest

```http
PUT /v2/{repository}/manifests/{tag}
Content-Type: application/vnd.docker.distribution.manifest.v2+json

{manifest_json}
```

**Response:**
- `201 Created`: Manifest uploaded successfully
- `Location` header contains manifest URL

#### Delete Manifest

```http
DELETE /v2/{repository}/manifests/{digest}
```

**Response:**
- `202 Accepted`: Manifest deletion accepted

### Blob Operations

#### Check Blob Exists

```http
HEAD /v2/{repository}/blobs/{digest}
```

**Response:**
- `200 OK`: Blob exists
- `404 Not Found`: Blob doesn't exist

#### Get Blob

```http
GET /v2/{repository}/blobs/{digest}
```

**Response:**
Binary blob data with appropriate `Content-Type`.

#### Upload Blob

Two-step process: initiate upload, then upload content.

##### Initiate Upload

```http
POST /v2/{repository}/blobs/uploads/
```

**Response:**
```http
202 Accepted
Location: /v2/{repository}/blobs/uploads/{uuid}
Range: 0-0
```

##### Upload Content

```http
PUT /v2/{repository}/blobs/uploads/{uuid}?digest={digest}
Content-Type: application/octet-stream

{binary_data}
```

**Response:**
```http
201 Created  
Location: /v2/{repository}/blobs/{digest}
```

#### Chunked Upload

For large blobs, use chunked upload:

```http
PATCH /v2/{repository}/blobs/uploads/{uuid}
Content-Type: application/octet-stream
Content-Range: {start}-{end}

{chunk_data}
```

#### Delete Blob

```http
DELETE /v2/{repository}/blobs/{digest}
```

**Response:**
- `202 Accepted`: Blob deletion accepted

## GhostDock Management API

### Base URL

```
https://your-registry.com/api/v1/
```

### User Management

#### List Users

```http
GET /api/v1/users
```

**Response:**
```json
{
  "users": [
    {
      "id": 1,
      "username": "admin",
      "email": "admin@example.com",
      "role": "admin",
      "created_at": "2024-01-01T00:00:00Z",
      "last_login": "2024-01-15T12:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "limit": 50
}
```

#### Create User

```http
POST /api/v1/users
Content-Type: application/json

{
  "username": "newuser",
  "email": "user@example.com",
  "password": "secure-password",
  "role": "user"
}
```

#### Update User

```http
PUT /api/v1/users/{id}
Content-Type: application/json

{
  "email": "newemail@example.com",
  "role": "admin"
}
```

#### Delete User

```http
DELETE /api/v1/users/{id}
```

### Repository Management

#### Repository Details

```http
GET /api/v1/repositories/{repository}
```

**Response:**
```json
{
  "name": "myapp/frontend",
  "description": "Frontend application",
  "visibility": "private",
  "owner": "myuser",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-15T12:00:00Z",
  "size": 1024000,
  "pull_count": 150,
  "tags": [
    {
      "name": "latest",
      "digest": "sha256:abc123...",
      "size": 524288,
      "created_at": "2024-01-15T12:00:00Z"
    }
  ]
}
```

#### Update Repository

```http
PUT /api/v1/repositories/{repository}
Content-Type: application/json

{
  "description": "Updated description",
  "visibility": "public"
}
```

#### Delete Repository

```http
DELETE /api/v1/repositories/{repository}
```

### Access Tokens

#### List Tokens

```http
GET /api/v1/tokens
```

#### Create Token

```http
POST /api/v1/tokens
Content-Type: application/json

{
  "name": "CI/CD Token",
  "permissions": ["pull", "push"],
  "expires_at": "2024-12-31T23:59:59Z"
}
```

**Response:**
```json
{
  "id": 1,
  "name": "CI/CD Token",
  "token": "ghd_abc123...",
  "permissions": ["pull", "push"],
  "created_at": "2024-01-15T12:00:00Z",
  "expires_at": "2024-12-31T23:59:59Z"
}
```

#### Delete Token

```http
DELETE /api/v1/tokens/{id}
```

### System Information

#### Health Check

```http
GET /api/v1/health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "services": {
    "database": "healthy",
    "storage": "healthy",
    "auth": "healthy"
  },
  "metrics": {
    "repositories": 25,
    "images": 150,
    "users": 10,
    "storage_used": "2.5GB"
  }
}
```

#### System Metrics

```http
GET /api/v1/metrics
```

**Response:**
```json
{
  "timestamp": "2024-01-15T12:00:00Z",
  "requests": {
    "total": 10000,
    "success": 9500,
    "errors": 500
  },
  "registry": {
    "pulls": 8000,
    "pushes": 1500,
    "repositories": 25,
    "images": 150
  },
  "storage": {
    "total_size": 2684354560,
    "blob_count": 500,
    "manifest_count": 150
  },
  "performance": {
    "avg_response_time": 150,
    "p95_response_time": 300,
    "p99_response_time": 500
  }
}
```

### Webhooks

#### List Webhooks

```http
GET /api/v1/webhooks
```

#### Create Webhook

```http
POST /api/v1/webhooks
Content-Type: application/json

{
  "url": "https://my-app.com/webhook",
  "events": ["push", "delete"],
  "repository": "myapp/*",
  "secret": "webhook-secret"
}
```

#### Test Webhook

```http
POST /api/v1/webhooks/{id}/test
```

### Audit Logs

#### Get Audit Logs

```http
GET /api/v1/audit
```

**Parameters:**
- `user` (string): Filter by user
- `action` (string): Filter by action
- `from` (datetime): Start time
- `to` (datetime): End time
- `limit` (int): Maximum results

**Response:**
```json
{
  "logs": [
    {
      "id": 1,
      "timestamp": "2024-01-15T12:00:00Z",
      "user": "admin",
      "action": "repository.create",
      "resource": "myapp/frontend",
      "ip_address": "192.168.1.100",
      "user_agent": "docker/20.10.0"
    }
  ],
  "total": 1000,
  "page": 1,
  "limit": 50
}
```

## Error Responses

All APIs use consistent error responses:

```json
{
  "error": {
    "code": "REPOSITORY_NOT_FOUND",
    "message": "Repository myapp/frontend not found",
    "details": {
      "repository": "myapp/frontend"
    }
  }
}
```

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | Authentication required |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `CONFLICT` | 409 | Resource already exists |
| `INVALID_REQUEST` | 400 | Invalid request format |
| `RATE_LIMITED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server error |

## Rate Limiting

API requests are rate limited:
- Default: 1000 requests per hour per user
- Burst: 100 requests per minute
- Headers returned:
  - `X-RateLimit-Limit`: Limit per hour
  - `X-RateLimit-Remaining`: Remaining requests
  - `X-RateLimit-Reset`: Reset time

## Pagination

APIs that return lists support pagination:

**Parameters:**
- `page` (int): Page number (default: 1)
- `limit` (int): Items per page (default: 50, max: 100)

**Response:**
```json
{
  "data": [...],
  "pagination": {
    "page": 1,
    "limit": 50,
    "total": 150,
    "pages": 3
  }
}
```

## Client Libraries

### Docker CLI

Standard Docker commands work with GhostDock:

```bash
# Configure Docker
docker login your-registry.com

# Push/pull images  
docker push your-registry.com/myapp:latest
docker pull your-registry.com/myapp:latest
```

### cURL Examples

#### Push Image (Simplified)

```bash
# 1. Get authentication token
TOKEN=$(curl -s -X POST \
  https://your-registry.com/auth \
  -H "Content-Type: application/json" \
  -d '{"username":"user","password":"pass"}' \
  | jq -r .token)

# 2. Upload blob
BLOB_DIGEST=$(sha256sum layer.tar.gz | cut -d' ' -f1)
curl -X POST \
  -H "Authorization: Bearer $TOKEN" \
  https://your-registry.com/v2/myapp/frontend/blobs/uploads/

# 3. Upload manifest
curl -X PUT \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/vnd.docker.distribution.manifest.v2+json" \
  -d @manifest.json \
  https://your-registry.com/v2/myapp/frontend/manifests/latest
```

### Python Client

```python
import requests

class GhostDockClient:
    def __init__(self, base_url, username, password):
        self.base_url = base_url
        self.session = requests.Session()
        self.authenticate(username, password)
    
    def authenticate(self, username, password):
        response = self.session.post(
            f"{self.base_url}/auth",
            json={"username": username, "password": password}
        )
        token = response.json()["token"]
        self.session.headers["Authorization"] = f"Bearer {token}"
    
    def list_repositories(self):
        response = self.session.get(f"{self.base_url}/v2/_catalog")
        return response.json()["repositories"]
    
    def get_tags(self, repository):
        response = self.session.get(f"{self.base_url}/v2/{repository}/tags/list")
        return response.json()["tags"]

# Usage
client = GhostDockClient("https://your-registry.com", "user", "pass")
repositories = client.list_repositories()
```

For more examples, see the [examples directory](../examples/) in the repository.
