# Web UI Guide

The GhostDock Registry includes a modern web interface for managing repositories, users, and settings.

## Overview

The web UI provides:
- **Repository Management**: View, search, and manage repositories
- **User Administration**: Manage users, permissions, and access tokens
- **Image Analytics**: View image layers, sizes, and vulnerability scans
- **System Monitoring**: Health checks, metrics, and logs
- **Authentication**: OAuth integration and session management

## Accessing the Web UI

The web interface runs on port 8080 by default:
- Local: http://localhost:8080
- Production: https://your-registry.com

## Authentication

### OAuth Providers

GhostDock supports multiple OAuth providers:

#### GitHub Login
1. Click "Sign in with GitHub"
2. Authorize the application
3. You'll be redirected back to GhostDock

#### Google Login
1. Click "Sign in with Google"
2. Select your Google account
3. Grant necessary permissions

#### Microsoft Login
1. Click "Sign in with Microsoft"
2. Enter your Microsoft/Azure AD credentials
3. Complete the authorization flow

### Local Accounts

For development or isolated environments, you can use local accounts:
1. Click "Create Account"
2. Enter username, email, and password
3. Confirm email (if configured)

## Repository Management

### Repository List

The main dashboard shows all accessible repositories:
- **Search**: Filter repositories by name or tag
- **Sort**: Order by name, size, last updated, or popularity
- **Filter**: Show public, private, or starred repositories

### Repository Details

Click on any repository to view:

#### Overview Tab
- Repository description and README
- Tags and their digest information
- Pull command examples
- Repository statistics

#### Tags Tab
- Complete tag listing with metadata
- Size information for each tag
- Creation and last update timestamps
- Delete tag functionality (with permissions)

#### Layers Tab
- Image layer breakdown
- Layer sharing across tags
- Size optimization opportunities
- Layer vulnerability information

#### Activity Tab
- Push/pull activity logs
- User access history
- Tag creation/deletion events
- Download statistics

#### Settings Tab
- Repository visibility (public/private)
- Access permissions
- Webhook configurations
- Delete repository (dangerous operation)

### Repository Operations

#### Pushing Images

Using Docker CLI:
```bash
# Tag your image
docker tag myapp:latest your-registry.com/myapp:latest

# Push to registry
docker push your-registry.com/myapp:latest
```

#### Pulling Images

```bash
# Pull specific tag
docker pull your-registry.com/myapp:latest

# Pull all tags
docker pull -a your-registry.com/myapp
```

#### Deleting Images

From the web UI:
1. Navigate to repository → Tags
2. Select tags to delete
3. Click "Delete Selected" 
4. Confirm the operation

## User Management

### User Roles

GhostDock has three user roles:

#### Admin
- Full system access
- User management
- System configuration
- All repository operations

#### User
- Create repositories
- Push to owned repositories
- Pull from accessible repositories
- Manage personal access tokens

#### Guest
- Pull from public repositories only
- Read-only access

### User Administration

Admins can manage users:

#### User List
- View all registered users
- Filter by role or status
- Search by username or email

#### User Details
- Edit user information
- Change user role
- Reset passwords
- View user activity

#### User Operations
- Create new users
- Suspend/activate accounts
- Delete users (with data cleanup)
- Bulk operations

## Access Tokens

Personal Access Tokens (PATs) allow programmatic access:

### Creating Tokens

1. Go to Settings → Access Tokens
2. Click "Generate New Token"
3. Set token name and permissions
4. Copy the generated token (shown once)

### Token Permissions

- **Pull**: Download images
- **Push**: Upload images  
- **Delete**: Delete images/repositories
- **Admin**: Full administrative access

### Using Tokens

```bash
# Login with token
echo $GHOSTDOCK_TOKEN | docker login your-registry.com -u username --password-stdin

# Or set in environment
export DOCKER_CONFIG=$HOME/.docker-ghostdock
echo '{"auths":{"your-registry.com":{"auth":"dXNlcm5hbWU6dG9rZW4="}}}' > $HOME/.docker-ghostdock/config.json
```

## System Administration

### Health Dashboard

Monitor system health:
- **Service Status**: Registry, database, storage health
- **Resource Usage**: CPU, memory, disk space
- **Request Metrics**: API calls, response times, errors
- **Storage Metrics**: Blob count, total size, cleanup stats

### Metrics and Monitoring

#### Prometheus Integration

GhostDock exposes metrics at `/metrics`:
- HTTP request metrics
- Database connection pool stats  
- Storage backend performance
- Authentication success/failure rates
- Repository and image statistics

#### Grafana Dashboard

Import the provided Grafana dashboard:
```bash
curl -o ghostdock-dashboard.json \
  https://raw.githubusercontent.com/ghostkellz/ghostdock/main/monitoring/grafana-dashboard.json
```

### System Settings

Configure system-wide settings:

#### Registry Settings
- Default repository visibility
- Maximum image size limits
- Rate limiting configuration
- Storage cleanup policies

#### Authentication Settings
- OAuth provider configuration
- Session timeout settings
- Password complexity requirements
- MFA enforcement

#### Storage Settings
- Storage backend configuration
- Garbage collection schedules
- Backup configurations
- Retention policies

### Logs and Audit

#### Activity Logs
- User authentication events
- Repository access logs
- Administrative actions
- System health events

#### Audit Trail
- User management changes
- Permission modifications
- Repository operations
- Configuration changes

## Advanced Features

### Vulnerability Scanning

Integration with security scanners:
1. Enable scanning in repository settings
2. Images are scanned on push
3. View results in Layers tab
4. Set policies to block vulnerable images

### Repository Mirroring

Mirror external registries:
1. Go to Admin → Mirroring
2. Add upstream registry
3. Configure sync schedule
4. Monitor sync status

### Webhooks

Configure webhooks for events:
1. Repository Settings → Webhooks
2. Add webhook URL
3. Select trigger events:
   - Image pushed
   - Image deleted
   - Repository created
   - Vulnerability found

### API Access

The web UI uses the same APIs available programmatically:
- Docker Registry v2 API
- GhostDock Management API
- GraphQL API (planned)

See [API Documentation](api.md) for details.

## Customization

### Themes

Choose from available themes:
- Light theme (default)
- Dark theme
- High contrast theme
- Custom CSS upload

### Branding

Customize the interface:
1. Admin → Branding
2. Upload company logo
3. Set custom colors
4. Add custom footer text

### Language

Supported languages:
- English (default)
- Spanish
- French
- German
- Japanese
- Chinese (Simplified)

## Mobile Experience

The web UI is responsive and works on mobile devices:
- Touch-friendly interface
- Swipe gestures for navigation
- Optimized for small screens
- Progressive Web App (PWA) support

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `?` | Show help |
| `/` | Focus search |
| `g r` | Go to repositories |
| `g u` | Go to users (admin) |
| `g s` | Go to settings |
| `Escape` | Close modals |

## Troubleshooting

### Common Issues

#### Cannot Login
- Check OAuth configuration
- Verify callback URLs
- Check browser console for errors

#### Cannot See Repositories  
- Check user permissions
- Verify repository visibility settings
- Ensure user is authenticated

#### Cannot Push Images
- Verify Docker is configured for registry
- Check authentication tokens
- Verify push permissions

#### Slow Loading
- Check network connectivity
- Monitor server resources
- Review browser console for errors

### Browser Support

Supported browsers:
- Chrome/Chromium 80+
- Firefox 75+
- Safari 13+
- Edge 80+

### Getting Help

- Check the [FAQ](faq.md)
- Review logs in Admin → Logs
- Check system status in Health Dashboard
- Contact administrator for permission issues
