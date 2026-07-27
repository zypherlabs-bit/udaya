# Udaya Network Growth Metrics

> **Version:** 1.0.0 | **Phase:** 12 — Post-Launch Survivability
> **Owner:** Core Dev Team

## 1. Purpose

Track the overall health and growth trajectory of the Udaya network. These metrics inform
capacity planning, infrastructure scaling decisions, and community growth reporting.

## 2. Core Network Metrics

### 2.1 Block Chain Metrics

| Metric | Prometheus Name | Type | Description |
|--------|-----------------|------|-------------|
| Block height | `udaya_block_height` | Gauge | Current chain tip height |
| Block interval (rolling avg) | `udaya_block_interval_seconds` | Gauge | Mean time between blocks over last 100 |
| Total transactions | `udaya_transactions_total` | Counter | All-time transaction count |
| Daily transaction volume | `rate(udaya_transactions_total[24h])` | Derived | Transactions per day |
| Average block size | `udaya_block_size_bytes` | Gauge | Mean block size in bytes |
| Block size p99 | `histogram_quantile(0.99, rate(udaya_block_size_bytes_bucket[24h]))` | Derived | Peak block size |
| UTXO set size | `udaya_utxo_set_size` | Gauge | Current UTXO count |
| UTXO set growth | `rate(udaya_utxo_set_size[24h])` | Derived | UTXOs added per day |

### 2.2 Transaction Metrics

| Metric | Prometheus Name | Type | Description |
|--------|-----------------|------|-------------|
| Mempool tx count | `udaya_mempool_tx_count` | Gauge | Transactions waiting for confirmation |
| Mempool size bytes | `udaya_mempool_size_bytes` | Gauge | Memory used by mempool |
| Mempool growth rate | `rate(udaya_mempool_tx_count[5m])` | Derived | Tx/s entering mempool |
| Transaction throughput | `rate(udaya_transactions_confirmed_total[1h])` | Derived | TPS confirmed |
| Avg fee per tx | `udaya_fee_per_byte` | Gauge | Mean fee (sat/vB) |
| Fee percentiles (p25/p50/p75/p95) | `udaya_fee_bytes_bucket` | Histogram | Fee distribution |

### 2.3 Network Capacity Metrics

| Metric | Target | Critical |
|--------|--------|----------|
| Block size headroom | < 75 % of max block size | > 95 % |
| Mempool saturation | < 30 % of max mempool | > 80 % |
| TPS vs max TPS | < 50 % of theoretical max | > 90 % |
| UTXO growth rate | < 10 % per month | > 30 % per month |

## 3. Prometheus Configuration

```yaml
scrape_configs:
  - job_name: 'Udaya-network'
    static_configs:
      - targets:
          - 'seed-us-west.Udaya.net:8332'
          - 'seed-us-east.Udaya.net:8332'
          - 'seed-eu.Udaya.net:8332'
          - 'seed-apac.Udaya.net:8332'
    metrics_path: '/metrics'
    scrape_interval: 30s
    # Aggregate across seed nodes for a network-wide view
```

### 3.1 Recording Rules

```yaml
groups:
  - name: udaya_network_growth
    interval: 5m
    rules:
      - record: Udaya:network_daily_tx_volume
        expr: rate(udaya_transactions_confirmed_total[24h])

      - record: Udaya:network_daily_blocks
        expr: rate(udaya_blocks_total[24h])

      - record: Udaya:network_avg_block_size
        expr: avg_over_time(udaya_block_size_bytes[24h])

      - record: Udaya:network_tps_avg_1h
        expr: rate(udaya_transactions_confirmed_total[1h]) / 3600

      - record: Udaya:network_utxo_growth_30d
        expr: |
          (udaya_utxo_set_size 
           - udaya_utxo_set_size offset 30d)
          / udaya_utxo_set_size offset 30d
          * 100
```

## 4. Grafana Dashboard: Network Growth

| Panel | Query | Visual |
|-------|-------|--------|
| Block Height | `udaya_block_height` | Stat + sparkline |
| Daily Transactions | `Udaya:network_daily_tx_volume` | Timeseries (7d, 30d, 90d) |
| Daily Blocks | `Udaya:network_daily_blocks` | Timeseries |
| TPS (1h avg) | `Udaya:network_tps_avg_1h` | Gauge |
| Block Size Trend | `Udaya:network_avg_block_size` | Timeseries |
| Block Size Distribution | `histogram_quantile(0.5/0.95/0.99, rate(udaya_block_size_bytes_bucket[7d]))` | Timeseries |
| UTXO Set Size | `udaya_utxo_set_size` | Area chart |
| UTXO Growth (30d) | `Udaya:network_utxo_growth_30d` | Stat (% change) |
| Mempool Saturation | `udaya_mempool_size_bytes / udaya_mempool_max_bytes * 100` | Gauge |
| Fee Percentiles | `histogram_quantile(0.5/0.75/0.95, rate(udaya_fee_per_byte_bucket[24h]))` | Timeseries |

## 5. Weekly Trend Analysis

Run every Monday at 09:00 UTC:

```bash
#!/usr/bin/env bash
# weekly-network-trend.sh
PROMETHEUS="http://localhost:9090"

echo "=== Udaya Network Weekly Trend ==="

echo "--- Block Production ---"
curl -sf "$PROMETHEUS/api/v1/query?query=Udaya:network_daily_blocks" | jq '.data.result[0].value[1]'

echo "--- Transaction Volume ---"
curl -sf "$PROMETHEUS/api/v1/query?query=Udaya:network_daily_tx_volume" | jq '.data.result[0].value[1]'

echo "--- UTXO Growth (30d) ---"
curl -sf "$PROMETHEUS/api/v1/query?query=Udaya:network_utxo_growth_30d" | jq '.data.result[0].value[1]'

echo "--- MemPool Size ---"
curl -sf "$PROMETHEUS/api/v1/query?query=udaya_mempool_size_bytes" | jq '.data.result[0].value[1]'
```

## 6. Capacity Planning Triggers

| Trigger Condition | Action | Timeline |
|-------------------|--------|----------|
| Block size > 75 % of max 7d running | Propose block size increase | 2 weeks |
| Mempool > 80 % for > 24 h | Scale out RPC nodes, increase relay bandwidth | Immediate |
| UTXO growth > 30 % / month | Implement UTXO commitment scheme | 1 month |
| TPS > 50 % of theoretical max | Begin layer-2 / sidechain evaluation | 1 month |
| Mempool backlog > 6 h of tx volume | Increase block gas/weight limit | 1 week |

## 7. Reporting

Network growth metrics feed into:
- [Weekly Operational Report](./weekly-operational-report-template.md)
- Network status page (status.Udaya.net)
- [Node Growth Metrics](./node-growth-metrics.md) — correlation with node count
- [Miner Growth Metrics](./miner-growth-metrics.md) — correlation with hashrate

## 8. References

- [Orphan Rate Monitoring](./orphan-rate-monitoring.md)
- [Node Growth Metrics](./node-growth-metrics.md)
- [90-Day Execution Plan](./90-day-execution-plan.md)