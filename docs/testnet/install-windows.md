# Udaya Testnet - Windows Installation Guide

## System Requirements

- Windows 10/11 64-bit
- 4GB RAM minimum (8GB recommended)
- 50GB free disk space
- Port 19798 open (P2P)
- Port 18332 open (RPC, localhost only recommended)

## Quick Install

### Option 1: Binary (Recommended)

1. Download the latest release from [GitHub Releases](https://github.com/udayafoundation/udaya/releases)
2. Extract `udaya-windows-x86_64.zip`
3. Copy `Udayad.exe` to `C:\Program Files\Udaya\`

### Option 2: Build from Source

```powershell
# Install Rust
winget install Rustlang.Rust.MSVC

# Clone and build
git clone https://github.com/udayafoundation/udaya.git
cd udaya
cargo build --release
```

## Configuration

```powershell
# Create directories
New-Item -ItemType Directory -Force -Path "C:\Program Files\Udaya\data" | Out-Null
New-Item -ItemType Directory -Force -Path "C:\Program Files\Udaya\logs" | Out-Null

# Copy configuration
Copy-Item config\testnet\bitfury-testnet.conf "C:\Program Files\Udaya\udaya.conf"
```

Edit `C:\Program Files\Udaya\udaya.conf`:
```toml
[storage]
data_dir = "C:/Program Files/Udaya/data"

[rpc]
username = "udaya"
password = "your_secure_password"
```

## Run as Service (Windows)

Create `C:\Program Files\Udaya\udaya-service.ps1`:
```powershell
$udayaPath = "C:\Program Files\Udaya\Udayad.exe"
$configPath = "C:\Program Files\Udaya\udaya.conf"

while ($true) {
    & $udayaPath --config $configPath start
    Start-Sleep -Seconds 30
}
```

Create a scheduled task or use NSSM:
```powershell
# Using NSSM (Non-Sucking Service Manager)
nssm install Udaya "C:\Program Files\Udaya\Udayad.exe" "--config C:\Program Files\Udaya\udaya.conf start"
nssm start Udaya
```

## Verify

```powershell
# Check process
Get-Process Udayad

# Query RPC
Invoke-RestMethod -Uri "http://127.0.0.1:18332/" `
  -Method Post `
  -Body '{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}' `
  -ContentType "application/json"
```

## Firewall

```powershell
# Allow P2P port
New-NetFirewallRule -DisplayName "Udaya P2P" -Direction Inbound -Protocol TCP -LocalPort 19798 -Action Allow

# Allow RPC port (localhost only)
New-NetFirewallRule -DisplayName "Udaya RPC" -Direction Inbound -Protocol TCP -LocalPort 18332 -Action Allow -RemoteAddress LocalAddress
```
