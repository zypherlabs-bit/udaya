# Udaya (UDYA)

<div align="center">

![Udaya Logo](website/images/udaya-logo.png)

**Production-grade, SHA-256d Proof-of-Work Layer-1 Blockchain**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue)](https://www.rust-lang.org)
[![Build Status](https://github.com/UdayaFoundation/Udaya/actions/workflows/ci.yml/badge.svg)](https://github.com/UdayaFoundation/Udaya/actions)
[![Coverage](https://codecov.io/gh/UdayaFoundation/Udaya/branch/main/graph/badge.svg)](https://codecov.io/gh/UdayaFoundation/Udaya)
[![Security](https://img.shields.io/badge/security-audited-green)](SECURITY.md)

</div>

---

## Overview

Udaya is a **production-grade, SHA-256d Proof-of-Work Layer-1 blockchain** implemented in Rust, featuring Bitcoin-inspired architecture with modernized infrastructure. It provides a secure, high-performance foundation for decentralized applications and global commerce.

### Key Features

- **SHA-256d PoW Consensus**: Bitcoin-compatible proof-of-work with difficulty adjustment
- **UTXO Model**: Full UTXO set management for transaction processing
- **SegWit & Taproot**: Modern script capabilities activated at genesis
- **HD Wallet**: BIP32/44/49/84/86 HD wallet with BIP39 mnemonic support
- **Stratum V2 Mining**: Advanced mining pool with ASIC optimization profiles
- **P2P Networking**: Decentralized peer-to-peer network protocol
- **JSON-RPC & REST API**: 22+ RPC methods for comprehensive node interaction
- **Prometheus Metrics**: Built-in observability and monitoring
- **Security Framework**: Comprehensive fuzzing and adversarial simulation
- **On-chain Governance**: Decentralized proposal and voting system
- **Docker & Kubernetes**: Production-ready containerization

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    External Interfaces                        │
│  [CLI] [JSON-RPC] [REST API] [WebSocket] [Explorer]        │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────┐
│                    Node Daemon (main.rs)                      │
│  - Lifecycle Management                                      │
│  - Configuration Loading                                     │
│  - Subsystem Initialization                                  │
└─────────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
┌───────▼────────┐  ┌──────▼──────┐  ┌────────▼────────┐
│  Core Engine   │  │   Wallet    │  │   Mining Pool   │
│  - Consensus   │  │   Engine    │  │   - Stratum V2  │
│  - Validation  │  │   - BIP32   │  │   - ASIC Mgmt   │
│  - Genesis     │  │   - BIP39   │  │   - Monitoring  │
│  - Governance  │  │   - HD Keys │  │                 │
└───────┬────────┘  └─────────────┘  └─────────────────┘
        │
   ┌────┴────┬────────┬────────┬────────┬────────┐
   │         │        │        │        │        │
┌──▼──┐  ┌───▼───┐ ┌──▼──┐ ┌──▼──┐ ┌──▼──┐ ┌──▼──┐
│P2P  │  │Mempool│ │Store│ │ API │ │Sec  │ │Obs  │
│Net  │  │       │ │Rocks│ │     │ │Fuzz │ │Met  │
│     │  │       │ │DB   │ │     │ │Adv  │ │ric  │
└─────┘  └───────┘ └─────┘ └─────┘ └─────┘ └─────┘
```

---

## Quick Start

### Prerequisites

- Rust 1.75+ ([install](https://www.rust-lang.org/tools/install))
- Git
- Linux, macOS, or Windows

### Build from Source

```bash
# Clone the repository
git clone https://github.com/UdayaFoundation/Udaya.git
cd Udaya

# Build the node
cargo build --release

# Run tests
cargo test

# Start the node
./target/release/udayad --config config/bitfury.conf
```

### Configuration

1. **Copy and edit the configuration file**:
   ```bash
   cp config/bitfury.conf config/udaya.conf
   # Edit config/udaya.conf with your settings
   ```

2. **Set RPC credentials via environment variables** (IMPORTANT):
   ```bash
   export RPC_USER=your_secure_username
   export RPC_PASSWORD=your_strong_random_password
   ```

3. **Configure CORS** for your application:
   ```toml
   [rpc]
   cors_domains = ["https://your-app.com"]
   ```

### Docker

```bash
# Build image
docker build -t udaya-node -f deployments/docker/Dockerfile .

# Run container
docker run -d -p 9798:9798 -p 8332:8332 \
  -v /path/to/data:/data/udaya \
  udaya-node:latest
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [Getting Started](docs/operations/getting-started.md) | Installation and setup guide |
| [Architecture](docs/architecture/README.md) | System design and architecture |
| [API Reference](docs/api/) | JSON-RPC and REST API documentation |
| [Developer Guide](docs/developer/) | Contributing and development setup |
| [Mining Guide](docs/mining/) | Mining setup and optimization |
| [Security](SECURITY.md) | Security policy and bug bounty |
| [Contributing](CONTRIBUTING.md) | Contribution guidelines |
| [FAQ](docs/operations/faq.md) | Frequently asked questions |

---

## API Endpoints

### JSON-RPC (port 8332)

**Blockchain Methods**:
- `getblockchaininfo` - Get blockchain information
- `getblockcount` - Get current block count
- `getblockhash` - Get block hash by height
- `getblock` - Get block details
- `gettransaction` - Get transaction details
- `gettxout` - Get UTXO information

**Mempool Methods**:
- `getmempoolinfo` - Get mempool statistics
- `getmempoolentry` - Get mempool entry by txid

**Wallet Methods**:
- `getbalance` - Get wallet balance
- `getnewaddress` - Generate new address
- `sendtoaddress` - Send transaction
- `listunspent` - List unspent outputs
- `listtransactions` - List transactions

**Mining Methods**:
- `getmininginfo` - Get mining information
- `getblocktemplate` - Get block template for mining
- `submitblock` - Submit mined block

**Network Methods**:
- `getpeerinfo` - Get peer information
- `getnetworkinfo` - Get network information
- `addnode` - Add peer connection

**Utility Methods**:
- `ping` - Ping server
- `getinfo` - Get general node information
- `stop` - Stop the node

### REST API

- `GET /health` - Health check with full report
- `GET /healthz` - Simple liveness check
- `GET /readyz` - Readiness check
- `GET /metrics` - Prometheus metrics
- `POST /` - JSON-RPC endpoint

---

## Development

### Running Tests

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# All tests
cargo test --all-features

# Fuzzing tests (requires nightly)
cargo +nightly fuzz run fuzz_target

# Performance benchmarks
cargo bench
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Audit dependencies
cargo audit

# Check licenses
cargo deny check
```

---

## Security

For security vulnerabilities, please follow our [Security Policy](SECURITY.md).

**Do NOT** open public issues for security vulnerabilities.

### Bug Bounty

We offer bug bounties for qualifying vulnerabilities:
- **Critical**: $5,000 - $50,000
- **High**: $1,000 - $10,000
- **Medium**: $200 - $2,000
- **Low**: $50 - $500

---

## Community

- **Discussions**: [GitHub Discussions](https://github.com/UdayaFoundation/Udaya/discussions)
- **Discord**: [Join our Discord](https://discord.gg/udaya)
- **Twitter**: [@UdayaFoundation](https://twitter.com/UdayaFoundation)
- **Blog**: [udaya.org/blog](https://udaya.org/blog)

---

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Install development tools
cargo install cargo-clippy cargo-fmt cargo-audit

# Run pre-commit hooks (if using pre-commit)
pre-commit install
```

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Roadmap

### Phase 1: Community Alpha (Q4 2026)
- [x] Core blockchain implementation
- [x] Basic wallet functionality
- [x] Mining support
- [ ] Desktop wallet GUI (Tauri)
- [ ] Mobile wallets (iOS/Android)
- [ ] Browser extension

### Phase 2: Community Beta (Q1 2027)
- [ ] Lightning Network mainnet
- [ ] Smart contract platform
- [ ] Cross-chain bridges
- [ ] Advanced privacy features

### Phase 3: Stable Release (Q2 2027)
- [ ] Full mainnet launch
- [ ] Enterprise features
- [ ] Advanced governance
- [ ] DeFi ecosystem

---

## Acknowledgments

Udaya builds upon the work of:
- Bitcoin Core developers
- Rust cryptocurrency ecosystem
- Open-source cryptographic libraries
- Community contributors

---

## Contact

- **Website**: [udaya.org](https://udaya.org)
- **Email**: info@udaya.org
- **Security**: security@udaya.org
- **Foundation**: Udaya Foundation

---

<div align="center">

**Built with ❤️ by the Udaya Foundation and community**

[Website](https://udaya.org) • [Documentation](https://docs.udaya.org) • [Discord](https://discord.gg/udaya) • [Twitter](https://twitter.com/UdayaFoundation)

</div>