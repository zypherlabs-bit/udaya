# Udaya Exchange Readiness Report

> **Date:** June 10, 2026
> **Status:** Ready for Exchange Integration ✅
> **Version:** 1.0.0

## 1. Executive Summary

Udaya (UDYA) is ready for exchange integration. The following report documents all technical aspects required for a cryptocurrency exchange to list and support UDYA deposits, withdrawals, and trading.

## 2. Chain Specifications

| Parameter | Value |
|-----------|-------|
| **Ticker** | UDYA |
| **Full Name** | Udaya |
| **Decimals** | 8 (1 UDYA = 100,000,000 satoshis) |
| **Consensus** | Proof-of-Work (SHA-256d) |
| **Block Time** | 600 seconds (10 minutes) |
| **Difficulty Adjustment** | Every 2,016 blocks (~2 weeks) |
| **Halving Interval** | Every 210,000 blocks (~4 years) |
| **Max Supply** | 21,000,000 UDYA |
| **P2P Port** | 9798 (mainnet) |
| **RPC Port** | 8332 |
| **Network Magic** | BF591AE7 (0xBF, 0x59, 0x1A, 0xE7) |
| **Protocol Version** | 70016 |

## 3. Address Formats

| Type | Standard | Example Prefix | Encoding |
|------|----------|---------------|----------|
| Legacy (P2PKH) | BIP-44 | `1` | Base58Check |
| P2SH-SegWit | BIP-49 | `3` | Base58Check |
| Native SegWit | BIP-84 | `btf1` | bech32 (BIP-173) |
| Taproot | BIP-86 | `btf1` | bech32m (BIP-350) |

## 4. Recommended Confirmation Settings

| Transaction Type | Confirmations | Time Estimate |
|-----------------|---------------|---------------|
| Small deposits (< 10 UDYA) | 3 | ~30 minutes |
| Standard deposits | 6 | ~60 minutes |
| Large deposits (> 1000 UDYA) | 12 | ~2 hours |
| Withdrawals | 1 | After broadcast |
| Internal transfers | 1 | After broadcast |

## 5. Reorg Handling

- **Maximum reorg depth:** 6 blocks
- **Finality checkpoints:** Every 100 blocks
- **Chain split detection:** Built into node
- **Recommended action for reorgs:** Wait for 6 confirmations before crediting deposits

## 6. RPC Methods

### Blockchain
| Method | Parameters | Description |
|--------|------------|-------------|
| `getblockchaininfo` | none | Current blockchain state |
| `getblockcount` | none | Current block height |
| `getblock` | hash/height | Block details |
| `getblockhash` | height | Block hash at height |
| `gettransaction` | txid | Transaction details |
| `gettxout` | txid, vout | UTXO information |

### Wallet
| Method | Parameters | Description |
|--------|------------|-------------|
| `getbalance` | none | Wallet balance |
| `getnewaddress` | none | Generate new receiving address |
| `sendtoaddress` | address, amount | Send UDYA |
| `listunspent` | none | List UTXOs |
| `listtransactions` | none | Transaction history |

### Network
| Method | Parameters | Description |
|--------|------------|-------------|
| `getpeerinfo` | none | Connected peers |
| `getnetworkinfo` | none | Network state |
| `getconnectioncount` | none | Number of connections |

### Mining
| Method | Parameters | Description |
|--------|------------|-------------|
| `getmininginfo` | none | Mining statistics |
| `getnetworkhashps` | none | Network hashrate |

## 7. Deposit Flow

1. Generate deposit address: `getnewaddress`
2. Monitor for incoming transactions: `listtransactions` 
3. Verify transaction: `gettransaction <txid>`
4. Wait for confirmations (6 recommended)
5. Credit user's account
6. Monitor for chain reorganization (up to 6 blocks)

## 8. Withdrawal Flow

1. Verify balance: `getbalance`
2. Build transaction with fee
3. Broadcast: `sendtoaddress <address> <amount>`
4. Monitor for broadcast confirmation
5. Mark withdrawal as complete after 1 confirmation

## 9. Node Requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU | 4 cores | 8+ cores |
| RAM | 8 GB | 16+ GB |
| Storage | 50 GB (SSD) | 200 GB+ (SSD) |
| Bandwidth | 100 Mbps | 1 Gbps |
| OS | Ubuntu 22.04 | Ubuntu 22.04 / 24.04 |

## 10. Docker Deployment

```bash
# Pull and run
docker pull Udaya/node:latest
docker run -d --name Udaya-exchange \
  -p 9798:9798 \
  -p 8332:8332 \
  -v Udaya-data:/data/Udaya \
  -e udaya_NETWORK=mainnet \
  Udaya/node:latest
```

## 11. Monitoring

### Health Check
```
GET /health
Response: {"status":"ok","service":"udayad","version":"1.0.0"}
```

### Prometheus Metrics
Available on RPC port at `/metrics`:
- `udaya_block_height`
- `udaya_peer_count`  
- `udaya_mempool_size`
- `udaya_hashrate`
- `udaya_uptime_seconds`

## 12. Security Recommendations

1. Run node on dedicated hardware with firewall
2. Restrict RPC to localhost or internal network
3. Enable TLS for remote RPC connections
4. Use strong RPC passwords (32+ characters)
5. Monitor node health with Prometheus/Grafana
6. Set up automated wallet backups
7. Maintain hot/cold wallet separation
8. Keep at least 3 full nodes synchronized
9. Configure alerting for node downtime

## 13. Support Contacts

- **Documentation:** https://docs.Udaya.net
- **Explorer:** https://explorer.Udaya.net  
- **GitHub:** https://github.com/Udaya/Udaya
- **Email:** exchange@Udaya.net