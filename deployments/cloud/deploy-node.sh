#!/bin/bash
set -euo pipefail

# ============================================================
# Udaya Cloud Node Deployment Script
# Run this script ON EACH cloud VM after provisioning.
# Usage: sudo bash deploy-node.sh <node-number> <rpc-password>
# Example: sudo bash deploy-node.sh 1 "s3cureRPCp@ss"
# ============================================================

NODE_NUM="${1:?Usage: $0 <node-number> <rpc-password>}"
RPC_PASSWORD="${2:?Usage: $0 <node-number> <rpc-password>}"

INSTALL_DIR="/usr/local/bin"
CONF_DIR="/etc/udaya"
DATA_DIR="/var/lib/udaya/node${NODE_NUM}"
LOG_DIR="/var/log/udaya"
SERVICE_DIR="/etc/systemd/system"
REPO_URL="https://github.com/UdayaFoundation/Udaya.git"
BINARY_NAME="Udayad"

echo "=========================================="
echo " Udaya Cloud Node ${NODE_NUM} Deployment"
echo "=========================================="
echo "Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)"

# --------------------------------------------------
# 1. Install system dependencies
# --------------------------------------------------
echo "[1/8] Installing dependencies..."
apt-get update -qq
apt-get install -y -qq \
    curl \
    git \
    jq \
    lsof \
    ufw \
    systemd \
    ca-certificates \
    > /dev/null 2>&1
echo "  Dependencies installed"

# --------------------------------------------------
# 2. Create udaya system user
# --------------------------------------------------
echo "[2/8] Creating system user..."
id -u udaya &>/dev/null || useradd -r -s /bin/false -d /var/lib/udaya -m udaya
echo "  User udaya ready"

# --------------------------------------------------
# 3. Create directories
# --------------------------------------------------
echo "[3/8] Creating directories..."
mkdir -p "${DATA_DIR}"
mkdir -p "${LOG_DIR}"
mkdir -p "${CONF_DIR}"
chown -R udaya:udaya "${DATA_DIR}" "${LOG_DIR}"
echo "  Directories ready"

# --------------------------------------------------
# 4. Deploy binary
# --------------------------------------------------
echo "[4/8] Deploying Udaya binary..."
if [ -f "/tmp/${BINARY_NAME}" ]; then
    cp "/tmp/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    echo "  Binary copied from /tmp"
elif [ -d "/opt/udaya-source" ]; then
    cd /opt/udaya-source
    echo "  Building from source..."
    su - udaya -c "cd /opt/udaya-source && cargo build --release --bin ${BINARY_NAME}" 2>&1 | tail -5
    cp "/opt/udaya-source/target/release/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    echo "  Binary built and installed"
else
    echo "  ERROR: No binary found at /tmp/${BINARY_NAME} or source at /opt/udaya-source"
    echo "  Please transfer the release binary to /tmp/${BINARY_NAME} before running this script"
    exit 1
fi
echo "  Binary deployed to ${INSTALL_DIR}/${BINARY_NAME}"

# --------------------------------------------------
# 5. Deploy configuration
# --------------------------------------------------
echo "[5/8] Deploying configuration..."
CONF_SRC="$(dirname "$0")/testnet-node${NODE_NUM}.conf"
if [ -f "${CONF_SRC}" ]; then
    cp "${CONF_SRC}" "${CONF_DIR}/testnet-node${NODE_NUM}.conf"
else
    echo "  WARNING: Config not found at ${CONF_SRC}, creating minimal config"
    cat > "${CONF_DIR}/testnet-node${NODE_NUM}.conf" <<CONFEOF
[network]
listen_port = $((19797 + NODE_NUM))
external_ip = ""
max_peers = 25
enable_upnp = false
preferred_peers = []
ban_duration_secs = 3600

[storage]
data_dir = "${DATA_DIR}"
prune_blocks = false
prune_target_gb = 20
db_cache_size_mb = 256

[consensus]
network = "testnet"
min_tx_fee = 100
max_block_size = 1000000

[mining]
enable = $([ "${NODE_NUM}" = "1" ] && echo "true" || echo "false")
mine_on_startup = $([ "${NODE_NUM}" = "1" ] && echo "true" || echo "false")
num_miner_threads = 1
coinbase_address = ""

[wallet]
enable = true
wallet_file = "node${NODE_NUM}-wallet.dat"
default_fee_rate = 50

