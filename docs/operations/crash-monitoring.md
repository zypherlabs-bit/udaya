# Udaya Crash Monitoring

> **Version:** 1.0.0 | **Phase:** 12 — Post-Launch Survivability
> **Owner:** Core Dev Team

## 1. Objectives

- Detect node process crashes within 30 seconds.
- Capture crash context (signal, stack, state) for root-cause analysis.
- Track crash frequency per version, per topology, per deployment.
- Provide automated alerting and post-mortem data collection.

## 2. Crash Detection Architecture

```
Node Process
    │
    ├── Process Supervisor (systemd / Docker restart policy)
    │       └── Reports exit code + signal to journald / Docker logs
    │
    ├── Health Check (Prometheus exporter)
    │       └── udaya_node_status → 0 on crash → Alertmanager
    │
    └── Heartbeat Watchdog (external)
            └── Every 30s: GET /health → 503 → page on-call
```

### 2.1 Prometheus Metric

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `udaya_crashes_total` | Counter | `version`, `signal`, `exit_code` | Total node crashes |
| `udaya_node_uptime_seconds` | Gauge | — | Seconds since process start |
| `udaya_node_restarts_total` | Counter | `reason` | Total restarts by trigger |

### 2.2 Alert Rules

```yaml
# Prometheus alert rule
- alert: NodeCrashed
  expr: udaya_node_status == 0
  for: 30s
  labels:
    severity: critical
  annotations:
    summary: "Udaya node has crashed"
    description: "Node {{ $labels.instance }} went down at {{ $value }}. Exit code may be in journald."

- alert: FrequentRestartLoop
  expr: rate(udaya_node_restarts_total[15m]) > 2
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "Node is restarting frequently — crash loop detected"
    description: "Node {{ $labels.instance }} restarted {{ $value }} times in the last 15 minutes."
```

## 3. Crash Data Collection

### 3.1 Linux (systemd)

```bash
# View last crash logs
journalctl -u udayad.service --since "5 minutes ago" --no-pager

# Capture core dump (requires ulimit -c unlimited)
coredumpctl list
coredumpctl info udayad

# Persistent crash log
journalctl -u udayad.service --since "1 hour ago" > /var/log/Udaya/crash-$(date +%Y%m%d-%H%M%S).log
```

### 3.2 Docker

```bash
# Check container restart count
docker inspect --format '{{ .RestartCount }}' Udaya-node

# View last container logs before crash
docker logs --tail 200 Udaya-node > crash-$(date +%Y%m%d-%H%M%S).log
```

### 3.3 Structured Crash Report

On crash, the node SHOULD write a JSON crash report to the data directory:

```json
{
  "timestamp": "2026-06-15T12:34:56Z",
  "version": "1.0.0",
  "commit": "a1b2c3d4",
  "signal": "SIGSEGV",
  "exit_code": 139,
  "last_block_height": 87412,
  "last_block_hash": "00000000abcdef...",
  "peer_count": 47,
  "mempool_size": 823,
  "uptime_seconds": 293847,
  "rust_backtrace": "..." 
}
```

## 4. Crash Trends Dashboard (Grafana)

### Panel: Crash Rate (7d)

```
Query:  rate(udaya_crashes_total[24h])
Visual: Stat — "Crashes / day"
Threshold: > 1 → red
```

### Panel: Crash Distribution by Version

```
Query:  sum by (version) (udaya_crashes_total)
Visual: Pie chart
```

### Panel: Topology Crash Map

```
Query:  sum by (instance) (udaya_crashes_total)
Visual: Table — sort descending
```

## 5. Post-Crash Actions

| Step | Action | Owner | Time |
|------|--------|-------|------|
| 1 | Acknowledge alert | On-call | < 5 min |
| 2 | Collect crash report + logs | On-call | < 15 min |
| 3 | Restart node if auto-restart didn't trigger | On-call | < 20 min |
| 4 | Classify severity | On-call Lead | < 30 min |
| 5 | Create GitHub issue with crash artifacts | On-call | < 1 h |
| 6 | Root-cause analysis | Dev | < 24 h |
| 7 | Deploy fix / workaround | Dev | Per severity |
| 8 | Post-mortem | Team | < 48 h |

## 6. Crash Severity Classification

| Severity | Definition | SLA |
|----------|-----------|-----|
| SEV-1 | Crash causes network split, fund loss, or > 50 % nodes offline | 15 min response |
| SEV-2 | Crash affects > 10 % of nodes or critical infrastructure | 1 h response |
| SEV-3 | Single node crash, non-deterministic, not reproducible | 24 h response |

## 7. Testing Crash Recovery

```bash
# Test 1: Kill process and verify auto-restart
kill -9 $(pgrep udayad)
sleep 5
pgrep udayad || echo "FAIL: Node did not restart"

# Test 2: Simulate OOM
stress --vm 2 --vm-bytes 2G --timeout 30s
sleep 10
curl -sf http://localhost:8332/health | jq .status

# Test 3: Verify crash counter incremented
curl -sf http://localhost:8332/metrics | grep udaya_node_restarts_total
```

## 8. References

- [Incident Response Playbook](./incident-response.md)
- [Observability Stack](./observability.md)
- [Alerting Rules](../scripts/monitoring/alerts/udaya_alerts.yml)