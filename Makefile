# GhostDock Makefile for common development tasks

.PHONY: help build run test clean dev docker-build docker-run setup-dev

# Default target
help:
	@echo "Available targets:"
	@echo "  build        - Build the release binary"
	@echo "  dev          - Run in development mode"
	@echo "  run          - Run with default configuration"
	@echo "  test         - Run tests"
	@echo "  check        - Check code without building"
	@echo "  clean        - Clean build artifacts"
	@echo "  setup-dev    - Set up development environment"
	@echo "  docker-build - Build Docker image"
	@echo "  docker-run   - Run with Docker Compose"
	@echo "  fmt          - Format code"
	@echo "  clippy       - Run clippy linter"

# Set up development environment
setup-dev:
	@echo "Setting up development environment..."
	mkdir -p dev-data/storage dev-data/logs
	export DATABASE_URL="sqlite:dev-data/ghostdock.db"
	@echo "Development environment ready!"
	@echo "Run 'make dev' to start the development server"

# Build release binary
build:
	@echo "Building GhostDock..."
	export DATABASE_URL="sqlite:ghostdock.db" && cargo build --release

# Run in development mode
dev: setup-dev
	@echo "Starting GhostDock in development mode..."
	export DATABASE_URL="sqlite:dev-data/ghostdock.db" && \
	cargo run -- --config config/dev.toml --dev

# Run with default configuration
run: build
	@echo "Starting GhostDock..."
	export DATABASE_URL="sqlite:ghostdock.db" && \
	./target/release/ghostdock

# Run tests
test:
	@echo "Running tests..."
	export DATABASE_URL="sqlite:test.db" && cargo test

# Check code without building
check:
	@echo "Checking code..."
	export DATABASE_URL="sqlite:ghostdock.db" && cargo check

# Format code
fmt:
	@echo "Formatting code..."
	cargo fmt

# Run clippy linter
clippy:
	@echo "Running clippy..."
	export DATABASE_URL="sqlite:ghostdock.db" && cargo clippy -- -D warnings

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf dev-data/ ghostdock.db test.db

# Build Docker image
docker-build:
	@echo "Building Docker image..."
	docker build -t ghostdock:latest .

# Run with Docker Compose
docker-run:
	@echo "Starting GhostDock with Docker Compose..."
	docker-compose up -d

# Stop Docker Compose
docker-stop:
	@echo "Stopping GhostDock..."
	docker-compose down

# View logs
logs:
	docker-compose logs -f ghostdock

# Database migration (when implemented)
migrate:
	@echo "Running database migrations..."
	export DATABASE_URL="sqlite:ghostdock.db" && \
	cargo run -- migrate

# Show GhostDock status
status:
	@echo "GhostDock Status:"
	@curl -s http://localhost:5000/health | jq '.' || echo "Registry not responding"
	@curl -s http://localhost:8080/ | head -n 1 | grep -o '<title>[^<]*' | sed 's/<title>/Web UI: /' || echo "Web UI not responding"

# Install development dependencies
install-deps:
	@echo "Installing development dependencies..."
	cargo install cargo-watch
	cargo install sqlx-cli
	@echo "Dependencies installed!"
