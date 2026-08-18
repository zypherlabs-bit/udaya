# Udaya Node Docker Image
# Multi-stage build for minimal production image
FROM rust:1.75-slim-bullseye AS builder

# Install dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY src/Cargo.toml src/Cargo.toml
COPY src/core/Cargo.toml src/core/Cargo.toml
COPY src/p2p/Cargo.toml src/p2p/Cargo.toml
COPY src/storage/Cargo.toml src/storage/Cargo.toml
COPY src/mining/Cargo.toml src/mining/Cargo.toml
COPY src/wallet/Cargo.toml src/wallet/Cargo.toml
COPY src/api/Cargo.toml src/api/Cargo.toml
COPY src/explorer/Cargo.toml src/explorer/Cargo.toml
COPY src/faucet/Cargo.toml src/faucet/Cargo.toml
COPY src/pool-server/Cargo.toml src/pool-server/Cargo.toml
COPY src/mempool/Cargo.toml src/mempool/Cargo.toml
COPY src/stability-tests/Cargo.toml src/stability-tests/Cargo.toml

# Create dummy source files to build dependencies
RUN mkdir -p src/core src/p2p src/storage src/mining src/wallet src/api src/explorer src/faucet src/pool-server src/mempool src/stability-tests
RUN touch src/main.rs src/core/lib.rs src/p2p/lib.rs src/storage/lib.rs src/mining/lib.rs src/wallet/lib.rs src/api/lib.rs src/explorer/lib.rs src/faucet/lib.rs src/pool-server/lib.rs src/mempool/lib.rs src/stability-tests/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release --bin udayad 2>&1 | tail -20

# Copy actual source code
COPY src/ src/

# Build the actual binary
RUN touch src/main.rs && cargo build --release --bin udayad 2>&1 | tail -20

# Production image
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    procps \
    net-tools \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false -m -d /data/udaya udaya

# Copy binary from builder
COPY --from=builder /build/target/release/udayad /usr/local/bin/udayad
COPY --from=builder /build/target/release/udaya-explorer /usr/local/bin/udaya-explorer
COPY --from=builder /build/target/release/udaya-faucet /usr/local/bin/udaya-faucet
COPY --from=builder /build/target/release/udaya-pool /usr/local/bin/udaya-pool

# Create data directories
RUN mkdir -p /data/udaya/{blockchain,logs,wallets} && \
    chown -R udaya:udaya /data/udaya

# Expose ports
# P2P: 9798 (mainnet), 19798 (testnet)
# RPC: 8332 (mainnet), 18332 (testnet)
# WebSocket: 8333 (mainnet), 18333 (testnet)
# Explorer: 8080
# Faucet: 8081
# Metrics: 9090
EXPOSE 9798 19798 8332 18332 8333 18333 8080 8081 9090

# Set volume for persistent data
VOLUME ["/data/udaya"]

# Switch to non-root user
USER udaya

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:18332/health || exit 1

# Default command
ENTRYPOINT ["/usr/local/bin/udayad"]
CMD ["--config", "/etc/udaya/udaya-testnet.conf", "start"]