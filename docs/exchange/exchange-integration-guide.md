# Udaya (UDYA) Exchange Integration Guide

## Overview
This document provides all technical details required for cryptocurrency exchanges to integrate Udaya (UDYA) for deposits, withdrawals, and trading.

## Chain Metadata

| Parameter | Value |
|-----------|-------|
| **Ticker** | UDYA |
| **Full Name** | Udaya |
| **Decimals** | 8 |
| **Smallest Unit** | 1 satoshi (0.00000001 UDYA) |
| **Consensus Algorithm** | SHA-256d (Double SHA-256) |
| **Type** | Proof-of-Work |
| **Block Time** | 600 seconds (10 minutes) |
| **Difficulty Adjustment** | Every 2,016 blocks (~2 weeks) |
| **Halving Interval** | Every 210,000 blocks (~4 years) |
| **Maximum Supply** | 21,000,000 UDYA |
| **Genesis Block Hash** | [Determined at mainnet launch] |
| **P2P Port** | 9798 (mainnet) |
| **RPC Port** | 8332 |
| **Address Prefixes** | 1 (mainnet P2PKH), 3 (mainnet P2SH) |
| **Signature Hash Type** | SIGHASH_ALL (0x01) |
| **Protocol Version** | 70016 |
| **User Agent** | /Udaya:1.0.0/ |

## Recommended Confirmation Settings

| Transaction Type | Confirmations | Notes |
|-----------------|---------------|-------|
| Small deposits (< 10 UDYA) | 3 | ~30 minutes |
| Standard deposits | 6 | ~60 minutes (recommended) |
| Large deposits (> 1000 UDYA) | 12 | ~2 hours |
| Withdrawals | 1 | After confirmed in wallet |
| Internal transfers | 1 | After confirmed in wallet |

## Reorg Handling
- Maximum reorg depth: **6 blocks**
- Finality checkpoints every **100 blocks**
- Exchange wallets should wait for **6+ confirmations** before crediting deposits
- Chain split detector will automatically detect and alert on forks

## RPC Endpoints

### Starting the Wallet Daemon
```bash
# Start full node with wallet
udayad -c config/mainnet/udaya-mainnet.conf start

# Run as backend service
udayad --datadir /data/Udaya --rpc-port 8332 start
```

### Authentication
Configure RPC credentials via environment variables or configuration:
```
username: ${RPC_USER:-Udaya}
password: ${RPC_PASSWORD}
```

### Available RPC Methods

#### Blockchain
- `getblockchaininfo` - Current blockchain state
- `getblockcount` - Current block height
- `getblock <hash|height>` - Block details
- `getblockhash <height>` - Block hash at height
- `gettransaction <txid>` - Transaction details
- `gettxout <txid> <vout>` - UTXO information

#### Wallet
- `getbalance` - Wallet balance (UDYA)
- `getnewaddress` - Generate new receiving address
- `sendtoaddress <address> <amount>` - Send UDYA
- `listunspent` - List UTXOs
- `listtransactions` - Transaction history

#### Network
- `getpeerinfo` - Connected peers
- `getnetworkinfo` - Network state
- `getconnectioncount` - Number of connections

#### Mining
- `getmininginfo` - Mining statistics
- `getnetworkhashps` - Network hashrate estimate

### Example: Deposits
```bash
# 1. Generate deposit address
curl -X POST http://127.0.0.1:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getnewaddress","params":[]}'

# Response: {"jsonrpc":"2.0","result":"1UDYAxChange67..."}

# 2. Check for new transactions (polling)
curl -X POST http://127.0.0.1:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"listtransactions","params":[]}'

# 3. Get specific transaction
curl -X POST http://127.0.0.1:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"gettransaction","params":["<txid>"]}'

# 4. Verify UTXO
curl -X POST http://127.0.0.1:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"gettxout","params":["<txid>",0]}'
```

### Example: Withdrawals
```bash
# 1. Check balance
curl -X POST http://127.0.0.1:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getbalance","params":[]}'

# 2. Send transaction
curl -X POST http://127.0.0.1:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"sendtoaddress","params":["<address>",1.5]}'

# Response includes txid
```

## Genesis Block Verification

The Udaya genesis block can be independently verified:
```bash
# Mine and verify genesis block
udayad mine-genesis --network mainnet \
  --statement "Udaya Foundation: Launching decentralized commerce" \
  --start-nonce 0 --max-nonce 10000000

# Verify existing genesis
udayad getblock 0
```

## Wallet Setup

### Production Wallet Configuration
```toml
[wallet]
enable = true
wallet_file = "/data/Udaya/wallet.dat"
default_fee_rate = 50

[rpc]
enable = true
listen_addr = "127.0.0.1"
listen_port = 8332
username = "${RPC_USER:-Udaya}"
password = "${RPC_PASSWORD}"
```

### Best Practices
1. Run nodes behind firewalls, only expose RPC to internal network
2. Use strong RPC passwords (32+ characters)
3. Monitor wallet balance via `getbalance` every 60 seconds
4. Process deposits with 6+ confirmations
5. Maintain hot/cold wallet separation
6. Keep at least 3 full nodes synchronized
7. Monitor node health via `/health` endpoint

## Monitoring

### Health Check
```bash
curl http://127.0.0.1:8332/health
# {"status":"ok","service":"udayad","version":"1.0.0","timestamp":"..."}
```

### Metrics
- Prometheus metrics available on RPC port
- Grafana dashboards included in repository
- Alerts for node sync status, peer count, mempool size

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Node not syncing | Check peer connectivity: `getpeerinfo` |
| Wallet not showing balance | Verify wallet file exists and RPC is enabled |
| Transaction stuck in mempool | Increase fee rate in config |
| Connection refused | Verify node is running and port is accessible |
| Address validation failure | Verify network (mainnet vs testnet) matches address prefix |

## Support
For exchange integration support, contact:
- Email: exchange@Udaya.net
- GitHub: https://github.com/Udaya/Udaya
- Documentation: https://docs.Udaya.net