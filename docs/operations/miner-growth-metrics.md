# Udaya Miner Growth Metrics

> **Version:** 1.0.0 | **Phase:** 12 — Post-Launch Survivability
> **Owner:** Mining Team

## 1. Purpose

Track the number, distribution, and behaviour of miners on the Udaya network. Mining
decentralisation is critical for network security and long-term viability.

## 2. Mining Metrics Taxonomy

```
Miner Growth
├── Hashrate (network total)
│   ├── Estimated hashrate (TH/s, PH/s)
│   ├── Hashrate distribution by pool
│   └── Hashrate distribution by algorithm
├── Miner Count
│   ├── Unique mining addresses
│   ├── Active workers per pool
│   └── Solo miners vs pool miners
├── Block Distribution
│   ├── Blocks mined per address (rolling 24 h)
│   ├── Block interval per miner
│   └── Coinbase maturity tracking
└── Pool Metrics
    ├── Active pools count
    ├── Pool hashrate share (%)
    ├── Pool fee rates
    └── Pool payout schemes (PPS, PPLNS, Solo)
```

## 3. Core Metrics

### 3.1 Prometheus Metrics

| Metric | Prometheus Name | Type | Labels | Description |
|--------|-----------------|------|--------|-------------|
| Estimated hashrate | `udaya_hashrate_estimate_hps` | Gauge | — | Network hashrate (H/s) |
| Hashrate by pool | `udaya_pool_hashrate_hps` | Gauge | `pool` | Per-pool hashrate |
| Mining addresses | `udaya_mining_address_count` | Gauge | — | Unique coinbase addresses (24 h) |
| Blocks per miner | `udaya_blocks_mined_total` | Counter | `address`, `pool` | Blocks found |
| Active pools | `udaya_active_pools` | Gauge | — | Number of active mining pools |
| Pool block share % | `Udaya:pool_share_percent` | Derived | `pool` | Percentage of recent blocks |
| Difficulty | `udaya_difficulty` | Gauge | — | Current mining difficulty |
| Block reward | `udaya_block_reward` | Gauge | — | Current block reward (incl. fees) |
| Avg miner fee rate | `udaya_miner_fee_rate_sats` | Gauge | — | Mean fee rate set by miners |

### 3.2 Derived Recording Rules

```yaml
groups:
  - name: udaya_miner_metrics
    interval: 5m
    rules:
      - record: Udaya:pool_share_percent
        expr: |
          rate(udaya_blocks_mined_total[24h])
          / ignoring(pool) sum(rate(udaya_blocks_mined_total[24h]))
          * 100

      - record: Udaya:hashrate_growth_7d
        expr: |
          (udaya_hashrate_estimate_hps
           - udaya_hashrate_estimate_hps offset 7d)
          / udaya_hashrate_estimate_hps offset 7d
          * 100

      - record: Udaya:miner_diversity_index
        expr: |
          1 - sum((rate(udaya_blocks_mined_total[24h]) / 
                   sum(rate(udaya_blocks_mined_total[24h])))^2)

      - record: Udaya:mining_pool_concentration
        expr: |
          max by (pool) (rate(udaya_blocks_mined_total[24h]))
          / sum(rate(udaya_blocks_mined_total[24h]))
          * 100
```

### 3.3 Alert Rules

```yaml
- alert: HashrateDrop
  expr: udaya_hashrate_estimate_hps < udaya_hashrate_estimate_hps offset 24h * 0.5
  for: 30m
  labels:
    severity: critical
  annotations:
    summary: "Network hashrate dropped by more than 50 % in 24 hours"
    description: "Hashrate: {{ $value | humanize }} H/s. Was {{ $labels.offset_value | humanize }} H/s 24h ago."

- alert: MiningCentralisation
  expr: Udaya:mining_pool_concentration > 50
  for: 6h
  labels:
    severity: warning
  annotations:
    summary: "Single pool controls > 50 % of hashrate"
    description: "Pool {{ $labels.pool }} controls {{ $value | humanize }}% of network hashrate."

- alert: NoNewMiners
  expr: udaya_mining_address_count < 3
  for: 24h
  labels:
    severity: warning
  annotations:
    summary: "Fewer than 3 unique mining addresses in 24 hours"
    description: "Only {{ $value }} unique miners. Possible mining centralisation."

- alert: DifficultyAdjustmentStalled
  expr: udaya_difficulty == udaya_difficulty offset 2016 * 10m
  for: 1h
  labels:
    severity: warning
  annotations:
    summary: "Difficulty has not changed at the last adjustment interval"
    description: "Difficulty stuck at {{ $value }} for over 2016 blocks."
```

## 4. Grafana Dashboard: Mining Overview

