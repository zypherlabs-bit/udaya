#!/bin/bash
# ============================================================
# Udaya Testnet Deployment Script
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
INSTALL_DIR="/usr/local/bin"
CONF_DIR="/etc/udaya"
DATA_DIR="/var/lib/udaya"
LOG_DIR="/var/log/udaya"
SERVICE_DIR="/etc/systemd/system"

echo "=== Udaya Testnet Deployment ==="
echo "Starting deployment at $(date)"

# Step 1: Build the binary
echo "[1/7] Building Udaya node..."
cd "$PROJECT_ROOT"
cargo build --release --bin udayad 2>&1 | tail -5
echo "  Build complete"

# Step 2: Create udaya user
echo "[2/7] Creating udaya system user..."
id -u udaya &>/dev/null || useradd -r -s /bin/false -d "$DATA_DIR" udaya
echo "  User ready"

# Step 3: Create directories
echo "[3/7] Creating directories..."
mkdir -p "$DATA_DIR"/{node1,node2,node3}
mkdir -p "$LOG_DIR"
mkdir -p "$CONF_DIR"
chown -R udaya:udaya "$DATA_DIR" "$LOG_DIR"
echo "  Directories ready"

# Step 4: Deploy binary
echo "[4/7] Deploying binary..."
cp "$PROJECT_ROOT/target/release/udayad" "$INSTALL_DIR/udayad"
chmod +x "$INSTALL_DIR/udayad"
echo "  Binary deployed to $INSTALL_DIR/udayad"

# Step 5: Deploy configurations
echo "[5/7] Deploying configurations..."
cp "$PROJECT_ROOT/config/testnet/seed-us.conf" "$CONF_DIR/testnet-node1.conf"
cp "$PROJECT_ROOT/config/testnet/seed-eu.conf" "$CONF_DIR/testnet-node2.conf"
cp "$PROJECT_ROOT/config/testnet/bitfury-testnet.conf" "$CONF_DIR/testnet-node3.conf"
chown root:root "$CONF_DIR"/*
chmod 644 "$CONF_DIR"/*
echo "  Configurations deployed"

# Step 6: Install systemd services
echo "[6/7] Installing systemd services..."
cp "$SCRIPT_DIR/udaya-node1.service" "$SERVICE_DIR/"
cp "$SCRIPT_DIR/udaya-node2.service" "$SERVICE_DIR/"
cp "$SCRIPT_DIR/udaya-node3.service" "$SERVICE_DIR/"
systemctl daemon-reload
echo "  Systemd services installed"

# Step 7: Enable and start services
echo "[7/7] Starting services..."
systemctl enable udaya-node1 udaya-node2 udaya-node3
systemctl start udaya-node1 udaya-node2 udaya-node3

sleep 5
echo ""
echo "=== Deployment Complete ==="
echo "Node 1: $(systemctl is-active udaya-node1)"
echo "Node 2: $(systemctl is-active udaya-node2)"
echo "Node 3: $(systemctl is-active udaya-node3)"
echo ""
echo "Data directory: $DATA_DIR"
echo "Logs: journalctl -u udaya-node1 -f"
echo "RPC: http://localhost:18332"
