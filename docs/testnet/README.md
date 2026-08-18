# Udaya Public Testnet

Welcome to the Udaya Public Testnet. This is a community test environment where you can experiment with Udaya blockchain features, test transactions, run nodes, and mine blocks.

## Network Parameters

| Parameter | Value |
|-----------|-------|
| Network Name | `testnet` |
| P2P Port | `19798` |
| RPC Port | `18332` |
| WebSocket Port | `18333` |
| Magic Bytes | `bf591ae7` |
| Target Block Time | 10 minutes |
| Minimum Difficulty | `0x207FFFFF` (testnet) |

## Seed Nodes

Connect to these seed nodes to join the testnet:

```
seed1.testnet.udaya.net:19798
seed2.testnet.udaya.net:19798
seed3.testnet.udaya.net:19798
seed-us.testnet.udaya.net:19798
seed-eu.testnet.udaya.net:19798
```

## Quick Start

### Prerequisites

- Rust 1.75+ (for building from source)
- 4GB RAM minimum
- 50GB disk space
- Port 19798 open for P2P
- Port 18332 open for RPC (optional, localhost only recommended)

### Build from Source

```bash
git clone https://github.com/udayafoundation/udaya.git
cd udaya
cargo build --release
```

### Run a Node

```bash
./target/release/udayad --config config/testnet/bitfury-testnet.conf
```

### Run as System Service (Linux)

```bash
sudo cp deployments/scripts/udaya-node1.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now udaya-node1
```

## Configuration

See `config/testnet/` for example configurations. Key settings:

```toml
[network]
listen_port = 19798
max_peers = 125

[rpc]
listen_addr = "127.0.0.1"
listen_port = 18332
username = "udaya"
password = "your_secure_password"

[mining]
enable = true
mine_on_startup = false
num_miner_threads = 2
```

## Mining

### Solo Mining

```bash
./target/release/udayad --config config/testnet/bitfury-testnet.conf --mine
```

### Mining Pool

```bash
./target/release/udaya-pool --config config/testnet/bitfury-testnet.conf
./target/release/udaya-miner --pool-url stratum+tcp://localhost:3333
```

## RPC Methods

Connect to `http://127.0.0.1:18332/` with JSON-RPC:

```bash
curl -u udaya:password -X POST http://127.0.0.1:18332/ \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}'
```

### Available Methods

- `getblockchaininfo` - Network and chain info
- `getblockcount` - Current block height
- `getblockhash <height>` - Block hash by height
- `getblock <hash>` - Block details
- `gettransaction <txid>` - Transaction details
- `getbalance` - Wallet balance
- `getnewaddress` - Generate new address
- `sendtoaddress <address> <amount>` - Send transaction
- `getmininginfo` - Mining information
- `getblocktemplate` - Block template for mining
- `getpeerinfo` - Connected peers
- `getnetworkinfo` - Network information
- `ping` - Ping server

## Faucet

Request testnet coins from the faucet:

```
https://faucet.testnet.udaya.net
```

## Explorer

View blocks and transactions:

```
https://explorer.testnet.udaya.net
```

## Troubleshooting

### Node won't start

Check logs:
```bash
journalctl -u udaya-node1 -n 50 --no-pager
```

### Can't connect to peers

Ensure port 19798 is open:
```bash
sudo ufw allow 19798/tcp
```

### RPC connection refused

Check RPC is enabled and listening on correct address in config.

### Stuck syncing

Delete data directory and resync:
```bash
sudo systemctl stop udaya-node1
sudo rm -rf /var/lib/udaya/node1/blockchain
sudo systemctl start udaya-node1
```

## Reporting Issues

Please report bugs and issues on GitHub:
https://github.com/udayafoundation/udaya/issues

## Security

Report security vulnerabilities to security@udaya.org. See [SECURITY.md](SECURITY.md) for details.
