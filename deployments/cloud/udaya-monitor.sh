#!/bin/bash
set -euo pipefail

# ============================================================
# Udaya Cloud Node Monitoring Script
# Monitors: process, chain height, peer count, disk, memory,
#           RPC health, restarts, errors
# Usage: bash udaya-monitor.sh <node-number> [interval-seconds]
# ============================================================

NODE_NUM="${1:?Usage: $0 <node-number> [interval-seconds>}"
INTERVAL="${2:-30}"
NODE_ID="udaya-node${NODE_NUM}"
RPC_PORT=$((18331 + NODE_NUM))
RPC_USER="${RPC_USER:-udaya}"
RPC_PASS="${RPC_PASSWORD:?Set RPC_PASSWORD env var}"

# Alert state tracking
LAST_HEIGHT=""
LAST_HASH=""
RESTART_COUNT=0
ERROR_COUNT=0

check_node() {
    local timestamp
    timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)

    # 1. Process check
    if ! pgrep -f "udayad start.*--config /etc/udaya/testnet-node${NODE_NUM}" > /dev/null 2>&1; then
        echo "${timestamp} ALERT: ${NODE_ID} process NOT RUNNING"
        RESTART_COUNT=$((RESTART_COUNT + 1))
        echo "${timestamp} RESTART_COUNT=${RESTART_COUNT}"
        return 1
    fi

    # 2. Service status
    local service_status
    service_status=$(systemctl is-active "${NODE_ID}" 2>/dev/null || echo "unknown")
    if [ "${service_status}" != "active" ]; then
        echo "${timestamp} ALERT: ${NODE_ID} service status=${service_status}"
        return 1
    fi

    # 3. RPC health
    local rpc_response
    if ! rpc_response=$(curl -s --max-time 5 -u "${RPC_USER}:${RPC_PASS}" \
        "http://127.0.0.1:${RPC_PORT}/" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"1.0","id":"health","method":"getblockcount","params":[]}' 2>/dev/null); then
        echo "${timestamp} ALERT: ${NODE_ID} RPC unreachable"
        return 1
    fi

    # 4. Parse RPC response
    local height result_error
    height=$(echo "${rpc_response}" | jq -r '.result // empty' 2>/dev/null || echo "")
    result_error=$(echo "${rpc_response}" | jq -r '.error // empty' 2>/dev/null || echo "")

    if [ -z "${height}" ] || [ "${height}" = "null" ]; then
        echo "${timestamp} ALERT: ${NODE_ID} RPC returned invalid response: ${rpc_response}"
        return 1
    fi

    if [ -n "${result_error}" ] && [ "${result_error}" != "null" ]; then
        echo "${timestamp} ALERT: ${NODE_ID} RPC error: ${result_error}"
        ERROR_COUNT=$((ERROR_COUNT + 1))
        return 1
    fi

    # 5. Chain divergence check
    if [ -n "${LAST_HEIGHT}" ] && [ "${height}" -lt "${LAST_HEIGHT}" ]; then
        echo "${timestamp} ALERT: ${NODE_ID} chain divergence! Height dropped from ${LAST_HEIGHT} to ${height}"
        return 1
    fi
    LAST_HEIGHT="${height}"

    # 6. System resources
    local mem_pct disk_pct cpu_load
    mem_pct=$(free | awk '/Mem:/ {printf "%.0f", $3/$2 * 100}')
    disk_pct=$(df /var/lib/udaya | awk 'NR==2 {gsub(/%/,""); print $5}')
    cpu_load=$(uptime | awk -F'load average:' '{print $2}' | awk '{print $1}' | tr -d ',')

    # 7. Disk space alert
    if [ "${disk_pct}" -gt 85 ]; then
        echo "${timestamp} ALERT: ${NODE_ID} disk usage critical: ${disk_pct}%"
        return 1
    fi

    # 8. Memory alert
    if [ "${mem_pct}" -gt 90 ]; then
        echo "${timestamp} ALERT: ${NODE_ID} memory usage critical: ${mem_pct}%"
        return 1
    fi

    # 9. Peer count
    local peer_count
    peer_count=$(curl -s --max-time 5 -u "${RPC_USER}:${RPC_PASS}" \
        "http://127.0.0.1:${RPC_PORT}/" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"1.0","id":"peers","method":"getpeerinfo","params":[]}' 2>/dev/null | jq 'length' 2>/dev/null || echo "0")

    # 10. Log recent errors
    local recent_errors
    recent_errors=$(journalctl -u "${NODE_ID}" --since "5 minutes ago" --no-pager -q 2>/dev/null | grep -ci "error\|panic\|fatal" || echo "0")

    if [ "${recent_errors}" -gt 10 ]; then
        echo "${timestamp} WARN: ${NODE_ID} ${recent_errors} errors in last 5 minutes"
        ERROR_COUNT=$((ERROR_COUNT + 1))
    fi

    # 11. Output status line
    echo "${timestamp} OK: ${NODE_ID} height=${height} peers=${peer_count} disk=${disk_pct}% mem=${mem_pct}% cpu=${cpu_load} errors=${recent_errors} restarts=${RESTART_COUNT}"
}

# Main loop
echo "Starting monitor for ${NODE_ID} (interval: ${INTERVAL}s)"
echo "Press Ctrl+C to stop"
echo ""

while true; do
    check_node || true
    sleep "${INTERVAL}"
done