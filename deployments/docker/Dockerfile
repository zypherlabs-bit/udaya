# Udaya Production Docker Image
# Multi-stage build for minimal attack surface

# Build stage
FROM rust:1.75-slim-bullseye AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .

RUN cargo build --release --bin udayad

# Runtime stage
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    libssl1.1 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create Udaya user
RUN groupadd -r Udaya && useradd -r -g Udaya -m -d /data/Udaya Udaya

# Copy binary
COPY --from=builder /build/target/release/udayad /usr/local/bin/udayad

# Create directories
RUN mkdir -p /data/Udaya/mainnet /data/Udaya/testnet /var/log/Udaya
RUN chown -R Udaya:Udaya /data/Udaya /var/log/Udaya

# Copy configurations
COPY config/mainnet/udaya-mainnet.conf /etc/Udaya/udaya.conf

# Expose P2P, RPC, WebSocket, and REST ports
EXPOSE 9798
EXPOSE 8332
EXPOSE 8333
EXPOSE 8334

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8332/health || exit 1

USER Udaya
WORKDIR /data/Udaya

ENTRYPOINT ["udayad"]
CMD ["-c", "/etc/Udaya/udaya.conf", "start"]