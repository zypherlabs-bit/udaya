# Udaya Chain Split Detection

> **Version:** 1.0.0 | **Phase:** 12 — Post-Launch Survivability
> **Owner:** Core Dev Team

## 1. What Constitutes a Chain Split

A chain split (fork) occurs when two or more nodes disagree on the canonical chain tip.
This can be:

| Type | Cause | Severity |
|------|-------|----------|
| **Accidental fork** | Network latency, simultaneous block finds | Usually < 1 block deep |
| **Persistent fork** | Consensus rule divergence (e.g. different version, invalid block) | Critical—requires immediate action |
| **Malicious fork** | 51 % attack, double-spend attempt | Critical—requires network-wide response |

## 2. Detection Mechanisms

### 2.1 Local Node Detection

Each node MUST detect a split by comparing chain tips with connected peers:

```rust
/// Pseudocode for split detection on the node
fn detect_chain_split(peer_tips: HashMap<PeerId, BlockHash>) -> Option<SplitEvent> {
    let local_tip = chain.get_tip_hash();
    let conflicting: Vec<_> = peer_tips
        .into_iter()
        .filter(|(_, tip)| *tip != local_tip)
        .collect();

    if conflicting.len() >= 3 {
        // At least 3 peers disagree → probable split
        let split_height = find_split_height(conflicting);
        Some(SplitEvent {
            peers_on_other_chain: conflicting.len(),
            split_height,
            local_tip,
        })
    } else {
        None
    }
}
```

### 2.2 Prometheus Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `udaya_chain_splits_detected_total` | Counter | `cause` | Total chain splits detected |
| `udaya_chain_splits_active` | Gauge | — | Currently active splits (0 or 1 normally) |
| `udaya_chain_split_depth` | Gauge | — | Depth of current split in blocks |
| `udaya_peers_on_other_chain` | Gauge | — | Peers on conflicting chain |
| `udaya_tip_divergence_seconds` | Gauge | — | Time since common ancestor |

### 2.3 Alert Rules

```yaml
- alert: ChainSplitDetected
  expr: rate(udaya_chain_splits_detected_total[5m]) > 0
  for: 1m
  labels:
    severity: critical
  annotations:
    summary: "Chain split detected"
    description: "Node {{ $labels.instance }} detected {{ $value }} split event(s) in last 5 min. Peers on other chain: check udaya_peers_on_other_chain."

- alert: ChainSplitPersistent
  expr: udaya_chain_splits_active > 0
  for: 10m
  labels:
    severity: critical
  annotations:
    summary: "Chain split is persisting — possible consensus failure"
    description: "Active split for > 10 min on {{ $labels.instance }}. Depth: {{ $labels.depth }} blocks."
```

## 3. Split Investigation Runbook

### Step 1 — Confirm the Split

```bash
# Compare chain tip with known reference nodes
REF_NODE="https://seed.Udaya.net"
LOCAL_TIP=$(curl -sf http://localhost:8332 -d '{"jsonrpc":"2.0","id":1,"method":"getbestblockhash","params":[]}' | jq -r .result)
REF_TIP=$(curl -sf "$REF_NODE" -d '{"jsonrpc":"2.0","id":1,"method":"getbestblockhash","params":[]}' | jq -r .result)

if [ "$LOCAL_TIP" != "$REF_TIP" ]; then
  echo "SPLIT DETECTED: Local tip $LOCAL_TIP != Reference tip $REF_TIP"
fi

# Find last common ancestor
LOCAL_HEIGHT=$(curl -sf http://localhost:8332 -d '{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}' | jq .result)
REF_HEIGHT=$(curl -sf "$REF_NODE" -d '{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}' | jq .result)

echo "Local height: $LOCAL_HEIGHT, Reference height: $REF_HEIGHT"
```

### Step 2 — Check Peer Versions

```bash
# List connected peers and their software versions
curl -sf http://localhost:8332 \
  -d '{"jsonrpc":"2.0","id":1,"method":"getpeerinfo","params":[]}' | \
  jq '.result[] | {addr, subver, height}'
```

Look for peers running a different `subver` (e.g. `Udaya:1.0.0` vs `Udaya:0.9.0`).

### Step 3 — Validate Divergent Blocks

```bash
# Get the block at split height from both chains
SPLIT_HEIGHT=$((LOCAL_HEIGHT - 1))  # work backwards
LOCAL_BLOCK=$(curl -sf http://localhost:8332 -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getblock\",\"params\":[\"$LOCAL_TIP\"]}")
REF_BLOCK=$(curl -sf "$REF_NODE" -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getblock\",\"params\":[\"$REF_TIP\"]}")

# Compare block headers
echo "Local version: $(echo $LOCAL_BLOCK | jq .result.version)"
echo "Ref version:   $(echo $REF_BLOCK | jq .result.version)"
echo "Local bits:    $(echo $LOCAL_BLOCK | jq .result.bits)"
echo "Ref bits:      $(echo $REF_BLOCK | jq .result.bits)"
```

### Step 4 — Escalation

| Finding | Action | Escalate To |
|---------|--------|------------|
| Different software version | Coordinate upgrade | Dev Lead |
| Invalid block (consensus violation) | Deploy hotfix | Full team |
| Network partition (latency) | Check DNS seeds, add peers | Infra |
| Malicious fork (51 %) | Pause exchanges, alert community | Emergency call |

## 4. Automated Split Resolution

### 4.1 Re-org Handling

When a split resolves naturally (one chain becomes longer), the node must:

1. Identify the fork point (common ancestor)
2. Disconnect blocks from the shorter chain
3. Connect blocks from the longer chain
4. Emit `udaya_reorgs_total` counter increment
5. Log each disconnected/connected block

### 4.2 Manual Intervention

If split persists > 1 hour and > 50 % of seed nodes confirm the same tip:

```bash
# Force node to follow the majority chain
MAJORITY_TIP=$(curl -sf "https://seed.Udaya.net" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getbestblockhash","params":[]}' | jq -r .result)

# Invalidate the local divergent chain
curl -sf http://localhost:8332 \
  -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"invalidateblock\",\"params\":[\"$MAJORITY_TIP\"]}"

# Reconsider the majority chain
curl -sf http://localhost:8332 \
  -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"reconsiderblock\",\"params\":[\"$MAJORITY_TIP\"]}"
```

## 5. Grafana Dashboard: Chain Split Monitor

| Panel | Query | Visual |
|-------|-------|--------|
| Active Splits | `udaya_chain_splits_active` | Stat (red if > 0) |
| Split Depth | `udaya_chain_split_depth` | Gauge |
| Peers on Other Chain | `udaya_peers_on_other_chain` | Gauge |
| Split Events (7d) | `rate(udaya_chain_splits_detected_total[7d])` | Timeseries |
| Reorg Events (7d) | `rate(udaya_reorgs_total[7d])` | Timeseries |

## 6. Testing Split Detection

```bash
# Create intentional split by running two nodes with different genesis
# (test environment only — never on mainnet)

# Node A — normal
udayad --datadir /tmp/node_a --port 19798 --rpcport 18332 start

# Node B — connect to A, then isolate
udayad --datadir /tmp/node_b --port 19799 --rpcport 18333 --connect 127.0.0.1:19798 start

# Stop B's connection and mine a different block
# Then verify split alerts fire
curl -sf http://localhost:18332/metrics | grep udaya_chain_splits_detected_total
```

## 7. References

- [90-Day Execution Plan](./90-day-execution-plan.md)
- [Incident Response Playbook](./incident-response.md)
- [Alerting Rules](../scripts/monitoring/alerts/udaya_alerts.yml)