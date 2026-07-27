# Udaya Public RPC Infrastructure

## Public RPC Endpoints
| Service | URL | Rate Limit | Auth |
|---------|-----|-----------|------|
| Mainnet RPC | https://rpc.Udaya.net | 100 req/min | Optional API key |
| Mainnet WS | wss://ws.Udaya.net | Unlimited | None |
| Testnet RPC | https://testnet-rpc.Udaya.net | 1,000 req/min | None |
| Testnet WS | wss://testnet-ws.Udaya.net | Unlimited | None |

## Infrastructure Architecture
- Global Anycast DNS (Cloudflare)
- Regional load balancers (5 PoPs)
- Auto-scaling node clusters (Kubernetes)
- Redis cache layer for hot data
- PostgreSQL archive for historical queries

## Security
- TLS 1.3 for all connections
- API key authentication (optional)
- IP-based rate limiting
- CORS configured for web applications
- Request size limits (1MB max)

## Monitoring
- Uptime: 99.9% SLA
- Latency P50: <50ms, P95: <200ms
- Health endpoint: /health
- Metrics endpoint: /metrics (Prometheus)