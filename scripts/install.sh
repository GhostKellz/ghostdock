#!/bin/bash
set -e

# GhostDock Registry Installation Script
# This script installs GhostDock Registry on Linux systems

INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/ghostdock"
DATA_DIR="/var/lib/ghostdock"
LOG_DIR="/var/log/ghostdock"
SERVICE_FILE="/etc/systemd/system/ghostdock.service"
USER="ghostdock"
GROUP="ghostdock"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_error "This script must be run as root"
        exit 1
    fi
}

# Detect system architecture
detect_arch() {
    case $(uname -m) in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        armv7l)
            ARCH="armv7"
            ;;
        *)
            print_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
}

# Detect Linux distribution
detect_distro() {
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        DISTRO=$ID
        VERSION=$VERSION_ID
    else
        print_error "Cannot detect Linux distribution"
        exit 1
    fi
    
    print_status "Detected: $PRETTY_NAME"
}

# Install system dependencies
install_dependencies() {
    print_status "Installing system dependencies..."
    
    case $DISTRO in
        ubuntu|debian)
            apt-get update
            apt-get install -y curl wget tar systemd sqlite3
            ;;
        centos|rhel|fedora)
            if command -v dnf &> /dev/null; then
                dnf install -y curl wget tar systemd sqlite
            else
                yum install -y curl wget tar systemd sqlite
            fi
            ;;
        alpine)
            apk add --no-cache curl wget tar openrc sqlite
            ;;
        *)
            print_warning "Unknown distribution. Please install curl, wget, tar, and systemd manually."
            ;;
    esac
}

# Create system user
create_user() {
    print_status "Creating system user: $USER"
    
    if id "$USER" &>/dev/null; then
        print_warning "User $USER already exists"
    else
        useradd --system --shell /bin/false --home-dir $DATA_DIR --create-home $USER
        print_success "Created user: $USER"
    fi
}

# Create directories
create_directories() {
    print_status "Creating directories..."
    
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$DATA_DIR"
    mkdir -p "$LOG_DIR"
    mkdir -p "$DATA_DIR/storage"
    
    # Set permissions
    chown -R $USER:$GROUP "$DATA_DIR"
    chown -R $USER:$GROUP "$LOG_DIR"
    chmod 755 "$CONFIG_DIR"
    chmod 750 "$DATA_DIR"
    chmod 750 "$LOG_DIR"
    
    print_success "Created directories"
}

# Download and install binary
install_binary() {
    print_status "Installing GhostDock binary..."
    
    # For now, we'll assume the binary is built locally
    # In a real scenario, this would download from GitHub releases
    
    if [[ -f "target/release/ghostdock" ]]; then
        cp target/release/ghostdock "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/ghostdock"
        print_success "Installed binary to $INSTALL_DIR/ghostdock"
    else
        print_error "Binary not found. Please run 'cargo build --release' first"
        exit 1
    fi
}

# Install configuration
install_config() {
    print_status "Installing configuration..."
    
    if [[ ! -f "$CONFIG_DIR/config.toml" ]]; then
        cp config/config.toml "$CONFIG_DIR/"
        
        # Generate JWT secret
        JWT_SECRET=$(openssl rand -hex 32)
        sed -i "s/your-secret-key-here/$JWT_SECRET/" "$CONFIG_DIR/config.toml"
        
        # Update paths
        sed -i "s|\\./storage|$DATA_DIR/storage|" "$CONFIG_DIR/config.toml"
        sed -i "s|ghostdock\\.db|$DATA_DIR/ghostdock.db|" "$CONFIG_DIR/config.toml"
        sed -i "s|ghostdock\\.log|$LOG_DIR/ghostdock.log|" "$CONFIG_DIR/config.toml"
        
        chown root:$GROUP "$CONFIG_DIR/config.toml"
        chmod 640 "$CONFIG_DIR/config.toml"
        
        print_success "Installed configuration"
    else
        print_warning "Configuration already exists at $CONFIG_DIR/config.toml"
    fi
}

# Install systemd service
install_service() {
    print_status "Installing systemd service..."
    
    cp scripts/ghostdock.service "$SERVICE_FILE"
    systemctl daemon-reload
    systemctl enable ghostdock
    
    print_success "Installed systemd service"
}

# Initialize database
init_database() {
    print_status "Initializing database..."
    
    export DATABASE_URL="sqlite:$DATA_DIR/ghostdock.db"
    sudo -u $USER DATABASE_URL="$DATABASE_URL" "$INSTALL_DIR/ghostdock" --config "$CONFIG_DIR/config.toml" &
    GHOSTDOCK_PID=$!
    sleep 2
    kill $GHOSTDOCK_PID 2>/dev/null || true
    wait $GHOSTDOCK_PID 2>/dev/null || true
    
    print_success "Database initialized"
}

# Configure firewall
configure_firewall() {
    print_status "Configuring firewall..."
    
    if command -v ufw &> /dev/null; then
        ufw allow 5000/tcp comment "GhostDock Registry"
        ufw allow 8080/tcp comment "GhostDock Web UI"
        print_success "UFW rules added"
    elif command -v firewall-cmd &> /dev/null; then
        firewall-cmd --permanent --add-port=5000/tcp
        firewall-cmd --permanent --add-port=8080/tcp
        firewall-cmd --reload
        print_success "Firewall rules added"
    else
        print_warning "No firewall detected. Please open ports 5000 and 8080 manually"
    fi
}

# Start service
start_service() {
    print_status "Starting GhostDock service..."
    
    systemctl start ghostdock
    sleep 3
    
    if systemctl is-active --quiet ghostdock; then
        print_success "GhostDock is running"
    else
        print_error "Failed to start GhostDock"
        systemctl status ghostdock
        exit 1
    fi
}

# Main installation
main() {
    print_status "Starting GhostDock Registry installation..."
    
    check_root
    detect_arch
    detect_distro
    install_dependencies
    create_user
    create_directories
    install_binary
    install_config
    install_service
    init_database
    configure_firewall
    start_service
    
    print_success "GhostDock Registry installation completed!"
    echo ""
    echo "Registry API: http://localhost:5000"
    echo "Web UI: http://localhost:8080"
    echo ""
    echo "Configuration: $CONFIG_DIR/config.toml"
    echo "Data directory: $DATA_DIR"
    echo "Logs: journalctl -u ghostdock -f"
    echo ""
    echo "To configure OAuth providers, edit $CONFIG_DIR/config.toml and restart:"
    echo "  systemctl restart ghostdock"
    echo ""
    echo "For Docker client configuration, add to /etc/docker/daemon.json:"
    echo '  {"insecure-registries": ["localhost:5000"]}'
    echo "Then restart Docker: systemctl restart docker"
}

# Run installation
main "$@"
