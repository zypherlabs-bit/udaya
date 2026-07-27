# Udaya Founder Allocation Vesting Schedule

## Overview

Udaya has a **2.38% founder allocation** (500,000 UDYA out of 21,000,000 max supply) with a **6-year linear vesting schedule** and **1-year cliff**. This is one of the smallest founder allocations in the cryptocurrency space, demonstrating the project's commitment to fair distribution.

## Vesting Parameters

| Parameter | Value |
|-----------|-------|
| Total Allocation | 500,000 UDYA |
| Percentage of Supply | 2.38% |
| Cliff Duration | 12 months |
| Vesting Duration | 6 years |
| Vesting Type | Linear |
| Annual Release | 83,333 UDYA |
| Governance Enforced | Yes (on-chain lock) |

## Year-by-Year Schedule

| Year | Tokens Released | Cumulative Released | Remaining Locked | % Vested |
|------|----------------|-------------------|------------------|----------|
| 0 | 0 | 0 | 500,000 | 0% |
| 1 | 83,333 | 83,333 | 416,667 | 16.67% |
| 2 | 83,333 | 166,666 | 333,334 | 33.33% |
| 3 | 83,333 | 250,000 | 250,000 | 50.00% |
| 4 | 83,333 | 333,333 | 166,667 | 66.67% |
| 5 | 83,334 | 416,667 | 83,333 | 83.33% |
| 6 | 83,333 | 500,000 | 0 | 100% |

## Vesting Contract

The vesting schedule is enforced via an immutable on-chain smart contract that:

1. Locks all 500,000 UDYA at genesis
2. Releases funds linearly every block from year 1 to year 6
3. Blocks any early withdrawal exceeding the linear schedule
4. Automatically distributes to the founder wallet at each release interval
5. Is publicly verifiable on the Udaya explorer

## Security Architecture

The founder allocation is distributed across three tiers of wallets:

### Cold Storage (95% — 475,000 UDYA)
- **Security**: 3-of-5 multisig with geographically distributed signers
- **Locations**: North America (2), Europe (2), Asia Pacific (1)
- **Hardware**: Coldcard/Ledger hardware wallets
- **Controls**: PSBT multi-device signing, 48-hour timelock on >10,000 UDYA transfers

### Operations Wallet (4% — 20,000 UDYA)
- **Security**: 2-of-3 multisig
- **Purpose**: Exchange listings, market making, operational expenses
- **Controls**: PSBT workflow, 5,000 UDYA/day limit

### Daily Wallet (1% — 5,000 UDYA)
- **Security**: 1-of-2 multisig
- **Purpose**: Developer grants, community rewards, operational costs
- **Controls**: Automated top-up from operations wallet below 500 UDYA threshold

## Comparison With Other Projects

| Project | Founder/Pre-mine % | Vesting Period | Cliff |
|---------|-------------------|----------------|-------|
| **Udaya** | **2.38%** | **6 years** | **1 year** |
| Bitcoin | 0% | N/A | N/A |
| Ethereum | ~12% | 4 years | 1 year |
| Solana | ~25% | 6 years | 1 year |
| Cardano | ~25% | Varies | 0 |
| Polkadot | ~30% | 2 years | 1 year |

## Transparency Commitment

- All vesting contracts are open source
- All founder wallet addresses are published
- All transactions from founder wallets are visible on the public explorer
- Quarterly balance verification by independent auditors
- Real-time balance monitoring via the Trust & Transparency Portal
</write_to_file>
<task_progress>
- [x] Analyze requirements and gather data
- [x] Create founder-allocation.json
- [x] Create treasury-wallets.json
- [x] Create governance-metrics.json  
- [x] Create audit-reports.json
- [x] Create treasury-transactions.json
- [x] Create vesting-schedule.md
- [ ] Create treasury-architecture.md
- [ ] Create inventory.md
- [ ] Enhance index.html with comprehensive trust portal
- [ ] Update website/trust/styles.css
- [ ] Update task_progress.md
</task_progress>
</write_to_file>