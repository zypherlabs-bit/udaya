#!/bin/bash
# Udaya Testnet Deployment Script
# Deploys 5 independent testnet nodes across different providers/regions

set -e

# Configuration
TESTNET_VERSION="1.0.0"
TESTNET_DATA_DIR="/var/lib/udaya-testnet"
TESTNET_LOG_DIR="/var/log/Udaya"
TESTNET_USER="Udaya"
SEED_NODES=("seed-us.Udaya.org" "seed-eu.Udaya.org" "seed-ap.Udaya.org")

echo "╔══════════════════════════════════════════════════╗"
echo "║  Udaya Testnet Deployment v${TESTNET_VERSION}            ║"
echo "╚══════════════════════════════════════════════════╝"

# Node Inventory
declare -A NODES
NODES["node1"]="US-East,AWS,us-east-1,54.1.1.1"
NODES["node2"]="US-West,DigitalOcean,sfo3,54.2.2.2" 
NODES["node3"]="Europe,Hetzner,fra1,54.3.3.3"
NODES["node4"]="Asia-Pacific,Linode,sgp1,54.4.4.4"
NODES["node5"]="Global,Vultr,lon1,54.5.5.5"

create_node_config() {
    local node_name=$1
    local region=$2
    local provider=$3
    local ip=$4
    local port=$5
    local rpc_port=$6

    mkdir -p "${TESTNET_DATA_DIR}/${node_name}"
    
    cat > "${TESTNET_DATA_DIR}/${node_name}/udaya.conf" << EOF
# Udaya Testnet Node Configuration
# Node: ${node_name}
# Region: ${region} (${provider})
# IP: ${ip}

[network]
listen_port = ${port}
external_ip = "${ip}"
max_peers = 250
enable_upnp = false
preferred_peers = [
    "seed-us.Udaya.org:9798",
    "seed-eu.Udaya.org:9798", 
    "seed-ap.Udaya.org:9798",
]
ban_duration_secs = 3600

[storage]
data_dir = "${TESTNET_DATA_DIR}/${node_name}/data"
prune_blocks = false
prune_target_gb = 20
db_cache_size_mb = 1024

[consensus]
network = "testnet"
min_tx_fee = 100
max_block_size = 1000000

[mining]
enable = false
mine_on_startup = false
num_miner_threads = 1

[wallet]
enable = true
wallet_file = "${TESTNET_DATA_DIR}/${node_name}/wallet.dat"
default_fee_rate = 50

[rpc]
enable = true
listen_addr = "0.0.0.0"
listen_port = ${rpc_port}
username = "${RPC_USER:-Udaya}"
password = "${RPC_PASSWORD}"
enable_ws = true
ws_port = $((rpc_port + 1))
cors_domains = ["*"]

[logging]
level = "info"
file = "${TESTNET_LOG_DIR}/${node_name}.log"
enable_json = true
EOF

    echo "  ✅ Configuration created for ${node_name} (${region})"
}

create_systemd_service() {
    local node_name=$1
    local port=$2

    cat > "/etc/systemd/system/Udaya-${node_name}.service" << EOF
[Unit]
Description=Udaya Testnet Node - ${node_name}
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=${TESTNET_USER}
Group=${TESTNET_USER}
ExecStart=/usr/local/bin/udayad --config ${TESTNET_DATA_DIR}/${node_name}/udaya.conf start
Restart=always
RestartSec=30
TimeoutStopSec=60
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable "Udaya-${node_name}"
    systemctl start "Udaya-${node_name}"
    echo "  ✅ Systemd service created for ${node_name}"
}

create_monitoring_config() {
    cat > "${TESTNET_DATA_DIR}/prometheus.yml" << EOF
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'udaya-testnet'
    scrape_interval: 10s
    static_configs:
      - targets:
EOF

    for node in "${!NODES[@]}"; do
        IFS=',' read -r region provider ip <<< "${NODES[$node]}"
        local rpc_port=$((18332 + ${node:4:1}))
        echo "        - '${ip}:${rpc_port}'" >> "${TESTNET_DATA_DIR}/prometheus.yml"
    done
}

# Main deployment
echo ""
echo "Step 1: Creating user and directories"
useradd -r -s /bin/false ${TESTNET_USER} 2>/dev/null || true
mkdir -p ${TESTNET_DATA_DIR} ${TESTNET_LOG_DIR}

echo ""
echo "Step 2: Deploying ${#NODES[@]} testnet nodes"
for node in "${!NODES[@]}"; do
    IFS=',' read -r region provider ip <<< "${NODES[$node]}"
    local port=$((9798 + ${node:4:1}))
    local rpc_port=$((18332 + ${node:4:1}))
    
    echo ""
    echo "Node: ${node}"
    echo "  Region: ${region}"
    echo "  Provider: ${provider}"
    echo "  IP: ${ip}"
    echo "  P2P Port: ${port}"
    echo "  RPC Port: ${rpc_port}"
    
    create_node_config "${node}" "${region}" "${provider}" "${ip}" "${port}" "${rpc_port}"
    create_systemd_service "${node}" "${port}"
done

echo ""
echo "Step 3: Setting up monitoring"
create_monitoring_config

echo ""
echo "Step 4: Node Inventory"
echo "═══════════════════════════════════════════════════"
echo "ID    | Region          | Provider       | IP          | P2P Port | RPC Port"
echo "───────────────────────────────────────────────────────────────────────────────"
for node in "${!NODES[@]}"; do
    IFS=',' read -r region provider ip <<< "${NODES[$node]}"
    local port=$((9798 + ${node:4:1}))
    local rpc_port=$((18332 + ${node:4:1}))
    printf "%-5s | %-15s | %-14s | %-11s | %-8s | %s\n" \
        "${node}" "${region}" "${provider}" "${ip}" "${port}" "${rpc_port}"
done

echo ""
echo "Step 5: Verifying node synchronization"
sleep 5

for node in "${!NODES[@]}"; do
    local rpc_port=$((18332 + ${node:4:1}))
    if curl -s -f "http://localhost:${rpc_port}/health" > /dev/null 2>&1; then
        echo "  ✅ ${node}: RUNNING"
    else
        echo "  ❌ ${node}: NOT RESPONDING"
    fi
done

echo ""
echo "╔══════════════════════════════════════════════════╗"
echo "║  Udaya Testnet Deployment Complete             ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""
echo "Seed Nodes:"
for seed in "${SEED_NODES[@]}"; do
    echo "  - ${seed}:9798"
done
echo ""
echo "Monitor: http://localhost:9090"
echo "Explorer: http://localhost:8080"
echo "Faucet: http://localhost:8081"
echo "Pool: http://localhost:9090"
echo ""
echo "To check status: systemctl status Udaya-*"
echo "To view logs: journalctl -u Udaya-node1 -f"