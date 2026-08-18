# Udaya Testnet - Linux Installation Guide

## System Requirements

- Ubuntu 20.04+ / Debian 11+ / RHEL 8+
- 4GB RAM minimum (8GB recommended)
- 50GB free disk space
- Port 19798 open (P2P)
- Port 18332 open (RPC, localhost only recommended)

## Quick Install

### Option 1: Binary (Recommended)

```bash
# Download latest release
wget https://github.com/udayafoundation/udaya/releases/latest/download/udaya-linux-x86_64.tar.gz
tar -xzf udaya-linux-x86_64.tar.gz
sudo cp udayad /usr/local/bin/
sudo chmod +x /usr/local/bin/udayad
```

### Option 2: Build from Source

```bash
# Install dependencies
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Build
git clone https://github.com/udayafoundation/udaya.git
cd udaya
cargo build --release
sudo cp target/release/udayad /usr/local/bin/
```

## Configuration

```bash
sudo mkdir -p /etc/udaya /var/lib/udaya /var/log/udaya
sudo useradd -r -s /bin/false -d /var/lib/udaya udaya

# Copy configuration
sudo cp config/testnet/bitfury-testnet.conf /etc/udaya/testnet.conf

# Edit configuration
sudo nano /etc/udaya/testnet.conf
```

Set RPC credentials:
```bash
echo "RPC_USER=your_username" | sudo tee /etc/udaya/.env
echo "RPC_PASSWORD=your_secure_password" | sudo tee -a /etc/udaya/.env
```

## Systemd Service

```bash
sudo cp deployments/scripts/udaya-node1.service /etc/systemd/system/udaya-testnet.service
sudo systemctl daemon-reload
sudo systemctl enable --now udaya-testnet
```

## Verify

```bash
# Check status
sudo systemctl status udaya-testnet

# Check logs
sudo journalctl -u udaya-testnet -f

# Query RPC
curl -u your_username:your_secure_password \
  -X POST http://127.0.0.1:18332/ \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}'
```

## Firewall

```bash
# UFW
sudo ufw allow 19798/tcp
sudo ufw allow 18332/tcp comment 'Udaya RPC'

# Firewalld
sudo firewall-cmd --add-port=19798/tcp --permanent
sudo firewall-cmd --add-port=18332/tcp --permanent
sudo firewall-cmd --reload
```
