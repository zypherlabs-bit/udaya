# Udaya Getting Started Guide

## Welcome to Udaya!

This guide will help you get started with Udaya, from installation to running your first node and making transactions.

---

## Table of Contents

1. [System Requirements](#system-requirements)
2. [Installation](#installation)
   - [Windows](#windows)
   - [Linux](#linux)
   - [macOS](#macos)
3. [Building from Source](#building-from-source)
4. [Configuration](#configuration)
5. [Running Your Node](#running-your-node)
6. [Basic Commands](#basic-commands)
7. [Troubleshooting](#troubleshooting)
8. [Next Steps](#next-steps)

---

## System Requirements

### Minimum Requirements

- **CPU**: 2 cores
- **RAM**: 4GB
- **Storage**: 50GB SSD (for full node)
- **OS**: Windows 10+, Linux (Ubuntu 22.04+, Debian 12+, Fedora 38+), macOS 13+
- **Network**: Stable internet connection

### Recommended Requirements

- **CPU**: 4+ cores
- **RAM**: 8GB+
- **Storage**: 100GB+ SSD
- **OS**: Latest stable version of your preferred OS
- **Network**: High-speed connection with low latency

---

## Installation

### Windows

#### Prerequisites

1. Install [Rust](https://www.rust-lang.org/tools/install):
   ```powershell
   winget install Rustlang.Rustup
   ```

2. Install build tools:
   - Visual Studio 2022 with "Desktop development with C++" workload
   - Or install via command line:
     ```powershell
     choco install visualstudio2022buildtools -y
     choco install visualstudio2022-workload-vctools -y
     ```

#### Installation Steps

```powershell
# Clone the repository
git clone https://github.com/UdayaFoundation/Udaya.git
cd Udaya

# Build Udaya
cargo build --release

# Verify installation
.\target\release\udayad --version
```

### Linux

#### Prerequisites (Ubuntu/Debian)

```bash
# Update system
sudo apt-get update && sudo apt-get upgrade -y

# Install dependencies
sudo apt-get install -y build-essential pkg-config libssl-dev git

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

#### Prerequisites (Fedora/RHEL)

```bash
# Update system
sudo dnf update -y

# Install dependencies
sudo dnf install -y gcc-c++ make pkg-config openssl-devel git

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

#### Installation Steps

```bash
# Clone the repository
git clone https://github.com/UdayaFoundation/Udaya.git
cd Udaya

# Build Udaya
cargo build --release

# Verify installation
./target/release/udayad --version
```

### macOS

#### Prerequisites

```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install pkg-config openssl

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

#### Installation Steps

```bash
# Clone the repository
git clone https://github.com/UdayaFoundation/Udaya.git
cd Udaya

# Build Udaya
cargo build --release

# Verify installation
./target/release/udayad --version
```

---

## Building from Source

### Standard Build

```bash
# Clone repository
git clone https://github.com/UdayaFoundation/Udaya.git
cd Udaya

# Build in release mode (optimized)
cargo build --release

# Run tests (optional but recommended)
cargo test --release
```

### Build Options

| Command | Description |
|---------|-------------|
| `cargo build` | Debug build (slower, with debug symbols) |
| `cargo build --release` | Optimized release build |
| `cargo build --release -j4` | Parallel build (4 threads) |
| `cargo build --all-features` | Build with all features enabled |

### Expected Build Artifacts

After successful build, you should have these binaries in `target/release/`:

- `udayad` - Main daemon
- `udaya-cli` - Command-line wallet
- `udaya-faucet` - Testnet faucet service
- `udaya-explorer` - Block explorer backend
- `udaya-pool-server` - Mining pool server

---

## Configuration

### Configuration Files

Udaya uses TOML configuration files located in the `config/` directory:

- `config/bitfury.conf` - Main configuration template
- `config/testnet/` - Testnet configurations
- `config/mainnet/` - Mainnet configurations (future)

### Basic Configuration

1. **Copy the template configuration**:
   ```bash
   cp config/bitfury.conf config/udaya.conf
   ```

2. **Edit the configuration** (example `config/udaya.conf`):
   ```toml
   [node]
   network = "testnet"
   rpc_port = 8332
   p2p_port = 9798

   [database]
   data_dir = "~/.udaya/data"
   cache_size = 1024

   [logging]
   level = "info"
   file = "~/.udaya/debug.log"

   [rpc]
   enabled = true
   cors_domains = ["http://localhost:*"]
   ```

### Environment Variables

Set these environment variables for RPC access:

```bash
# Linux/macOS
export RPC_USER=your_secure_username
export RPC_PASSWORD=your_strong_random_password

# Windows (PowerShell)
$env:RPC_USER="your_secure_username"
$env:RPC_PASSWORD="your_strong_random_password"
```

---

## Running Your Node

### Starting the Node

```bash
# Start with your configuration
./target/release/udayad --config config/udaya.conf

# Or with command-line options
./target/release/udayad --network testnet --rpc-port 8332
```

### Common Command-Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `--config <file>` | Configuration file path | `config/bitfury.conf` |
| `--network <net>` | Network (mainnet/testnet) | `testnet` |
| `--rpc-port <port>` | RPC port | `8332` |
| `--p2p-port <port>` | P2P port | `9798` |
| `--data-dir <dir>` | Data directory | `~/.udaya` |
| `--log-level <level>` | Log level | `info` |

### Running in Background

```bash
# Linux/macOS
nohup ./target/release/udayad --config config/udaya.conf > node.log 2>&1 &

# Windows
start /B .\target\release\udayad --config config\udaya.conf
```

---

## Basic Commands

### Node Management

```bash
# Check node status
curl http://localhost:8332/healthz

# Get node info
curl -X POST http://localhost:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"getinfo","params":[],"id":1}'

# Stop node gracefully
curl -X POST http://localhost:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"stop","params":[],"id":1}'
```

### Wallet Operations

```bash
# Generate new address
./target/release/udaya-cli getnewaddress

# Check balance
./target/release/udaya-cli getbalance

# List transactions
./target/release/udaya-cli listtransactions

# Send transaction
./target/release/udaya-cli sendtoaddress <address> <amount>
```

### Network Operations

```bash
# Get peer info
curl -X POST http://localhost:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"getpeerinfo","params":[],"id":1}'

# Get network info
curl -X POST http://localhost:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"getnetworkinfo","params":[],"id":1}'
```

---

## Troubleshooting

### Common Issues and Solutions

#### Build Failures

| Issue | Solution |
|-------|----------|
| `rustc not found` | Install Rust toolchain properly |
| `linker not found` | Install build-essential (Linux) or Visual Studio (Windows) |
| `OpenSSL not found` | Install libssl-dev (Linux) or OpenSSL (macOS/Windows) |
| `Out of memory` | Close other applications or add swap space |

#### Runtime Issues

| Issue | Solution |
|-------|----------|
| `Port already in use` | Change port in config or kill existing process |
| `Permission denied` | Check file permissions and data directory access |
| `Configuration error` | Validate your config file syntax |
| `Connection refused` | Check if node is running and firewall settings |

#### Performance Issues

| Issue | Solution |
|-------|----------|
| Slow sync | Increase database cache size in config |
| High CPU usage | Limit connections in config |
| High memory usage | Reduce cache size or use pruned mode |

### Debugging Commands

```bash
# Enable debug logging
./target/release/udayad --config config/udaya.conf --log-level debug

# View logs
tail -f ~/.udaya/debug.log

# Check resource usage
top -p $(pgrep udayad)

# Network diagnostics
netstat -tulnp | grep udayad
```

---

## Next Steps

### After Successful Installation

1. **Join the community**:
   - [GitHub Discussions](https://github.com/UdayaFoundation/Udaya/discussions)
   - [Discord Server](https://discord.gg/udaya)

2. **Explore advanced features**:
   - Set up a mining node
   - Configure a full archive node
   - Explore the JSON-RPC API

3. **Contribute to the project**:
   - Report bugs and issues
   - Submit pull requests
   - Help with documentation

4. **Stay updated**:
   - Watch the repository for updates
   - Follow [@UdayaFoundation](https://twitter.com/UdayaFoundation)
   - Read the [Udaya Blog](https://udaya.org/blog)

---

## Additional Resources

- [API Documentation](docs/api/README.md)
- [Mining Guide](docs/mining/README.md)
- [Configuration Reference](docs/operations/node-deployment.md)
- [Troubleshooting Guide](docs/operations/incident-response.md)
- [Security Best Practices](SECURITY.md)

---

## Support

If you encounter issues not covered in this guide:

1. **Check existing issues**: [GitHub Issues](https://github.com/UdayaFoundation/Udaya/issues)
2. **Search discussions**: [GitHub Discussions](https://github.com/UdayaFoundation/Udaya/discussions)
3. **Ask for help**: Create a new discussion with details about your issue
4. **Report bugs**: Open a new issue with reproduction steps

For security-related issues, please follow our [Security Policy](SECURITY.md).

---

**Last Updated**: 2026-07-27
**Udaya Version**: 1.0.0-alpha.1
**Documentation Status**: Phase 3 Public Alpha Testing