# Udaya Infrastructure Defense Guide

## DDoS Protection Architecture
```
Internet → Cloudflare (DDoS mitigation) → Anycast DNS
  → Load Balancer → Node Cluster → RocksDB
```

## Layer 3/4 Protection
- TCP SYN flood protection via kernel tuning
- Connection rate limiting (100 conn/sec/IP)
- Maximum connection pool (10,000 connections)
- BGP blackhole for sustained attacks
- Anycast routing across 5+ global PoPs

## Layer 7 Protection
- RPC rate limiting: 100 req/min (public), 10K req/min (authenticated)
- Request size limits: 1MB max RPC payload
- Concurrent request limits: 50 per IP
- Request validation and sanitization
- IP reputation scoring

## Peer-to-Peer Defense
| Attack | Defense |
|--------|---------|
| Eclipse | Random peer selection, diverse subnets |
| Sybil | Peer scoring, ban threshold (100 score) |
| Addr flood | Rate-limited addr relay, minimum 30s between |
| Header flood | Per-peer header sync limits |

## TLS Configuration
```nginx
ssl_ciphers ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;
ssl_protocols TLSv1.2 TLSv1.3;
ssl_prefer_server_ciphers on;
ssl_ecdh_curve secp384r1;
```

## Monitoring Thresholds
| Metric | Warning | Critical | Action |
|--------|---------|----------|--------|
| Bandwidth | 500 Mbps | 1 Gbps | Rate limit |
| Connections | 5,000 | 8,000 | Reject new |
| Error rate | 5% | 15% | Enable failover |
| Latency | 200ms | 500ms | Route to backup |