# Udaya 90-Day Post-Launch Execution Plan

> **Date:** June 15, 2026
> **Version:** 2.0.0
> **Status:** Operational — Phase 12
> **Owner:** Core Dev Team

## Overview

This execution plan defines **what, when, and who** for the first 90 days after mainnet genesis.
It supersedes the earlier v1.0 draft with concrete milestones, named owners, measurable KPIs,
and automated runbooks.

---

## Phase 1: Launch Week (Days 0–7)

### Day 0 — Genesis

| Action | Owner | Verification |
|--------|-------|-------------|
| Mine genesis block & publish manifest | Dev Lead | Genesis hash in docs/security/genesis-verification.md |
| Deploy 4 seed nodes (US-E, US-W, EU, APAC) | Infra | `udayad getpeerinfo` >= 4 outbound |
| Publish genesis block hash to README, website, docs | Docs | All three sources match |
| Enable DNS seeds: `seed.Udaya.net` | Infra | `dig seed.Udaya.net` returns A records |
| Set up initial mining pool at `pool.Udaya.net:3333` | Mining | Stratum handshake succeeds |

### Day 1–2 — Network Bootstrap

| Action | Target | Metric |
|--------|--------|--------|
| Peer connectivity | >= 50 connected peers | `udaya_peer_count` |
| Block propagation | < 30 s per block | `udaya_block_propagation_seconds` p99 |
| Mempool acceptance | First 100 txs accepted | `udaya_mempool_tx_count` |
| Explorer indexing | Block 0 confirmed visible | Explorer /block/0 returns 200 |
| Wallet generate/recover | Generate + recover 5 wallets | Automated test pass |

### Day 3–7 — Stabilisation

| Action | Target | Alert if |
|--------|--------|----------|
| Chain splits | 0 events | `udaya_chain_splits_detected_total` > 0 |
| Orphan rate | < 1 % | `rate(udaya_orphan_blocks_total[1h])` > 0.01 |
| Block time | 600 s ± 120 s | Mean outside [480, 720] over 6 h |
| Difficulty adjustment | First adjustment correct | Block time moving avg converges |
| Monitoring infra | All dashboards green | Any critical alert fires |

### Runbook: Launch-Day Checklist Script

```bash
#!/usr/bin/env bash
# pre-launch-check.sh — exit non-zero if any check fails
set -euo pipefail

echo "[1/8] Genesis block published"
curl -sf http://localhost:8332 \
  -d '{"jsonrpc":"2.0","id":1,"method":"getblock","params":["0"]}' | jq .result.hash

echo "[2/8] Seed nodes reachable"
for seed in us-e.seed.Udaya.net us-w.seed.Udaya.net eu.seed.Udaya.net apac.seed.Udaya.net; do
  dig +short "$seed" || echo "WARNING: $seed not resolved"
done

echo "[3/8] DNS seed returns peers"
dig +short seed.Udaya.net

echo "[4/8] Node listening on 9798"
nc -z localhost 9798 || exit 1

echo "[5/8] RPC responding"
curl -sf http://localhost:8332 \
  -d '{"jsonrpc":"2.0","id":1,"method":"getblockchaininfo","params":[]}' | jq .result.blocks

echo "[6/8] Prometheus metrics"
curl -sf http://localhost:8332/metrics | grep udaya_blocks_total

echo "[7/8] Health endpoint"
curl -sf http://localhost:8332/health | jq .status

echo "[8/8] Explorer reachable"
curl -sf http://explorer.Udaya.net/api/block/0 | jq .hash

echo "All checks passed."
```

---

## Phase 2: Exchange Integration (Days 8–30)

### Week 2 — Exchange Onboarding

| Date | Action | Owner |
|------|--------|-------|
| D+8 | Publish genesis block verification tools | Dev |
| D+9 | Share Exchange Integration Package with first exchange | BD |
| D+10 | Test deposit workflow in exchange sandbox | QA |
| D+12 | Test withdrawal workflow in exchange sandbox | QA |
| D+14 | Verify reorg handling (1, 2, 6 confirmations) | QA |
| D+15 | Document issues, publish errata | Docs |
| D+21 | First exchange listing goes live | BD |

### Wallet & Recovery Verification

| Test | Expected | Pass |
|------|----------|------|
| 12-word mnemonic → address | Deterministic, matches BIP39 | ✓ |
| 24-word mnemonic → address | Deterministic, matches BIP39 | ✓ |
| BIP84 (bech32) deposit | Exchange sees correct address | ✓ |
| Paper wallet sweep | UTXO spends correctly | ✓ |
| Cross-platform restore | Same seed → same keys on Win/Mac/Linux | ✓ |

