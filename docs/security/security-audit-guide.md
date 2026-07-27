# Udaya Security Audit Guide

## Overview
Udaya implements continuous security testing through fuzzing, adversarial simulation, and formal verification.

## Running Automated Security Audits
```bash
# Full security audit (100,000 fuzz iterations)
udayad security-audit --fuzz-iterations 100000

# Quick audit (10,000 iterations)
udayad security-audit --fuzz-iterations 10000

# Extended audit (1,000,000 iterations, recommended for releases)
udayad security-audit --fuzz-iterations 1000000
```

## Fuzz Testing
The fuzzing engine tests:
- Transaction deserialization (malformed transactions)
- Block deserialization (corrupted blocks)
- Script execution (edge case scripts)
- Mempool insertion (flood scenarios)
- P2P message handling (malformed packets)
- Address generation (deterministic checks)
- Key derivation (entropy edge cases)

## Adversarial Simulations
The simulator tests resistance against:
| Attack | Detection Method | Defense |
|--------|-----------------|---------|
| 51% Attack | Hashrate monitoring | Checkpoints, node diversity |
| Selfish Mining | Timestamp analysis | Anti-withholding detection |
| Eclipse Attack | Peer diversity checks | Random peer selection |
| Sybil Attack | Connection scoring | Ban threshold enforcement |
| Double Spend | UTXO validation | 6-confirmation finality |
| Mempool Flood | Rate limiting | Per-peer fee thresholds |
| Chain Reorg | Depth monitoring | Max 6-block reorg limit |

## Audit Reports
Reports are generated as JSON and published to the security-audit-report.json file:
```json
{
  "fuzzing": {
    "iterations": 100000,
    "critical_failures": 0,
    "warnings": 0,
    "duration_secs": 45.2
  },
  "adversarial": {
    "attacks_simulated": 12,
    "successful_attacks": 0,
    "score": 98.5
  }
}
```

## Continuous Integration
Security audits run automatically on:
- Every pull request (10K fuzz iterations)
- Every merge to main (100K fuzz iterations)
- Every release candidate (1M fuzz iterations)