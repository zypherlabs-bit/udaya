# Udaya Testnet - Mining Guide

## Overview

Udaya uses SHA-256d Proof-of-Work (PoW) consensus, similar to Bitcoin. Miners compete to find valid block headers by iterating through nonces. The first miner to find a valid hash below the target broadcasts the block and receives the block reward.

## Block Reward

- **Current Reward**: 50 UDYA per block
- **Halving Interval**: 210,000 blocks (~4 years)
- **Target Block Time**: 10 minutes

## Solo Mining

### CPU Mining (Testing Only)

```bash
# Start node with mining enabled
./target/release/udayad --config config/testnet/bitfury-testnet.conf --mine

# Or via config
[mining]
enable = true
mine_on_startup = true
num_miner_threads = 4
```

### GPU Mining

Udaya supports Stratum V2 mining protocol for GPU/ASIC miners:

```bash
# Start the mining pool
./target/release/udaya-pool --config config/testnet/bitfury-testnet.conf

# Connect your miner
./target/release/udaya-miner --pool-url stratum+tcp://localhost:3333 --wallet-address YOUR_ADDRESS
```

## Stratum V2 Protocol

The Udaya mining pool implements Stratum V2 with:

- Job distribution and difficulty adjustment
- ASIC optimization profiles
- Miner telemetry
- Fair payout distribution

## Mining Configuration

```toml
[mining]
enable = true
mine_on_startup = false
num_miner_threads = 2
coinbase_address = "YOUR_UDYA_ADDRESS"
```

## Monitoring Mining

```bash
# Via RPC
curl -u udaya:password -X POST http://127.0.0.1:18332/ \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getmininginfo","params":[]}'
```

## Tips

1. Testnet difficulty is very low - blocks mine quickly
2. Testnet coins have no real value
3. Join the Discord for testnet mining pools
4. Report any mining issues on GitHub
