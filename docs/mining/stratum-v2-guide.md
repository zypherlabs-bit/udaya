# Udaya Stratum V2 Mining Guide

## Overview
Udaya supports Stratum V2 (SV2) for secure, efficient, and decentralized mining.

## Pool Endpoints
| Region | Stratum V2 | SSL | Backup |
|--------|------------|-----|--------|
| Global | pool.Udaya.net:3333 | pool.Udaya.net:3334 | us.pool.Udaya.net:3333 |
| Europe | eu.pool.Udaya.net:3333 | eu.pool.Udaya.net:3334 | - |
| Asia | asia.pool.Udaya.net:3333 | asia.pool.Udaya.net:3334 | - |

## Mining Setup

### ASIC Configuration
```bash
# Antminer / Whatsminer
URL: stratum+tcp://pool.Udaya.net:3333
Worker: your_UDYA_address.worker1
Password: x
```

### CPU/GPU Mining (Testnet Only)
```bash
udayad start --mine --network testnet
```

### Connection Verification
```bash
curl -X POST http://pool.Udaya.net:3333 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"mining.info","params":[]}'
```

## Pool Features
- **PPLNS**: Pay Per Last N Shares (1% fee)
- **PPS**: Pay Per Share (2% fee)
- **Minimum Payout**: 0.01 UDYA
- **Payout Schedule**: Every 6 hours (threshold met)
- **Stale Share Protection**: <1% accepted threshold

## Security
- SSL/TLS encryption on port 3334
- Worker authentication via address-based identification
- DDoS protection via rate limiting
- Real-time hashrate monitoring

## Decentralization Metrics
The pool tracks and publishes:
- Nakamoto coefficient
- Geographic distribution
- Hashrate concentration
- Pool dominance percentage