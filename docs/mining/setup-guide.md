# Udaya Mining Setup Guide

## Hardware Requirements
### ASIC Miners (Recommended)
| Model | Hashrate | Power | Connection |
|-------|----------|-------|------------|
| Antminer S19 Pro | 110 TH/s | 3250W | Stratum V2 |
| Antminer S19 XP | 140 TH/s | 3010W | Stratum V2 |
| Whatsminer M50S | 126 TH/s | 3276W | Stratum V2 |
| Udaya BF-ASIC S1 | 100 GH/s | 3250W | Stratum V2 |

### CPU/GPU Mining (Testnet Only)
- CPU: 8+ cores recommended
- GPU: NVIDIA CUDA or AMD OpenCL
- RAM: 8GB minimum
- Storage: 50GB SSD

## Pool Connection

### Method 1: ASIC Configuration
Navigate to your ASIC miner's web interface:
1. Go to Miner Configuration
2. Set URL: `stratum+tcp://pool.Udaya.net:3333`
3. Set Worker: `YOUR_UDYA_ADDRESS.WorkerName`
4. Set Password: `x` (or any value)
5. Save and restart miner

### Method 2: CPU/GPU Mining (Testnet)
```bash
# Run full node with mining enabled
udayad --network testnet start --mine

# With specific thread count
udayad --network testnet start --mine --miner-threads 4
```

### Method 3: bfgminer (Advanced)
```bash
bfgminer -o stratum+tcp://pool.Udaya.net:3333 \
  -u YOUR_UDYA_ADDRESS \
  -p x \
  --stratam
```

## Solo Mining (Advanced)
```bash
udayad start --mine
```
Requires full node with >1 PH/s hashrate to be competitive.

## Verification
Check your mining status:
```bash
curl -X POST http://localhost:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getmininginfo","params":[]}'
```

## Troubleshooting
| Problem | Solution |
|---------|----------|
| No connection | Check firewall (port 3333/3334) |
| High reject rate | Reduce overclock, check network latency |
| Low hashrate | Update firmware, check power supply |
| Stale shares | Switch to closer pool region |