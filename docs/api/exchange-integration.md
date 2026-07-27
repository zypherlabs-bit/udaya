# Udaya Exchange Integration Guide
## UDYA - SHA-256d Proof-of-Work Cryptocurrency

### Chain Information

| Property | Value |
|----------|-------|
| Ticker | **UDYA** |
| Algorithm | **SHA-256d** (Double SHA-256) |
| Type | **Proof-of-Work** |
| Decimals | **8** |
| Genesis Hash | [TBD at mainnet launch] |
| Block Time | **600 seconds** (10 minutes) |
| Difficulty Adjustment | Every **2016 blocks** (~2 weeks) |
| Halving | Every **210,000 blocks** (~4 years) |
| Max Supply | **21,000,000 UDYA** |
| Initial Reward | **50 UDYA** |
| Current Reward | **50 UDYA** (at mainnet launch) |
| Maturity | **100 confirmations** for coinbase |
| P2P Port | **9798** |
| RPC Port | **8332** |

### Wallet Setup for Exchanges

#### 1. Install Udaya Node

```bash
# Download latest release
wget https://github.com/Udaya/Udaya/releases/latest/download/udayad-linux-amd64.tar.gz
tar -xzf udayad-linux-amd64.tar.gz
sudo mv udayad /usr/local/bin/

# Create configuration
mkdir -p /etc/Udaya
cat > /etc/Udaya/udaya.conf << 'EOF'
[network]
listen_port = 9798
max_peers = 125

[storage]
data_dir = "/data/Udaya/exchange"
db_cache_size_mb = 4096

[consensus]
network = "mainnet"
min_tx_fee = 1000

[rpc]
enable = true
listen_addr = "127.0.0.1"
listen_port = 8332
username = "${RPC_USER:-Udaya}"
password = "${RPC_PASSWORD}"
enable_ws = false

[wallet]
enable = true
wallet_file = "exchange-wallet.dat"
default_fee_rate = 50

[logging]
level = "info"
enable_json = true
EOF
```

#### 2. Start the Node

```bash
# Initialize and sync
udayad -c /etc/Udaya/udaya.conf start

# Wait for full sync
# Check status:
udayad -c /etc/Udaya/udaya.conf getinfo
```

#### 3. Create Deposit Addresses

```bash
# Generate new receiving address
curl -X POST http://127.0.0.1:8332 \
  -H "Content-Type: application/json" \
  -u "exchange:YOUR_STRONG_RPC_PASSWORD" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getnewaddress","params":[]}'

# Response: {"result": "UDYA1q..."}
```

### RPC API Reference

#### Blockchain

```bash
# Get blockchain info
curl -X POST http://127.0.0.1:8332 \
  -u "$RPC_USER:$RPC_PASSWORD" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getblockchaininfo","params":[]}'

# Response:
{
  "chain": "mainnet",
  "blocks": 123456,
  "bestblockhash": "0000000000...",
  "difficulty": 12345678.9,
  "mediantime": 1234567890
}
```

#### Wallet Operations

```bash
# Check balance
curl -X POST http://127.0.0.1:8332 \
  -u "$RPC_USER:$RPC_PASSWORD" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getbalance","params":[]}'

# List unspent outputs
curl -X POST http://127.0.0.1:8332 \
  -u "exchange:YOUR_STRONG_RPC_PASSWORD" \
  -d '{"jsonrpc":"2.0","id":1,"method":"listunspent","params":[]}'

# Send transaction
curl -X POST http://127.0.0.1:8332 \
  -u "exchange:YOUR_STRONG_RPC_PASSWORD" \
  -d '{"jsonrpc":"2.0","id":1,"method":"sendtoaddress","params":["UDYA1q...", 1.5]}'

# List transactions
curl -X POST http://127.0.0.1:8332 \
  -u "exchange:YOUR_STRONG_RPC_PASSWORD" \
  -d '{"jsonrpc":"2.0","id":1,"method":"listtransactions","params":["*", 10, 0]}'
```

### Transaction Monitoring

#### Confirmations
For deposit safety, exchanges should require:

