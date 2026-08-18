#!/bin/bash
set -euo pipefail

# ============================================================
# Udaya Cloud Node Firewall Configuration
# Run as root on each cloud VM
# Usage: sudo bash udaya-firewall.sh <node-number>
# ============================================================

NODE_NUM="${1:?Usage: $0 <node-number>}"
P2P_PORT=$((19797 + NODE_NUM))

echo "Configuring firewall for Udaya Node ${NODE_NUM}..."

# Reset UFW
ufw --force disable 2>/dev/null || true
ufw --force reset 2>/dev/null || true

# Default deny incoming, allow outgoing
ufw default deny incoming
ufw default allow outgoing
ufw default allow routed

# SSH - restrict to admin IP range in production
# For initial setup, allow SSH. Lock down later with:
# ufw delete allow 22/tcp
# ufw allow from <ADMIN_IP> to any port 22 proto tcp
ufw allow 22/tcp comment 'SSH - restrict to admin IP after setup'

# P2P - must be public for peer discovery and block relay
ufw allow "${P2P_PORT}/tcp" comment "Udaya P2P Node ${NODE_NUM}"

# RPC - binds to localhost only, NO ufw rule needed
# WebSocket - disabled in cloud config, NO ufw rule needed
# Explorer/Faucet/Metrics - not exposed in cloud config

# Enable
ufw --force enable

echo "Firewall status:"
ufw status verbose

echo ""
echo "Post-deployment SSH restriction:"
echo "  ufw delete allow 22/tcp"
echo "  ufw allow from <YOUR_ADMIN_IP> to any port 22 proto tcp"