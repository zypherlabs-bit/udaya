# Udaya Protocol Upgrade Procedures

## Overview
Udaya uses a BIP-9 style version bits activation mechanism for protocol upgrades, with community signaling and miner activation thresholds.

## Upgrade Types
### Soft Forks
Backward-compatible upgrades. Old nodes remain on the network but cannot validate new rules.

### Hard Forks
Non-backward-compatible upgrades. All nodes must upgrade within the activation window.

## Activation Process
1. **BFIP Submission** — Detailed specification published on GitHub
2. **Review Period** — 30 days for community feedback
3. **Code Implementation** — Reference implementation merged after review
4. **Testnet Activation** — 2,016 blocks (2 weeks) on testnet
5. **Miner Signaling** — Version bit signaling over 2,016-block window
6. **Lock-in** — 95% threshold met → activation after 2,016 blocks
7. **Enforcement** — New rules enforced from activation height

## Emergency Upgrades
Critical security patches may bypass the standard process with:
- Technical Steering Committee emergency declaration
- 5/7 Council approval
- Minimum 48-hour public notice