- **Minimum confirmations:** **6** (standard)
- **High-value deposits:** **12+** confirmations
- **Large withdrawals:** Manual review + 6 confirmations

#### Reorg Handling
- Maximum reorg depth: **6 blocks**
- If a reorg exceeds 6 blocks, alert operations team
- Checkpoint finality at **100 blocks**

#### Webhook Notification Example
```javascript
// Webhook event for new deposit
{
  "event": "new_transaction",
  "txid": "abc123...",
  "address": "UDYA1q...",
  "amount": 1.5,
  "confirmations": 0,
  "timestamp": 1234567890
}

// When confirmed
{
  "event": "confirmed_transaction",
  "txid": "abc123...",
  "address": "UDYA1q...",
  "amount": 1.5,
  "confirmations": 6,
  "timestamp": 1234567890
}
```

### Address Formats

Udaya supports multiple address formats for backwards compatibility:

| Type | Prefix | Example |
|------|--------|---------|
| Legacy (P2PKH) | 1 | `1UDYA...` |
| Nested SegWit (P2SH) | 3 | `3UDYA...` |
| Native SegWit (bech32) | UDYA1 | `UDYA1q...` |
| Taproot (bech32m) | UDYA1 | `UDYA1p...` |

**Recommendation:** Use **Native SegWit (UDYA1)** addresses for new integrations.

### Fee Estimation

```bash
# Get recommended fee rate (sats/byte)
curl -X POST http://127.0.0.1:8332 \
  -u "exchange:YOUR_STRONG_RPC_PASSWORD" \
  -d '{"jsonrpc":"2.0","id":1,"method":"estimatesmartfee","params":[6]}'
```

| Priority | Target Blocks | Fee Rate (sats/byte) |
|----------|---------------|---------------------|
| High | 2-3 | 100-200 |
| Medium | 4-6 | 50-100 |
| Low | 6+ | 10-50 |

### Withdrawal Process

1. **Validate withdrawal address** (checksum verification)
2. **Check internal balance** sufficient
3. **Confirm with 2FA/approval** for large amounts
4. **Build transaction** with appropriate fee
5. **Broadcast** to network
6. **Monitor** for first confirmation

```bash
# Example withdrawal
curl -X POST http://127.0.0.1:8332 \
  -u "exchange:YOUR_STRONG_RPC_PASSWORD" \
  -d '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"sendtoaddress",
    "params":["UDYA1q...", 10.5, "Exchange Withdrawal", "user123"]
  }'
```

### Hot/Wallet Architecture

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│   Deposit    │────▶│  Monitoring  │────▶│  Cold Wallet │
│   Watcher    │     │  Service     │     │  (Offline)   │
└─────────────┘     └──────────────┘     └──────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │   Hot Wallet │
                    │  (Online)    │
                    └──────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ Withdrawal   │
                    │ Engine       │
                    └──────────────┘
```

### Security Best Practices

1. **Run node as non-root user** (Udaya user)
2. **Firewall**: Only allow RPC from internal IPs
3. **TLS**: Use reverse proxy for external RPC access
4. **Rate limiting**: Max 100 RPC requests/second
5. **Backup**: Daily encrypted wallet backups
6. **Monitoring**: Alert on large transactions (100+ UDYA)
7. **Audit**: Regular balance reconciliation

### Troubleshooting

**Node won't sync:**
```bash
# Check disk space
df -h /data/Udaya

# Check network connectivity
curl -I http://seed.Udaya.net:9798

# Reset and re-sync
rm -rf /data/Udaya/mainnet/blockchain
udayad -c /etc/Udaya/udaya.conf start
```

**Transaction not confirming:**
```bash
# Check fee rate
udayad gettransaction <txid>

# Increase fee with CPFP or RBF if supported
# Or wait for mempool to clear
```

### Support

- **Developer Chat:** https://discord.gg/Udaya
- **Technical Docs:** https://docs.Udaya.net
- **GitHub Issues:** https://github.com/Udaya/Udaya/issues
- **Email:** exchange-support@Udaya.net