| Panel | Query | Visual |
|-------|-------|--------|
| Network Hashrate | `udaya_hashrate_estimate_hps` | Stat (TH/s or PH/s) |
| Hashrate 7d Change | `Udaya:hashrate_growth_7d` | Stat (% increase/decrease) |
| Pool Hashrate Share | `Udaya:pool_share_percent` | Pie chart |
| Hashrate Trend (30d) | `udaya_hashrate_estimate_hps` | Timeseries |
| Unique Miners (24h) | `udaya_mining_address_count` | Stat |
| Miner Diversity Index | `Udaya:miner_diversity_index` | Gauge (0 = centralised, 1 = fully diverse) |
| Pool Concentration | `Udaya:mining_pool_concentration` | Gauge (red if > 50 %) |
| Blocks Found / Day | `sum(rate(udaya_blocks_mined_total[24h]))` | Stat |
| Difficulty | `udaya_difficulty` | Timeseries |
| Block Reward | `udaya_block_reward` | Timeseries |

## 5. Stratum Pool Metrics

For the official Udaya mining pool (`pool.Udaya.net:3333`):

| Metric | Prometheus Name | Type | Description |
|--------|-----------------|------|-------------|
| Active workers | `udaya_pool_worker_count` | Gauge | Connected mining workers |
| Share rate | `udaya_pool_share_rate` | Gauge | Shares submitted per second |
| Accepted shares | `udaya_pool_accepted_shares_total` | Counter | Valid shares |
| Rejected shares | `udaya_pool_rejected_shares_total` | Counter | Invalid/stale shares |
| Reject rate | `udaya_pool_reject_rate` | Gauge | % rejected |
| Block found | `udaya_pool_blocks_found_total` | Counter | Blocks found by pool |

### 5.1 Pool Alert Rules

```yaml
- alert: PoolHighRejectRate
  expr: udaya_pool_reject_rate > 10
  for: 10m
  labels:
    severity: warning
  annotations:
    summary: "Mining pool reject rate is above 10 %"
    description: "Reject rate: {{ $value | humanize }}% on {{ $labels.instance }}."

- alert: PoolNoBlocksFound
  expr: rate(udaya_pool_blocks_found_total[24h]) == 0
  for: 6h
  labels:
    severity: critical
  annotations:
    summary: "Pool has not found a block in 6 hours"
    description: "No blocks found on {{ $labels.instance }} in the last 6 hours."
```

## 6. Miner Decentralisation Targets

| Metric | Week 1 | Month 1 | Month 3 | Quarter 2 |
|--------|--------|---------|---------|-----------|
| Network hashrate | >= 10 TH/s | >= 100 TH/s | >= 1 PH/s | >= 10 PH/s |
| Unique miners / day | >= 3 | >= 10 | >= 50 | >= 200 |
| Active pools | >= 1 | >= 2 | >= 5 | >= 10 |
| Max pool share | < 100 % | < 70 % | < 50 % | < 40 % |
| Miner diversity index | > 0.1 | > 0.3 | > 0.5 | > 0.7 |

## 7. Mining Pool Onboarding

When a new mining pool joins the network:

```bash
# Pool connection verification script
POOL_HOST=$1
POOL_PORT=${2:-3333}

echo "Testing pool $POOL_HOST:$POOL_PORT..."

# Test TCP connectivity
nc -z "$POOL_HOST" "$POOL_PORT" || { echo "FAIL: Pool unreachable"; exit 1; }

# Test Stratum handshake
echo '{"id":1,"method":"mining.subscribe","params":[]}' | timeout 5 nc "$POOL_HOST" "$POOL_PORT"

# Check if pool is submitting blocks
curl -sf http://localhost:8332 \
  -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getblocksubsidy\",\"params\":[]}" | \
  jq '.result.miner'

echo "Pool $POOL_HOST online and reachable."
```

## 8. Hashrate Estimation

Udaya estimates network hashrate using the standard formula:

```
hashrate = difficulty * 2^32 / block_time
```

Where:
- `difficulty` = current mining difficulty
- `2^32` = share target (constant for SHA-256d)
- `block_time` = 600 s (target) or rolling average

### 8.1 Estimation Accuracy

| Window | Accuracy | Use Case |
|--------|----------|----------|
| Last 24 blocks | ± 30 % | Short-term spikes |
| Last 2016 blocks | ± 10 % | Difficulty adjustment |
| Last 10080 blocks (7d) | ± 5 % | Trend analysis |

## 9. Reporting

Miner metrics feed into:
- [Weekly Operational Report](./weekly-operational-report-template.md)
- Mining dashboard (pool.Udaya.net/dashboard)
- [Network Growth Metrics](./network-growth-metrics.md)
- [Node Growth Metrics](./node-growth-metrics.md)

## 10. References

- [Node Growth Metrics](./node-growth-metrics.md)
- [90-Day Execution Plan](./90-day-execution-plan.md)
- [Stratum Pool Protocol](../mining/stratum-pool.md)