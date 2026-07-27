# Udaya Mining Telemetry Dashboard

## Overview
Real-time telemetry dashboard for mining operations, pool analytics, and decentralization metrics.

## Key Metrics
| Metric | Description | Source |
|--------|-------------|--------|
| Pool Hashrate | Aggregate hashrate across all connected miners | Stratum |
| Miner Hashrate | Per-worker hashrate | Stratum |
| Share Acceptance | Accepted/rejected/stale share ratio | Stratum |
| Block Discovery | Time since last block found | Chain |
| Luck | Actual vs expected blocks found | Stats |

## Decentralization Analytics
- Nakamoto Coefficient: Minimum entities for 51%
- Gini Coefficient: Hashrate distribution equality
- HHI: Herfindahl-Hirschman Index for pool concentration
- Geographic Distribution: Miner location breakdown

## Grafana Dashboard
Available at `deployments/grafana/dashboards/mining-telemetry.json`

### Panels
1. Network Overview: Hashrate, difficulty, block height
2. Pool Performance: Hashrate, shares, miners
3. Miner Distribution: Top miners, geographic map
4. Block History: Blocks found, time between blocks
5. Decentralization: Nakamoto coeff, HHI over time