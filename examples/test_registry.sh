#!/bin/bash

# GhostDock Registry CLI Example Scripts
# This file contains example commands for interacting with GhostDock Registry

set -e

# Configuration
REGISTRY_URL="localhost:5000"
WEB_URL="localhost:8080"
USERNAME="admin"
PASSWORD="password"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_section() {
    echo -e "\n${BLUE}=== $1 ===${NC}"
}

print_command() {
    echo -e "${YELLOW}$ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check dependencies
check_dependencies() {
    print_section "Checking Dependencies"
    
    if ! command_exists docker; then
        print_error "Docker is not installed"
        exit 1
    fi
    print_success "Docker found"
    
    if ! command_exists curl; then
        print_error "curl is not installed"
        exit 1
    fi
    print_success "curl found"
    
    if ! command_exists jq; then
        echo "jq not found, installing..."
        if command_exists apt-get; then
            sudo apt-get update && sudo apt-get install -y jq
        elif command_exists yum; then
            sudo yum install -y jq
        elif command_exists brew; then
            brew install jq
        else
            print_error "Please install jq manually"
            exit 1
        fi
    fi
    print_success "jq found"
}

# Test registry connectivity
test_connectivity() {
    print_section "Testing Connectivity"
    
    print_command "curl -s http://$REGISTRY_URL/v2/"
    if curl -s "http://$REGISTRY_URL/v2/" > /dev/null; then
        print_success "Registry API is accessible"
    else
        print_error "Cannot connect to registry API"
        return 1
    fi
    
    print_command "curl -s http://$WEB_URL/"
    if curl -s "http://$WEB_URL/" > /dev/null; then
        print_success "Web UI is accessible"
    else
        print_error "Cannot connect to web UI"
        return 1
    fi
}

# Configure Docker daemon
configure_docker() {
    print_section "Configuring Docker"
    
    # Check if registry is in insecure registries
    if docker info 2>/dev/null | grep -q "$REGISTRY_URL"; then
        print_success "Registry already configured"
        return 0
    fi
    
    echo "Adding registry to Docker daemon configuration..."
    
    DAEMON_CONFIG="/etc/docker/daemon.json"
    TEMP_CONFIG="/tmp/daemon.json"
    
    if [[ -f "$DAEMON_CONFIG" ]]; then
        # Update existing config
        jq --arg registry "$REGISTRY_URL" \
           '.["insecure-registries"] = (.["insecure-registries"] // []) + [$registry] | .["insecure-registries"] |= unique' \
           "$DAEMON_CONFIG" > "$TEMP_CONFIG"
    else
        # Create new config
        jq -n --arg registry "$REGISTRY_URL" \
           '{"insecure-registries": [$registry]}' > "$TEMP_CONFIG"
    fi
    
    sudo mv "$TEMP_CONFIG" "$DAEMON_CONFIG"
    sudo systemctl restart docker
    
    print_success "Docker configured and restarted"
}

# Basic registry operations
test_basic_operations() {
    print_section "Testing Basic Registry Operations"
    
    # Pull a test image
    print_command "docker pull alpine:latest"
    docker pull alpine:latest
    print_success "Pulled alpine:latest"
    
    # Tag for local registry
    print_command "docker tag alpine:latest $REGISTRY_URL/test/alpine:latest"
    docker tag alpine:latest "$REGISTRY_URL/test/alpine:latest"
    print_success "Tagged image"
    
    # Push to registry
    print_command "docker push $REGISTRY_URL/test/alpine:latest"
    if docker push "$REGISTRY_URL/test/alpine:latest"; then
        print_success "Pushed image to registry"
    else
        print_error "Failed to push image"
        return 1
    fi
    
    # Remove local image
    print_command "docker rmi $REGISTRY_URL/test/alpine:latest"
    docker rmi "$REGISTRY_URL/test/alpine:latest"
    print_success "Removed local image"
    
    # Pull from registry
    print_command "docker pull $REGISTRY_URL/test/alpine:latest"
    if docker pull "$REGISTRY_URL/test/alpine:latest"; then
        print_success "Pulled image from registry"
    else
        print_error "Failed to pull image"
        return 1
    fi
}

# Test API endpoints
test_api_endpoints() {
    print_section "Testing API Endpoints"
    
    # Get API version
    print_command "curl -s http://$REGISTRY_URL/v2/"
    API_RESPONSE=$(curl -s "http://$REGISTRY_URL/v2/")
    echo "$API_RESPONSE" | jq '.'
    print_success "API version retrieved"
    
    # List repositories
    print_command "curl -s http://$REGISTRY_URL/v2/_catalog"
    CATALOG=$(curl -s "http://$REGISTRY_URL/v2/_catalog")
    echo "$CATALOG" | jq '.'
    
    # Extract repository name
    REPO=$(echo "$CATALOG" | jq -r '.repositories[0]')
    if [[ "$REPO" != "null" ]]; then
        print_success "Found repository: $REPO"
        
        # List tags
        print_command "curl -s http://$REGISTRY_URL/v2/$REPO/tags/list"
        TAGS=$(curl -s "http://$REGISTRY_URL/v2/$REPO/tags/list")
        echo "$TAGS" | jq '.'
        print_success "Listed tags for $REPO"
        
        # Get manifest
        TAG=$(echo "$TAGS" | jq -r '.tags[0]')
        if [[ "$TAG" != "null" ]]; then
            print_command "curl -s -H 'Accept: application/vnd.docker.distribution.manifest.v2+json' http://$REGISTRY_URL/v2/$REPO/manifests/$TAG"
            MANIFEST=$(curl -s -H "Accept: application/vnd.docker.distribution.manifest.v2+json" \
                      "http://$REGISTRY_URL/v2/$REPO/manifests/$TAG")
            echo "$MANIFEST" | jq '.schemaVersion, .mediaType, .config.digest' 2>/dev/null || echo "$MANIFEST"
            print_success "Retrieved manifest for $REPO:$TAG"
        fi
    fi
}

# Test web UI endpoints
test_web_endpoints() {
    print_section "Testing Web UI Endpoints"
    
    print_command "curl -s http://$WEB_URL/health"
    HEALTH=$(curl -s "http://$WEB_URL/health")
    echo "$HEALTH" | jq '.' 2>/dev/null || echo "$HEALTH"
    print_success "Health endpoint accessible"
    
    print_command "curl -s http://$WEB_URL/api/v1/health"
    API_HEALTH=$(curl -s "http://$WEB_URL/api/v1/health")
    echo "$API_HEALTH" | jq '.' 2>/dev/null || echo "$API_HEALTH"
    print_success "Management API health endpoint accessible"
}

# Performance test
performance_test() {
    print_section "Basic Performance Test"
    
    echo "Testing concurrent pulls..."
    
    # Run 5 concurrent pulls
    for i in {1..5}; do
        (
            time docker pull "$REGISTRY_URL/test/alpine:latest" > /dev/null 2>&1
        ) &
    done
    wait
    
    print_success "Concurrent pulls completed"
}

# Cleanup
cleanup() {
    print_section "Cleanup"
    
    print_command "docker rmi $REGISTRY_URL/test/alpine:latest"
    docker rmi "$REGISTRY_URL/test/alpine:latest" 2>/dev/null || true
    
    print_command "docker rmi alpine:latest" 
    docker rmi alpine:latest 2>/dev/null || true
    
    print_success "Cleanup completed"
}

# Main function
main() {
    echo -e "${BLUE}GhostDock Registry Test Suite${NC}"
    echo "Registry: $REGISTRY_URL"
    echo "Web UI: $WEB_URL"
    
    check_dependencies
    test_connectivity
    
    # Ask user if they want to configure Docker
    read -p "Configure Docker daemon for insecure registry? (y/N): " -r
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        configure_docker
    fi
    
    test_basic_operations
    test_api_endpoints
    test_web_endpoints
    performance_test
    
    # Ask user if they want to cleanup
    read -p "Run cleanup? (Y/n): " -r
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        cleanup
    fi
    
    print_section "Test Suite Complete"
    print_success "All tests passed!"
}

# Handle script arguments
case "${1:-}" in
    connectivity)
        test_connectivity
        ;;
    docker-config)
        configure_docker
        ;;
    basic)
        test_basic_operations
        ;;
    api)
        test_api_endpoints
        ;;
    web)
        test_web_endpoints
        ;;
    performance)
        performance_test
        ;;
    cleanup)
        cleanup
        ;;
    *)
        main
        ;;
esac
