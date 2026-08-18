# Udaya Phase 6 — Cloud Deployment Guide

## Overview

This directory contains all artifacts required to deploy the Udaya public testnet to **Oracle Cloud Always Free** ARM VMs.

## Architecture

```
Cloud Node 1 (10.0.1.10)  ──P2P──  Cloud Node 2 (10.0.1.11)
      │                              │
      └──────────────P2P─────────────┘
                              │
                       Cloud Node 3 (10.0.1.12)
```

Each node runs:
- `Udayad` daemon (testnet mode)
- `systemd` service with auto-restart
- Persistent RocksDB storage
- RPC on localhost (not public)
- P2P on public port

## Files

| File | Purpose |
|------|---------|
| `udaya-node@.service` | Templated systemd unit |
| `testnet-node1.conf` | Node 1 config (mining enabled) |
| `testnet-node2.conf` | Node 2 config (sync node) |
| `testnet-node3.conf` | Node 3 config (sync node) |
| `deploy-node.sh` | Per-VM deployment script |
| `udaya-firewall.sh` | UFW firewall rules |
| `udaya-monitor.sh` | Lightweight health monitor |
| `udaya-logrotate` | Log rotation config |
| `provision-oracle.py` | Oracle Cloud provisioning script |
| `README.md` | This file |

## Prerequisites

### Oracle Cloud Account
- Free tier account at https://cloud.oracle.com
- No credit card required for Always Free tier (in most regions)

### Local Tools
```bash
pip install oci
oci setup config  # configure tenancy, user, key, region
```

### SSH Key
```bash
ssh-keygen -t rsa -b 4096 -C "udaya-deploy" -f ~/.ssh/udaya_cloud
```

## Deployment Steps

### Step 1: Build Release Binary

```bash
cd /path/to/Udaya
cargo build --release --bin Udayad
```

The binary will be at `target/release/Udayad`.

### Step 2: Provision Oracle Cloud Infrastructure

```bash
python deployments/cloud/provision-oracle.py \
    --compartment-id <YOUR_COMPARTMENT_OCID> \
    --region us-ashburn-1 \
    --ssh-key-path ~/.ssh/udaya_cloud.pub \
    --binary-path ./target/release/Udayad \
    --rpc-password "$(openssl rand -hex 32)"
```

This creates:
- 1 VCN (10.0.0.0/16)
- 1 Internet Gateway
- 1 Route Table
- 1 Public Subnet
- 3 VM.Standard.A1.Flex instances (1 OCPU, 6GB RAM, 50GB boot volume each)

### Step 3: Transfer Binary and Scripts

```bash
NODE_IP="<node-ip-from-step-2>"
scp -i ~/.ssh/udaya_cloud ./target/release/Udayad ubuntu@${NODE_IP}:/tmp/Udayad
scp -i ~/.ssh/udaya_cloud deployments/cloud/* ubuntu@${NODE_IP}:/tmp/
```

### Step 4: Deploy Each Node

```bash
# On each VM (as root):
ssh -i ~/.ssh/udaya_cloud ubuntu@${NODE_IP}
sudo bash /tmp/deploy-node.sh 1 "your-secure-rpc-password"
```

Repeat for nodes 2 and 3 with the appropriate node number.

### Step 5: Verify Deployment

```bash
# Check service status
sudo systemctl status udaya-node@1.service
sudo systemctl status udaya-node@2.service
sudo systemctl status udaya-node@3.service

# Check logs
sudo journalctl -u udaya-node@1.service -f

# Check P2P connectivity
sudo ss -tlnp | grep 1979

# Query RPC (from the VM itself)
curl -u udaya:your-rpc-password http://localhost:18332/ \
    -d '{"jsonrpc":"1.0","id":"1","method":"getblockcount","params":[]}'
```

## Node Specifications

| Node | Role | P2P Port | RPC Port | Mining |
|------|------|----------|----------|--------|
| Node 1 | Mining seed | 19798 | 18332 | Yes |
| Node 2 | Sync/relay | 19799 | 18334 | No |
| Node 3 | Sync/relay | 19800 | 18336 | No |

## Firewall Rules

| Port | Direction | Source | Purpose |
|------|-----------|--------|---------|
| 22 | Inbound | Admin IP only | SSH |
| 19798 | Inbound | 0.0.0.0/0 | Node 1 P2P |
| 19799 | Inbound | 0.0.0.0/0 | Node 2 P2P |
| 19800 | Inbound | 0.0.0.0/0 | Node 3 P2P |

RPC ports (18332, 18334, 18336) are **not** exposed externally. They bind to `127.0.0.1`.

## Security

- SSH: Restrict to known admin IPs after initial setup
- RPC: localhost-only, authenticated
- P2P: Public but protocol-level DoS protection
- No database/storage ports exposed
- No admin interfaces exposed
- Restricted RPC methods (wallet operations blocked)

## Monitoring

```bash
# Run monitor in background
nohup bash deployments/cloud/udaya-monitor.sh 1 30 > /var/log/udaya/monitor-node1.log 2>&1 &
```

## Log Rotation

```bash
sudo cp deployments/cloud/udaya-logrotate /etc/logrotate.d/udaya
sudo logrotate -f /etc/logrotate.d/udaya
```

## Troubleshooting

### Node won't start
```bash
sudo journalctl -u udaya-node@1.service -n 100 --no-pager
sudo systemctl restart udaya-node@1.service
```

### Can't connect to peers
```bash
# Check firewall
sudo ufw status verbose
# Check P2P port
sudo ss -tlnp | grep 1979
# Check preferred peers in config
grep preferred_peers /etc/udaya/testnet-node1.conf
```

### Disk full
```bash
df -h /var/lib/udaya
sudo systemctl stop udaya-node@1.service
# Enable pruning in config and restart
```

## Cleanup

```bash
# Stop all nodes
sudo systemctl stop udaya-node@1 udaya-node@2 udaya-node@3
sudo systemctl disable udaya-node@1 udaya-node@2 udaya-node@3

# Remove data (WARNING: destroys blockchain)
sudo rm -rf /var/lib/udaya/node*
sudo rm -f /etc/udaya/testnet-node*.conf
sudo rm -f /etc/systemd/system/udaya-node@*.service
sudo systemctl daemon-reload
```

## Phase 6 Pass Criteria Checklist

- [ ] 3 cloud nodes running
- [ ] Developer PC can be powered OFF
- [ ] Network remains operational
- [ ] Nodes discover each other
- [ ] Nodes synchronize
- [ ] Same best-block hash confirmed
- [ ] Blocks propagate
- [ ] Transactions propagate
- [ ] Restart recovery passes
- [ ] Disconnect/reconnect recovery passes
- [ ] RPC security works
- [ ] Firewall correctly configured
- [ ] Automatic service restart works
- [ ] Persistent storage works
- [ ] Monitoring works
- [ ] No secrets exposed
