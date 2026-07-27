# Udaya Orphan Rate Monitoring

> **Version:** 1.0.0 | **Phase:** 12 — Post-Launch Survivability
> **Owner:** Core Dev Team

## 1. What Is an Orphan Block?

An **orphan block** is a valid block that is not part of the canonical chain. This happens when:

1. Two miners find a block at roughly the same height
2. The network propagates both blocks
3. One block arrives slightly later and is discarded when the other block's chain extends first
4. The discarded block becomes an orphan

> **Note:** Orphans are distinct from **stale blocks** (blocks mined on top of an already-orphaned parent) and **invalid blocks** (blocks that fail consensus rules).

## 2. Why Orphan Rate Matters

| Metric | Healthy | Warning | Critical |
|--------|---------|---------|----------|
| Orphan rate | < 1 % | 1–5 % | > 5 % |
| Network latency (block propagation) | < 2 s | 2–10 s | > 10 s |
| Peak orphan bursts | < 5 / h | 5–20 / h | > 20 / h |

A high orphan rate indicates:
- **Poor network connectivity** — blocks take too long to propagate
- **Mining centralisation** — large pools produce orphans for smaller miners
- **Network congestion** — bandwidth saturation delays block relay
- **Consensus divergence** — different node versions producing incompatible blocks (rare)

## 3. Metrics

### 3.1 Prometheus Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `udaya_orphan_blocks_total` | Counter | `cause` | Total orphan blocks received |
| `udaya_orphan_rate_ratio` | Gauge | — | Orphans / total blocks (rolling 1 h) |
| `udaya_orphan_block_height` | Gauge | — | Height of most recent orphan |
| `udaya_orphan_block_age_seconds` | Gauge | — | Time between orphan and canonical block at same height |
| `udaya_block_propagation_seconds` | Histogram | — | Time to receive a new block from first announcement |
| `udaya_block_propagation_bytes_total` | Counter | — | Bandwidth used for block relay |

### 3.2 Derived Metrics (Recording Rules)

```yaml
# Prometheus recording rules
groups:
  - name: udaya_orphan_derived
    interval: 1m
    rules:
      - record: Udaya:orphan_rate_1h
        expr: |
          rate(udaya_orphan_blocks_total[1h])
          /
          (rate(udaya_blocks_total[1h]) + rate(udaya_orphan_blocks_total[1h]))
          * 100

      - record: Udaya:orphan_rate_24h
        expr: |
          rate(udaya_orphan_blocks_total[24h])
          /
          (rate(udaya_blocks_total[24h]) + rate(udaya_orphan_blocks_total[24h]))
          * 100
```

### 3.3 Alert Rules

```yaml
- alert: HighOrphanRate
  expr: Udaya:orphan_rate_1h > 1
  for: 10m
  labels:
    severity: warning
  annotations:
    summary: "Orphan rate above 1 %"
    description: "Orphan rate is {{ $value | humanize }}% over the last hour on {{ $labels.instance }}."

- alert: CriticalOrphanRate
  expr: Udaya:orphan_rate_1h > 5
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "Orphan rate critical — above 5 %"
    description: "Orphan rate is {{ $value | humanize }}% over the last hour on {{ $labels.instance }}. Investigate network topology immediately."

- alert: OrphanBurstDetected
  expr: rate(udaya_orphan_blocks_total[5m]) > 5
  for: 2m
  labels:
    severity: warning
  annotations:
    summary: "Burst of orphan blocks detected"
    description: "{{ $value }} orphans/min in last 5 min. Possible network partition or latency spike."
```

## 4. Orphan Investigation Runbook

### 4.1 When Alert Fires

```bash
# Step 1: Get current orphan count
curl -sf http://localhost:8332/metrics | grep udaya_orphan_blocks_total

# Step 2: Get peer latency distribution
curl -sf http://localhost:8332 \
  -d '{"jsonrpc":"2.0","id":1,"method":"getpeerinfo","params":[]}' | \
  jq '.result[] | {addr: .addr, ping_time: .pingtime, height: .height}' | \
  jq -s 'sort_by(.ping_time)'

# Step 3: Get recent orphan details (if node exposes them)
curl -sf http://localhost:8332 \
  -d '{"jsonrpc":"2.0","id":1,"method":"getorphaninfo","params":[]}'
```

