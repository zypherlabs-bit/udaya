# Udaya Reorganization Handling Guide

## Overview
Chain reorganizations (reorgs) occur when a competing chain tip accumulates more proof-of-work. Udaya limits reorg depth to 6 blocks.

## Reorg Detection
```bash
# Monitor for reorgs via RPC
curl -X POST http://localhost:8332 \
  -d '{"jsonrpc":"2.0","id":1,"method":"getchaintips","params":[]}'

# Alert when reorg detected
udayad alerts --monitor-reorgs
```

## Exchange Reorg Handling
### Deposit Confirmations
| Amount | Min Confirmations | Action |
|--------|------------------|--------|
| < 1 UDYA | 1 | Instant credit |
| 1-100 UDYA | 6 | Standard wait |
| 100-10,000 UDYA | 12 | Extended wait |
| > 10,000 UDYA | 100 | Manual review |

### Reorg Response Procedure
1. **Detection**: Monitoring system detects >2 block reorg
2. **Assessment**: Identify affected transactions
3. **Rollback**: Reverse any unconfirmed credits
4. **Re-verify**: Confirm new chain tip is valid
5. **Re-credit**: Credit deposits on canonical chain
6. **Report**: Log incident and notify operations

## Reorg Risk Mitigation
- Maintain connections to diverse peers
- Monitor multiple block explorers for consensus
- Run multiple nodes behind load balancer
- Implement checkpoint verification at 100 blocks