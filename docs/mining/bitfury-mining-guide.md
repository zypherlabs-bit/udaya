# Udaya Mining Operations Guide

## Overview
Udaya uses SHA-256d Proof-of-Work, compatible with Bitcoin ASIC miners. This guide covers both solo mining and pool mining operations.

## Network Parameters

| Parameter | Value |
|-----------|-------|
| Algorithm | SHA-256d |
| Block Time | 600 seconds (10 min) |
| Initial Reward | 50 UDYA |
| Halving Interval | 210,000 blocks |
| Difficulty Adj. | Every 2,016 blocks |
| P2P Port | 9798 (mainnet) |
| Stratum Port | 3333 (pool) |

## Solo Mining

### Running a Solo Mining Node
```bash
# Start with mining enabled
udayad start --mine --miner-threads 4

# Or with custom configuration
udayad -c config/mainnet/udaya-mainnet.conf start --miner-threads 8
```

### Solo Mining Configuration
```toml
[mining]
enable = true
num_miner_threads = 4
coinbase_address = "<YOUR_UDYA_ADDRESS>"
```

## Pool Mining

### Connecting to Udaya Pool
```bash
# Using bfgminer
bfgminer -o stratum+tcp://pool.Udaya.net:3333 -u <worker> -p <password>

# Using cgminer
cgminer -o stratum+tcp://pool.Udaya.net:3333 -u <worker> -p <password>

# Using nicehash
nicehash -a sha256 -o stratum+tcp://pool.Udaya.net:3333 -u <worker>
```

### Pool Endpoints
| Port | Protocol | Description |
|------|----------|-------------|
| 3333 | Stratum V2 | Standard mining |
| 3334 | Stratum V2+SSL | Encrypted mining |

### Pool API
```
GET /api/pool/info       - Pool statistics
GET /api/pool/miners     - Connected miners
GET /api/pool/miner/<name> - Individual miner stats
GET /api/pool/blocks     - Mined blocks
POST /api/pool/submit    - Submit share
```

## Pool Response Examples

### Pool Info
```json
{
  "name": "Udaya Pool",
  "hashrate_ghs": 12500.5,
  "miners_connected": 42,
  "total_blocks": 156,
  "total_shares": 1048576,
  "pool_fee_percent": 1.0,
  "min_payout_UDYA": 1.0,
  "status": "active"
}
```

### Miner Stats
```json
{
  "name": "worker1",
  "hashrate_ghs": 125.5,
  "valid_shares": 15000,
  "invalid_shares": 23,
  "last_share_ago": 5,
  "estimated_daily_UDYA": 0.0125
}
```

## ASIC Compatibility

Udaya's SHA-256d algorithm is compatible with all Bitcoin ASIC miners:

| Manufacturer | Models | Compatible |
|-------------|--------|------------|
| Bitmain | Antminer S19, S21 series | ✅ Yes |
| MicroBT | Whatsminer M50, M60 series | ✅ Yes |
| Canaan | Avalon A12xx, A13xx series | ✅ Yes |
| Udaya | BF-ASIC series | ✅ Yes |

## Difficulty Adjustment

- Difficulty recalculates every 2,016 blocks (~2 weeks)
- Uses Bitcoin Difficulty Adjustment Algorithm (DAA)
- Maximum adjustment: 4x up or down
- Fast re-target uses last 6 blocks for emergency adjustment

## Block Reward Schedule

| Height Range | Reward per Block | Total Coins |
|-------------|-----------------|-------------|
| 0 - 209,999 | 50 UDYA | 10,500,000 |
| 210,000 - 419,999 | 25 UDYA | 5,250,000 |
| 420,000 - 629,999 | 12.5 UDYA | 2,625,000 |
| 630,000 - 839,999 | 6.25 UDYA | 1,312,500 |
| ... halving continues every 210,000 blocks | | |

## Orphan & Reorg Handling

- Maximum reorg depth: 6 blocks
- Orphan rate expected: <0.1% under normal conditions
- Miners should wait for 2+ confirmations before considering block final

## Monitoring

### Health Check
```bash
curl http://localhost:8332/health
# {"status":"ok","service":"udayad","version":"1.0.0"}
```

### Miner Monitoring
```bash
# Using pool API
curl http://pool.Udaya.net:9090/api/pool/info

# Using RPC
udayad getmininginfo
```

## Best Practices

1. **Use a dedicated mining address** for each worker
2. **Monitor share acceptance rate** - should be >99%
3. **Set up failover pools** for redundancy
4. **Run stratum on separate port** from P2P
5. **Enable SSL** for stratum connections (port 3334)
6. **Configure proper difficulty** based on hashrate
7. **Monitor orphan rate** during network instability
8. **Keep mining software updated** for protocol improvements

## Troubleshooting

| Issue | Solution |
|-------|----------|
| High orphan rate | Check network connectivity, reduce stale shares |
| Low hashrate reporting | Verify stratum connection, check miner settings |
| Connection refused | Verify pool is running, check firewall rules |
| Authentication failed | Verify worker name and password |
| High invalid share rate | Check miner configuration, update firmware |