### 4.2 Common Causes & Fixes

| Symptom | Likely Cause | Fix |
|---------|-------------|-----|
| High ping to all peers (> 500 ms) | Poor network connectivity | Add geographically closer peers |
| High ping to specific peers | Remote/faulty peers | Ban high-latency peers |
| Orphan bursts correlate with large blocks | Block relay bandwidth insufficient | Increase `max_relay_bandwidth` |
| Orphans from single pool | Pool mining with high latency | Contact pool operator |
| Orphans across all nodes | Network-wide latency (e.g. DDoS) | Activate DDoS protection, add nodes |
| Orphans after software update | Consensus incompatibility | Roll back or patch |

### 4.3 Mitigation Commands

```bash
# Increase peer connection limit to improve propagation
udayad set --max-peers=250

# Add specific low-latency peers
udayad addnode "seed-us-west.Udaya.net:9798" add

# Ban a peer causing high orphan rate
udayad setban "192.168.1.100" "86400"

# Enable compact block relay (BIP152)
udayad set --compact-blocks=true

# Increase relay bandwidth (default: 5 MB/s)
udayad set --max-relay-bandwidth=20971520  # 20 MB/s
```

## 5. Grafana Dashboard: Orphan Monitor

| Panel | Query | Visual |
|-------|-------|--------|
| Orphan Rate (1 h) | `Udaya:orphan_rate_1h` | Gauge (green < 1 %, yellow 1–5 %, red > 5 %) |
| Orphan Rate (24 h) | `Udaya:orphan_rate_24h` | Stat |
| Orphans / min | `rate(udaya_orphan_blocks_total[5m])` | Timeseries |
| Block Propagation (p50/p95/p99) | `histogram_quantile(0.5/0.95/0.99, rate(udaya_block_propagation_seconds_bucket[5m]))` | Timeseries |
| Peer Ping Times | `udaya_peer_ping_seconds` | Table (top 20 slowest) |
| Orphans by Cause | `sum by (cause) (udaya_orphan_blocks_total)` | Pie chart |

## 6. Targets & Service Level

| Timeframe | Target Orphan Rate | Action if Exceeded |
|-----------|-------------------|-------------------|
| Launch week (D0–7) | < 2 % | Acceptable during bootstrap |
| Month 1 | < 1.5 % | Review peer topology |
| Month 2 | < 1 % | Optimise relay settings |
| Month 3+ | < 0.5 % | Continuous improvement |

## 7. Automated Recovery

```python
#!/usr/bin/env python3
"""auto-orphan-mitigation.py — runs as a cron job or on alert webhook."""
import subprocess
import json
import sys

RATE_THRESHOLD = 5.0  # orphans/min

def get_orphan_rate():
    metrics = subprocess.check_output(
        ["curl", "-sf", "http://localhost:8332/metrics"]
    ).decode()
    for line in metrics.splitlines():
        if line.startswith("udaya_orphan_blocks_total"):
            return float(line.split()[1])
    return 0.0

def ban_slow_peers():
    peers = json.loads(subprocess.check_output([
        "curl", "-sf", "http://localhost:8332",
        "-d", '{"jsonrpc":"2.0","id":1,"method":"getpeerinfo","params":[]}'
    ]))["result"]
    for p in peers:
        if p.get("pingtime", 0) > 2.0:  # > 2s ping
            subprocess.run([
                "udayad", "setban", p["addr"], "3600"
            ])
            print(f"Banned {p['addr']} (ping: {p['pingtime']}s)")

if __name__ == "__main__":
    rate = get_orphan_rate()
    print(f"Current orphan count: {rate}")
    if rate > RATE_THRESHOLD:
        print("Orphan rate high — banning slow peers")
        ban_slow_peers()
```

## 8. References

- [Chain Split Detection](./chain-split-detection.md)
- [Network Growth Metrics](./network-growth-metrics.md)
- [90-Day Execution Plan](./90-day-execution-plan.md)
- [Alerting Rules](../scripts/monitoring/alerts/udaya_alerts.yml)