---

## Phase 3: Growth (Days 31–60)

### Infrastructure Scaling

| Milestone | Target Date | Metric |
|-----------|-------------|--------|
| 10 seed nodes | D+45 | `udaya_peer_count` per node |
| Public RPC endpoints (rate-limited) | D+35 | API uptime > 99.9 % |
| Block explorer production | D+30 | Page load < 2 s |
| Grafana dashboards public | D+35 | Dashboard availability |
| Alertmanager + PagerDuty integration | D+32 | Alert latency < 1 min |

### Community Tools

| Tool | Launch Date | URL |
|------|-------------|-----|
| Faucet (testnet) | D+35 | faucet.Udaya.net |
| Mining dashboard | D+40 | pool.Udaya.net/dashboard |
| Network status page | D+35 | status.Udaya.net |
| Community mining pool guide | D+40 | docs/mining/pool-setup.md |

---

## Phase 4: Maturity (Days 61–90)

### Security & Resilience

| Activity | Date | Artifact |
|----------|------|----------|
| External security audit (firm) | D+60–75 | Published audit report |
| Bug bounty programme launch | D+70 | HackerOne / Immunefi page |
| Penetration test results review | D+75 | Risk register update |
| Emergency upgrade drill | D+80 | Drill report |

### Ecosystem

| Deliverable | Date | Owner |
|-------------|------|-------|
| SDK v1.0 (Rust, Python, JS) | D+60 | Dev |
| Developer portal | D+65 | Web |
| Governance proposal #1 | D+70 | Community |
| Grant programme announcement | D+75 | Foundation |

---

## Automated Monitoring Runbook

### Every 5 Minutes (Prometheus + Alertmanager)

| Check | Alert | Severity |
|-------|-------|----------|
| Node process alive | NodeDown | critical |
| Block interval < 1 h | BlockProductionStopped | critical |
| Peer count >= 8 | LowPeerCount | warning |
| Mempool < 200 MB | MempoolSizeHigh | warning |
| Orphan rate < 5 /min | HighOrphanRate | warning |
| Chain splits == 0 | ChainSplitDetected | critical |

### Every 24 Hours (Cron Job)

```bash
#!/usr/bin/env bash
# daily-health-summary.sh
echo "=== Udaya Daily Health Summary ==="
echo "Date: $(date -u)"
echo "Block height: $(curl -sf http://localhost:8332 -d '{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}' | jq .result)"
echo "Peers: $(curl -sf http://localhost:8332/metrics | grep udaya_peer_count | grep -v '#' | awk '{print $2}')"
echo "Mempool tx count: $(curl -sf http://localhost:8332/metrics | grep udaya_mempool_tx_count | grep -v '#' | awk '{print $2}')"
echo "Uptime: $(curl -sf http://localhost:8332/metrics | grep udaya_node_uptime_seconds | grep -v '#' | awk '{print $2}')s"
```

---

## KPIs & Success Criteria

| Metric | Week 1 | Week 4 | Week 12 |
|--------|--------|--------|---------|
| Connected nodes | >= 50 | >= 200 | >= 1 000 |
| Mining hashrate | >= 10 TH/s | >= 100 TH/s | >= 1 PH/s |
| Daily transactions | >= 100 | >= 1 000 | >= 10 000 |
| Exchange listings | 0 | 2 | >= 5 |
| GitHub stars | >= 100 | >= 500 | >= 2 000 |
| Community members | >= 500 | >= 5 000 | >= 20 000 |

## Emergency Contacts

| Role | Contact | SLA |
|------|---------|-----|
| Core developer on-call | dev-oncall@Udaya.net | 15 min |
| Infrastructure | infra@Udaya.net | 30 min |
| Security | security@Udaya.net | Immediate |
| Exchange support | exchange@Udaya.net | 1 hour |
| Community | community@Udaya.net | 2 hours |

---

## Appendices

- **A**: [Crash Monitoring](./crash-monitoring.md)
- **B**: [Chain Split Detection](./chain-split-detection.md)
- **C**: [Orphan Rate Monitoring](./orphan-rate-monitoring.md)
- **D**: [Network Growth Metrics](./network-growth-metrics.md)
- **E**: [Node Growth Metrics](./node-growth-metrics.md)
- **F**: [Miner Growth Metrics](./miner-growth-metrics.md)
- **G**: [Weekly Operational Report Template](./weekly-operational-report-template.md)