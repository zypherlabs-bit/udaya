#!/bin/bash
# Udaya Node Health Check
# Returns 0 if healthy, 1 if unhealthy

set -e

# Check if process is running
if ! pgrep -x "udayad" > /dev/null; then
    echo "ERROR: udayad process not running"
    exit 1
fi

# Check RPC endpoint
if command -v curl &> /dev/null; then
    RPC_STATUS=$(curl -s -o /dev/null -w "%{http_code}" \
        --connect-timeout 5 \
        --max-time 10 \
        http://localhost:8332/health 2>/dev/null || echo "000")
    
    if [ "$RPC_STATUS" != "200" ]; then
        echo "WARNING: RPC endpoint returned status $RPC_STATUS"
    fi
fi

# Check disk space
DATA_DIR="/var/lib/Udaya/data"
if [ -d "$DATA_DIR" ]; then
    DISK_USAGE=$(df "$DATA_DIR" | tail -1 | awk '{print $5}' | sed 's/%//')
    if [ "$DISK_USAGE" -gt 90 ]; then
        echo "WARNING: Disk usage at ${DISK_USAGE}%"
    fi
fi

# Check memory
MEM_TOTAL=$(free -m | awk '/^Mem:/{print $2}')
MEM_USED=$(free -m | awk '/^Mem:/{print $3}')
MEM_PCT=$((MEM_USED * 100 / MEM_TOTAL))
if [ "$MEM_PCT" -gt 95 ]; then
    echo "WARNING: Memory usage at ${MEM_PCT}%"
fi

echo "Udaya node is healthy"
exit 0