# Udaya Layer-2 Scaling Research

## Overview
Research and design for Layer-2 scaling solutions on the Udaya network, enabling high-throughput, low-latency transactions while maintaining the security guarantees of the base layer.

## 1. Payment Channel Networks

### 1.1 Architecture
Udaya's payment channel network follows the Lightning Network specification with adaptations for the Udaya protocol:
- **HTLC**: Hashed TimeLock Contracts for atomic routing
- **PTLC**: Point TimeLock Contracts (Taproot-enabled) for privacy
- **Multi-path Payments**: Splitting payments across multiple routes

### 1.2 Key Differences from Bitcoin Lightning
| Feature | Bitcoin Lightning | Udaya Lightning |
|---------|-----------------|-------------------|
| Signature | ECDSA | Schnorr (native) |
| Script | Bitcoin Script | Udaya Script |
| Anchors | 2-of-2 multisig | Taproot key-path |
| Routing | Source-based | Source-based + trampoline |

### 1.3 Implementation Status
- [ ] Channel establishment (funding transactions)
- [ ] HTLC/PTLC commitment transactions
- [ ] Routing and pathfinding
- [ ] Watchtower service
- [ ] Mobile wallet integration

## 2. State Channels
Generalized state channels for application-specific state updates without global consensus.

## 3. Sidechains

### 3.1 Federated Sidechains
- Federation of n-of-m validators
- Two-way peg via federation
- Specialized for high-throughput applications

### 3.2 Drivechains
- BIP-300/301 style drivechains
- Miner-managed sidechain withdrawal
- Extensible for custom VM environments

## Research Milestones
| Milestone | Timeline | Deliverable |
|-----------|----------|-------------|
| Channel spec | Q3 2026 | BFIP specification |
| Reference implementation | Q4 2026 | Rust implementation |
| Testnet deployment | Q1 2027 | Public testing |
| Mainnet launch | Q2 2027 | Production release |