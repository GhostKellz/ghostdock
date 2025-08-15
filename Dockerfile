# Build stage
FROM rust:1.75-alpine as builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

# Set working directory
WORKDIR /usr/src/app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this will be cached)
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache ca-certificates curl

# Create app user
RUN addgroup -g 1001 -S ghostdock && \
    adduser -u 1001 -S ghostdock -G ghostdock

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /usr/src/app/target/release/ghostdock /app/ghostdock

# Copy configuration
COPY config/config.toml /etc/ghostdock/config.toml

# Create directories
RUN mkdir -p /var/lib/ghostdock /var/log/ghostdock && \
    chown -R ghostdock:ghostdock /var/lib/ghostdock /var/log/ghostdock /app

# Switch to app user
USER ghostdock

# Expose ports
EXPOSE 5000 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:5000/health || exit 1

# Set default command
CMD ["/app/ghostdock", "--config", "/etc/ghostdock/config.toml"]
