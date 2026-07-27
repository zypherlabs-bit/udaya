# Udaya Node Growth Metrics

> **Version:** 1.0.0 | **Phase:** 12 — Post-Launch Survivability
> **Owner:** Core Dev Team

## 1. Purpose

Track the number, distribution, and health of Udaya nodes on the network. Node growth is
a primary indicator of decentralisation and network resilience.

## 2. Data Sources

| Source | Data | Update Frequency |
|--------|------|-----------------|
| Seed node peer lists | Node IPs, versions, heights | Real-time |
| DNS seeds | Node counts by DNS zone | Every DNS query |
| Public RPC nodes | Self-reported status | Every 60 s |
| Network crawler | Active scan of all reachable nodes | Every 6 h |

## 3. Core Node Metrics

### 3.1 Prometheus Metrics

| Metric | Prometheus Name | Type | Labels | Description |
|--------|-----------------|------|--------|-------------|
| Total reachable nodes | `udaya_network_node_count` | Gauge | — | Estimated total active nodes |
| Connected peers | `udaya_peer_count` | Gauge | `instance` | Peers connected to this node |
| Nodes by version | `udaya_node_versions` | Gauge | `version` | Count per software version |
| Nodes by country | `udaya_node_country` | Gauge | `country` | Geo-distribution |
| Nodes by ISP | `udaya_node_isp` | Gauge | `isp` | ISP distribution |
| Inbound connections | `udaya_inbound_connections` | Gauge | `instance` | Peers connecting to this node |
| Outbound connections | `udaya_outbound_connections` | Gauge | `instance` | Peers this node connects to |
| New nodes / day | `rate(udaya_network_node_count[24h])` | Derived | — | Node growth rate |

### 3.2 Derived Recording Rules

```yaml
groups:
  - name: udaya_node_metrics
    interval: 5m
    rules:
      - record: Udaya:node_growth_daily
        expr: rate(udaya_network_node_count[24h])

      - record: Udaya:node_growth_weekly
        expr: rate(udaya_network_node_count[7d])

      - record: Udaya:node_version_diversity
        expr: |
          count by (version) (udaya_node_versions)
          / ignoring(version) sum(udaya_node_versions)
          * 100

      - record: Udaya:node_geo_diversity
        expr: count by (country) (udaya_node_country)
```

## 4. Grafana Dashboard: Node Growth

| Panel | Query | Visual |
|-------|-------|--------|
| Total Nodes | `udaya_network_node_count` | Stat + sparkline |
| Node Growth (7d) | `Udaya:node_growth_weekly` | Stat (increase/decrease) |
| Daily New Nodes | `Udaya:node_growth_daily` | Bar chart |
| Version Distribution | `Udaya:node_version_diversity` | Pie chart |
| Geographic Distribution | `Udaya:node_geo_diversity` | World map (Geomap) |
| ISP Distribution | `sum by (isp) (udaya_node_isp)` | Horizontal bar |
| Peer Count per Node | `avg(udaya_peer_count)` | Gauge |
| Inbound vs Outbound | `avg(udaya_inbound_connections)` vs `avg(udaya_outbound_connections)` | Timeseries |
| Reachable Nodes (7d) | `udaya_network_node_count` | Timeseries with weekly comparison |

## 5. Network Crawler

Deploy a dedicated crawler node that periodically scans the network:

```rust
// Pseudocode for network crawler
fn crawl_network() -> CrawlResult {
    let seed_nodes = resolve_dns_seeds("seed.Udaya.net");
    let mut discovered: HashSet<SocketAddr> = seed_nodes.into_iter().collect();
    let mut visited: HashSet<SocketAddr> = HashSet::new();
    let mut versions: HashMap<String, u32> = HashMap::new();

    while let Some(addr) = discovered.iter().next() {
        if visited.contains(addr) { continue; }
        if let Ok(peers) = get_addr_peers(*addr) {
            for peer in peers {
                discovered.insert(peer.addr);
            }
            *versions.entry(peer.subver).or_insert(0) += 1;
        }
        visited.insert(*addr);
    }

    CrawlResult {
        total_nodes: visited.len(),
        version_distribution: versions,
        // ...
    }
}
```

### 5.1 Crawler Configuration

```yaml
# crawler-config.yml
crawler:
  interval: 360m  # every 6 hours
  seed_nodes:
    - seed.Udaya.net:9798
    - seed-us-west.Udaya.net:9798
    - seed-us-east.Udaya.net:9798
    - seed-eu.Udaya.net:9798
    - seed-apac.Udaya.net:9798
  max_concurrency: 50
  timeout_seconds: 10
  output_metrics: true
```

## 6. Node Health Categories

| Category | Definition | Target % |
|----------|-----------|----------|
| Healthy | Fully synced, >= 8 peers, responding to RPC | > 90 % |
| Syncing | Catching up, block height > 90 % of tip | < 8 % |
| Stalled | Peer count < 3, height not advancing | < 1 % |
| Unreachable | Not responding to P2P or RPC | < 1 % |

### 6.1 Health Alert

```yaml
- alert: HighPercentageUnreachable
  expr: (udaya_network_node_unreachable / udaya_network_node_count) * 100 > 5
  for: 1h
  labels:
    severity: warning
  annotations:
    summary: "More than 5 % of nodes are unreachable"
    description: "{{ $value | humanize }}% of nodes unreachable. Possible network-wide issue."
```

## 7. Node Growth Targets

| Timeframe | Target Nodes | Notes |
|-----------|-------------|-------|
| Launch week | >= 50 | Bootstrap minimum |
| Month 1 | >= 200 | Requires exchange listings |
| Month 2 | >= 500 | Community + marketing |
| Month 3 | >= 1 000 | SDK release, developer outreach |
| Quarter 2 | >= 5 000 | Ecosystem maturity |

## 8. Correlation Analysis

Correlate node count with other metrics to understand growth drivers:

```sql
-- Example: Grafana annotation query
SELECT
  time,
  udaya_network_node_count,
  Udaya:network_daily_tx_volume,
  udaya_github_stars
FROM metrics
WHERE $__timeFilter()
```

Typical correlations:
- **Exchange listing → +30–50 % node growth** (speculators run nodes)
- **SDK release → +20–30 % node growth** (developers run nodes)
- **Price movement → +10–20 % node growth** (FOMO, independent of utility)
- **Marketing campaign → +5–15 % node growth** (brand awareness)

## 9. Reporting

Node metrics feed into:
- [Weekly Operational Report](./weekly-operational-report-template.md)
- Network status page (status.Udaya.net)
- Udaya website /ecosystem page
- [Network Growth Metrics](./network-growth-metrics.md)

## 10. References

- [Miner Growth Metrics](./miner-growth-metrics.md)
- [Network Growth Metrics](./network-growth-metrics.md)
- [90-Day Execution Plan](./90-day-execution-plan.md)