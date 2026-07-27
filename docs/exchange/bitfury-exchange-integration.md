# Udaya Exchange Integration Guide

## Overview
This document provides everything needed to integrate UDYA on a cryptocurrency exchange.

## Chain Metadata

| Field | Value |
|-------|-------|
| Ticker | UDYA |
| Name | Udaya |
| Algorithm | SHA-256d (Double SHA-256) |
| Type | Proof-of-Work |
| Block Time | 600 seconds (10 minutes) |
| Decimals | 8 |
| Max Supply | 21,000,000 UDYA |
| Genesis Hash | [Determined at mainnet launch] |
| Network Magic | BF591AE7 |
| P2P Port | 9798 |
| RPC Port | 8332 |

## Address Formats

### Mainnet
- **Legacy (P2PKH):** Starts with '1' (Base58)
- **P2SH:** Starts with '3' (Base58)
- **Bech32 (SegWit):** Starts with 'UDYA1'
- **Taproot:** Starts with 'UDYA1' (Bech32m)

### Testnet
- **Legacy:** Starts with 'm' or 'n'
- **P2SH:** Starts with '2'
- **Bech32:** Starts with 'tUDYA1'

## Running a Node for Exchange Operations

### Quick Start with Docker
```bash
# Pull and run mainnet node
docker pull Udaya/node:latest
docker run -d --name Udaya-exchange \
  -p 9798:9798 \
  -p 8332:8332 \
  -v Udaya-data:/data/Udaya \
  -e udaya_NETWORK=mainnet \
  Udaya/node:latest
```

### Manual Start
```bash
# Build from source
cargo build --release

# Run with exchange configuration
./target/release/udayad -c config/mainnet/udaya-mainnet.conf start
```

### Configuration for Exchange
```toml
# config/mainnet/udaya-mainnet.conf
[storage]
data_dir = "/data/Udaya"
prune_blocks = false

[rpc]
listen_addr = "0.0.0.0"
listen_port = 8332
username = "${RPC_USER:-Udaya}"
password = "${RPC_PASSWORD}"
enable_ws = true
ws_port = 8333

[network]
listen_port = 9798
max_peers = 125
```

## JSON-RPC API Methods

### Blockchain
```json
// Get blockchain info
{"jsonrpc":"2.0","id":1,"method":"getblockchaininfo","params":[]}

// Get block by hash/height
{"jsonrpc":"2.0","id":1,"method":"getblock","params":["0000000000..."]}

// Get transaction
{"jsonrpc":"2.0","id":1,"method":"gettransaction","params":["txid123..."]}

// Get UTXO info
{"jsonrpc":"2.0","id":1,"method":"gettxout","params":["txid123...", 0]}
```

### Wallet (if using built-in wallet)
```json
// Get balance
{"jsonrpc":"2.0","id":1,"method":"getbalance","params":[]}

// Generate new address
{"jsonrpc":"2.0","id":1,"method":"getnewaddress","params":[]}

// Send transaction
{"jsonrpc":"2.0","id":1,"method":"sendtoaddress","params":["address", 1.5]}

// List transactions
{"jsonrpc":"2.0","id":1,"method":"listtransactions","params":[]}
```

### Network
```json
// Get peer info
{"jsonrpc":"2.0","id":1,"method":"getpeerinfo","params":[]}

// Get network info
{"jsonrpc":"2.0","id":1,"method":"getnetworkinfo","params":[]}

// Get connection count
{"jsonrpc":"2.0","id":1,"method":"getconnectioncount","params":[]}
```

### Mining
```json
// Get mining info
{"jsonrpc":"2.0","id":1,"method":"getmininginfo","params":[]}

// Get network hashrate
{"jsonrpc":"2.0","id":1,"method":"getnetworkhashps","params":[]}
```

## Reorg Handling

| Parameter | Value |
|-----------|-------|
| Max Reorg Depth | 6 blocks |
| Recommended Confirmations | 6+ for deposits |
| Finality Checkpoints | Every 100 blocks |

### Deposit Flow
1. Monitor mempool for incoming transactions to exchange addresses
2. Wait for 1 confirmation (block inclusion)
3. Credit user after 6 confirmations (1 hour)
4. For large deposits, consider 100 confirmations (finality)

### Withdrawal Flow
1. Build transaction with appropriate fee
2. Broadcast via `sendtoaddress` RPC
3. Monitor for inclusion in next block
4. Mark as complete after 1 confirmation

## Health Check Endpoint
```
GET /health
Response: {"status":"ok","service":"udayad","version":"1.0.0"}
```

## WebSocket Events
Connect to `wss://ws.Udaya.net` for real-time updates:
- `NewBlock` - When a new block is mined
- `NewTransaction` - When a new transaction enters mempool
- `MempoolUpdate` - Mempool size changes
- `ChainReorg` - Chain reorganization detected

## Verifying Genesis Block

```bash
# Start node with genesis verification
udayad start --verify-genesis

# Check blockchain info
udayad getinfo

# Verify genesis hash matches published manifest
# Published manifest available at:
curl https://raw.githubusercontent.com/Udaya/Udaya/main/docs/mainnet-manifest.json
```

## Security Recommendations

1. Run node on dedicated hardware
2. Use firewall rules to restrict RPC access
3. Enable TLS for remote RPC connections
4. Use strong passwords (generate with `openssl rand -base64 32`)
5. Monitor node health with Prometheus/Grafana
6. Set up automated backups of wallet.dat
7. Configure alerting for node downtime
8. Keep node software updated

## Support
- Documentation: https://docs.Udaya.net
- Explorer: https://explorer.Udaya.net
- GitHub: https://github.com/Udaya/Udaya
- Discord: https://discord.gg/Udaya