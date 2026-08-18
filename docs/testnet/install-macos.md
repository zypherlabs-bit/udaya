# Udaya Testnet - macOS Installation Guide

## System Requirements

- macOS 11+ (Intel or Apple Silicon)
- 4GB RAM minimum (8GB recommended)
- 50GB free disk space
- Port 19798 open (P2P)

## Quick Install

### Option 1: Binary (Recommended)

```bash
# Download latest release
curl -L https://github.com/udayafoundation/udaya/releases/latest/download/udaya-macos-universal.tar.gz -o udaya.tar.gz
tar -xzf udaya.tar.gz
sudo cp udayad /usr/local/bin/
sudo chmod +x /usr/local/bin/udayad
```

### Option 2: Build from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Install dependencies
brew install openssl pkg-config

# Build
git clone https://github.com/udayafoundation/udaya.git
cd udaya
cargo build --release
sudo cp target/release/udayad /usr/local/bin/
```

## Configuration

```bash
mkdir -p ~/.udaya/data ~/.udaya/logs
cp config/testnet/bitfury-testnet.conf ~/.udaya/udaya.conf
```

Edit `~/.udaya/udaya.conf`:
```toml
[storage]
data_dir = "~/.udaya/data"

[rpc]
username = "udaya"
password = "your_secure_password"
```

## Run

```bash
# Direct
udayad --config ~/.udaya/udaya.conf start

# With mining
udayad --config ~/.udaya/udaya.conf --mine
```

## Run as Service (LaunchAgent)

Create `~/Library/LaunchAgents/org.udaya.node.plist`:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>org.udaya.node</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/udayad</string>
        <string>--config</string>
        <string>/Users/yourname/.udaya/udaya.conf</string>
        <string>start</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/Users/yourname/.udaya/logs/stdout.log</string>
    <key>StandardErrorPath</key>
    <string>/Users/yourname/.udaya/logs/stderr.log</string>
</dict>
</plist>
```

```bash
launchctl load ~/Library/LaunchAgents/org.udaya.node.plist
```

## Firewall

```bash
# macOS Firewall
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /usr/local/bin/udayad
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --unblockapp /usr/local/bin/udayad
```
