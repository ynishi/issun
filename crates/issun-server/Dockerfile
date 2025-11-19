# Multi-stage build for ISSUN relay server
# Stage 1: Builder
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/issun/Cargo.toml crates/issun/
COPY crates/issun-macros/Cargo.toml crates/issun-macros/
COPY crates/issun-server/Cargo.toml crates/issun-server/

# Create dummy source files to cache dependencies
RUN mkdir -p crates/issun/src crates/issun-macros/src crates/issun-server/src && \
    echo "fn main() {}" > crates/issun-server/src/main.rs && \
    echo "pub fn dummy() {}" > crates/issun/src/lib.rs && \
    echo "pub fn dummy() {}" > crates/issun-macros/src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release -p issun-server

# Remove dummy files
RUN rm -rf crates/*/src

# Copy actual source code
COPY crates/ crates/

# Build release binary
RUN cargo build --release -p issun-server

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 issun

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/issun-server /usr/local/bin/issun-server

# Create directories for certs
RUN mkdir -p /app/certs && chown -R issun:issun /app

USER issun

# Environment variables with defaults
ENV ISSUN_BIND_ADDR=0.0.0.0:5000 \
    ISSUN_CERT_PATH=/app/certs/cert.pem \
    ISSUN_KEY_PATH=/app/certs/key.pem \
    ISSUN_MAX_CLIENTS=1000 \
    ISSUN_HEARTBEAT_INTERVAL=5 \
    RUST_LOG=issun_server=info

# Expose QUIC port (UDP)
EXPOSE 5000/udp

# Health check (basic process check)
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD pgrep -x issun-server > /dev/null || exit 1

CMD ["issun-server"]