[rpc]
enable = true
listen_addr = "127.0.0.1"
listen_port = $((18331 + NODE_NUM))
username = "udaya"
password = "${RPC_PASSWORD}"
enable_ws = false
ws_port = $((18330 + NODE_NUM))
cors_domains = []
rate_limit_rps = 10
rate_limit_burst = 20
max_request_size_mb = 10
max_connections = 50
restricted_methods = [
    "importprivkey",
    "dumpprivkey",
    "signrawtransactionwithkey",
    "walletpassphrase",
    "walletpassphrasechange",
    "walletlock",
    "encryptwallet"
]

[logging]
level = "info"
file = "/var/log/udaya/node${NODE_NUM}.log"
enable_json = true
CONFEOF
fi

# Replace placeholders with actual external IP
EXTERNAL_IP=$(curl -s ifconfig.me 2>/dev/null || curl -s ip.sb 2>/dev/null || hostname -I | awk '{print $1}')
sed -i "s|__EXTERNAL_IP_NODE${NODE_NUM}__|${EXTERNAL_IP}|g" "${CONF_DIR}/testnet-node${NODE_NUM}.conf"

chown root:root "${CONF_DIR}/testnet-node${NODE_NUM}.conf"
chmod 644 "${CONF_DIR}/testnet-node${NODE_NUM}.conf"
echo "  Configuration deployed for Node ${NODE_NUM} (IP: ${EXTERNAL_IP})"

# --------------------------------------------------
# 6. Install systemd service
# --------------------------------------------------
echo "[6/8] Installing systemd service..."
SERVICE_SRC="$(dirname "$0")/udaya-node@.service"
if [ -f "${SERVICE_SRC}" ]; then
    cp "${SERVICE_SRC}" "${SERVICE_DIR}/udaya-node@${NODE_NUM}.service"
else
    cat > "${SERVICE_DIR}/udaya-node@${NODE_NUM}.service" <<SVCEOF
[Unit]
Description=Udaya Testnet Node ${NODE_NUM}
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=udaya
Group=udaya
ExecStart=${INSTALL_DIR}/${BINARY_NAME} start \\
    --config ${CONF_DIR}/testnet-node${NODE_NUM}.conf \\
    --datadir ${DATA_DIR}

Restart=always
RestartSec=30
StartLimitIntervalSec=300
StartLimitBurst=5

LimitNOFILE=65536
LimitNPROC=4096
LimitAS=infinity
LimitFSIZE=10G

NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=${DATA_DIR} ${LOG_DIR} /tmp

StandardOutput=journal
StandardError=journal
SyslogIdentifier=udaya-node${NODE_NUM}

[Install]
WantedBy=multi-user.target
SVCEOF
fi
systemctl daemon-reload
systemctl enable "udaya-node@${NODE_NUM}.service"
echo "  Systemd service installed"

# --------------------------------------------------
# 7. Configure firewall
# --------------------------------------------------
echo "[7/8] Configuring firewall..."
ufw --force disable 2>/dev/null || true
ufw --force reset 2>/dev/null || true

# Default policies
ufw default deny incoming
ufw default allow outgoing
ufw default allow routed

# SSH (restrict to known admin IPs in production)
ufw allow 22/tcp comment 'SSH'

# P2P port (public - required for peer discovery)
ufw allow $((19797 + NODE_NUM))/tcp comment 'Udaya P2P'

# RPC port (localhost only - no public exposure)
# No ufw rule needed; RPC binds to 127.0.0.1

ufw --force enable
ufw status verbose
echo "  Firewall configured"

# --------------------------------------------------
# 8. Start service
# --------------------------------------------------
echo "[8/8] Starting Udaya node..."
systemctl start "udaya-node@${NODE_NUM}.service"
sleep 5
systemctl status "udaya-node@${NODE_NUM}.service" --no-pager || true

echo ""
echo "=========================================="
echo " Deployment Complete - Node ${NODE_NUM}"
echo "=========================================="
echo "Service:  udaya-node@${NODE_NUM}.service"
echo "Data:     ${DATA_DIR}"
echo "Config:   ${CONF_DIR}/testnet-node${NODE_NUM}.conf"
echo "Logs:     journalctl -u udaya-node@${NODE_NUM}.service -f"
echo "P2P:      port $((19797 + NODE_NUM))"
echo "RPC:      localhost:$((18331 + NODE_NUM))"
echo ""
echo "Verify node:"
echo "  systemctl is-active udaya-node@${NODE_NUM}.service"
echo "  journalctl -u udaya-node@${NODE_NUM}.service -n 50"