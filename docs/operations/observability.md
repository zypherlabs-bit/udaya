# Udaya Observability Stack

## Architecture
```
Application → OpenTelemetry SDK → Collector → Backends
                                                 ↓
                         ┌─────────────────────────────────┐
                         │  Prometheus (Metrics)            │
                         │  Loki (Logs)                     │
                         │  Tempo (Traces)                  │
                         └─────────────────────────────────┘
                                     ↓
                         ┌─────────────────────────────────┐
                         │  Grafana (Visualization)         │
                         │  Alertmanager (Alerting)         │
                         └─────────────────────────────────┘
```

## Prometheus Configuration
```yaml
scrape_configs:
  - job_name: 'Udaya-node'
    static_configs:
      - targets: ['localhost:8332']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

## Key Metrics
| Metric | Type | Description |
|--------|------|-------------|
| udaya_blocks_total | Counter | Total blocks processed |
| udaya_peer_count | Gauge | Connected peers |
| udaya_mempool_size | Gauge | Mempool transaction count |
| udaya_block_processing_seconds | Histogram | Block validation time |
| udaya_network_bytes_total | Counter | Network bandwidth |

## Logging (JSON Structured)
```json
{"level":"info","ts":"2026-05-28T12:00:00Z","msg":"block received","height":789412,"hash":"0000...","tx_count":1842,"size":1240000}
```

## Grafana Dashboards
- `node-health.json`: CPU, memory, disk, network
- `chain-monitor.json`: Block height, hashrate, difficulty
- `mining-telemetry.json`: Pool hashrate, miner distribution
- `governance.json`: Proposals, votes, treasury