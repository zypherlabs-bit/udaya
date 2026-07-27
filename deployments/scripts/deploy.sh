#!/bin/bash
# ============================================================
# Udaya Mainnet Deployment Script
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
LOG_FILE="/var/log/Udaya-deploy.log"

echo "=== Udaya Mainnet Deployment ==="
echo "Starting deployment at $(date)"

# Load environment
export RUST_LOG="${RUST_LOG:-info}"
export udaya_NETWORK="${udaya_NETWORK:-mainnet}"
export udaya_DATA_DIR="${udaya_DATA_DIR:-/data/Udaya}"

# Step 1: Build the binary
echo "[1/6] Building Udaya node..."
cd "$PROJECT_ROOT"
cargo build --release --bin udayad 2>&1 | tee -a "$LOG_FILE"
echo "  ✅ Build complete"

# Step 2: Initialize data directory
echo "[2/6] Initializing data directory..."
mkdir -p "$udaya_DATA_DIR"/{blockchain,logs,wallets}
echo "  ✅ Data directory ready at $udaya_DATA_DIR"

# Step 3: Copy configuration
echo "[3/6] Deploying configuration..."
mkdir -p /etc/Udaya
if [ -f "$PROJECT_ROOT/config/mainnet/udaya.conf" ]; then
    cp "$PROJECT_ROOT/config/mainnet/udaya.conf" /etc/Udaya/udaya.conf
    echo "  ✅ Configuration deployed"
else
    echo "  ⚠️  No mainnet config found, using defaults"
fi

# Step 4: Initialize genesis block
echo "[4/6] Initializing blockchain..."
if [ ! -f "$udaya_DATA_DIR/blockchain/genesis.dat" ]; then
    ./target/release/udayad \
        --network mainnet \
        --datadir "$udaya_DATA_DIR" \
        --config /etc/Udaya/udaya.conf \
        getblockchaininfo 2>&1 | tee -a "$LOG_FILE"
    echo "  ✅ Blockchain initialized"
else
    echo "  ✅ Blockchain already initialized"
fi

# Step 5: Set up systemd service
echo "[5/6] Installing systemd service..."
cat > /etc/systemd/system/udayad.service << 'SERVICEEOF'
[Unit]
Description=Udaya Mainnet Node
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=Udaya
Group=Udaya
ExecStart=/usr/local/bin/udayad start \
    --network mainnet \
    --datadir /data/Udaya \
    --config /etc/Udaya/udaya.conf

Restart=always
RestartSec=30
StartLimitInterval=300
StartLimitBurst=5

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096
LimitAS=infinity
LimitFSIZE=10G

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/data/Udaya /var/log/Udaya

# Logging
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SERVICEEOF

# Create Udaya user if not exists
id -u Udaya &>/dev/null || useradd -r -s /bin/false -d /data/Udaya Udaya
chown -R Udaya:Udaya "$udaya_DATA_DIR"

systemctl daemon-reload
echo "  ✅ Systemd service installed"

# Step 6: Start the node
echo "[6/6] Starting Udaya node..."
systemctl enable udayad
systemctl start udayad

# Wait for startup
sleep 5
if systemctl is-active --quiet udayad; then
    echo "  ✅ Node started successfully"
else
    echo "  ⚠️  Node may not have started. Check: systemctl status udayad"
    journalctl -u udayad -n 20 --no-pager
fi

echo ""
echo "=== Deployment Complete ==="
echo "Node status: $(systemctl is-active udayad)"
echo "Data directory: $udaya_DATA_DIR"
echo "Logs: journalctl -u udayad -f"
echo "RPC endpoint: http://localhost:8332"
echo "P2P endpoint: 0.0.0.0:9798"