# Udaya Node Deployment Playbook

## Infrastructure Requirements
| Node Type | CPU | RAM | Storage | Network |
|-----------|-----|-----|---------|---------|
| Seed Node | 8 cores | 16 GB | 500 GB SSD | 1 Gbps |
| Full Node | 4 cores | 8 GB | 200 GB SSD | 100 Mbps |
| Archive Node | 16 cores | 32 GB | 2 TB SSD | 1 Gbps |
| Mining Node | 4 cores | 8 GB | 100 GB SSD | 100 Mbps |

## Docker Deployment
```bash
# Build image
docker build -f deployments/docker/Dockerfile -t Udaya/node:latest .

# Run node
docker run -d --name Udaya-node \
  -p 9798:9798 -p 8332:8332 \
  -v Udaya-data:/data/Udaya \
  -e RUST_LOG=info \
  Udaya/node:latest

# Verify
docker logs -f Udaya-node
curl -X POST http://localhost:8332 \
  -d '{"jsonrpc":"2.0","id":1,"method":"getblockchaininfo","params":[]}'
```

## Kubernetes Deployment
```bash
# Deploy seed node
kubectl apply -f deployments/k8s/seed-node-deployment.yaml

# Deploy full node
kubectl apply -f deployments/k8s/Udaya-node.yaml

# Check status
kubectl get pods -l app=Udaya-node
kubectl logs -l app=Udaya-node
```

## Configuration
```toml
[network]
listen_port = 9798
max_peers = 125

[storage]
data_dir = "/data/Udaya"
db_cache_size_mb = 4096

[consensus]
network = "mainnet"

[rpc]
enable = true
listen_port = 8332
```

## Monitoring Setup
- Prometheus metrics at `/metrics`
- Health check at `/health`
- Grafana dashboard: `deployments/grafana/dashboards/node-health.json`

## Backup & Recovery
```bash
# Backup chain data
tar -czf Udaya-backup-$(date +%Y%m%d).tar.gz /data/Udaya

# Restore
tar -xzf Udaya-backup-*.tar.gz -C /data/Udaya
docker restart Udaya-node