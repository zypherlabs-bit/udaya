# Udaya Treasury Architecture

## Overview

The Udaya Treasury is a multi-layered, multi-signature controlled fund management system designed for maximum security, transparency, and operational efficiency. All wallets are publicly verifiable on the Udaya blockchain.

## Architecture Diagram

```
                    ┌─────────────────────────────────────┐
                    │      BLOCK SUBSIDY (2% / block)      │
                    │         Protocol-enforced            │
                    └────────────────┬────────────────────┘
                                     │
                                     ▼
                    ┌─────────────────────────────────────┐
                    │        FOUNDATION TREASURY           │
                    │        5-of-7 Council Multisig       │
                    │        Balance: 145,678 UDYA         │
                    └──┬──────────┬──────────┬─────────────┘
                       │          │          │
          ┌────────────┘          │          └──────────────┐
          ▼                      ▼                         ▼
┌──────────────────┐  ┌──────────────────┐  ┌─────────────────────┐
│  ECOSYSTEM FUND  │  │ COMMUNITY GRANTS │  │    BUG BOUNTY       │
│  4-of-6 Multisig │  │  3-of-5 Multisig │  │   2-of-3 Multisig   │
│  50,000 UDYA     │  │  19,678 UDYA     │  │   10,000 UDYA       │
│  Exchange/Market │  │  Developer SDK   │  │  Security Research  │
└──────────────────┘  └──────────────────┘  └─────────────────────┘
```

## Founder Allocation (Separate from Treasury)

```
┌─────────────────────────────────────────────────────────┐
│                  FOUNDER ALLOCATION                       │
│              500,000 UDYA (2.38% of supply)               │
│              6-year linear vesting, 1-yr cliff            │
└──────────┬───────────────┬────────────────┬───────────────┘
           │               │                │
           ▼               ▼                ▼
┌────────────────────┐ ┌──────────────┐ ┌──────────────┐
│   COLD STORAGE     │ │  OPERATIONS  │ │    DAILY     │
│   475,000 UDYA     │ │  20,000 UDYA │ │  5,000 UDYA  │
│   3-of-5 Multisig  │ │ 2-of-3 Multi │ │ 1-of-2 Multi │
│   Geographically   │ │ PSBT Workfl. │ │ Auto Top-up  │
│   Distributed      │ │              │ │              │
└────────────────────┘ └──────────────┘ └──────────────┘
```

## Wallet Inventory

| Wallet | Address | Balance | Multisig | Custodians |
|--------|---------|---------|----------|------------|
| Foundation Treasury | UDYA1qtreasury... | 145,678 UDYA | 5-of-7 | Foundation Council |
| Ecosystem Fund | UDYA1qecosystem... | 50,000 UDYA | 4-of-6 | Council + Ecosystem Team |
| Community Grants | UDYA1qgrants... | 19,678 UDYA | 3-of-5 | Treasury Committee |
| Bug Bounty | UDYA1qbounty... | 10,000 UDYA | 2-of-3 | Security Team |
| Founder Cold | UDYA1qfoundercold... | 475,000 UDYA | 3-of-5 | 5 Geographically Distributed Signers |
| Founder Ops | UDYA1qfounderops... | 20,000 UDYA | 2-of-3 | Operations Personnel |
| Founder Daily | UDYA1qfounderdaily... | 5,000 UDYA | 1-of-2 | Daily Operators |

## Security Controls by Tier

### Tier 1: Cold Storage
- **Threshold**: 3-of-5 multisig
- **Signer Distribution**: 5 signers across 3 continents
- **Hardware**: Coldcard/Ledger hardware wallets
- **Signing**: PSBT-based, air-gapped when possible
- **Timelock**: 48-hour delay on transactions >10,000 UDYA
- **Audit**: Quarterly third-party verification

### Tier 2: Operational Wallets
- **Threshold**: Varies (2-of-3 to 4-of-6)
- **Controls**: PSBT workflow required
- **Limits**: Transaction caps per day/week
- **Reconciliation**: Monthly balance verification
- **Monitoring**: 24/7 alerting on transactions

### Tier 3: Daily/Active Wallets
- **Threshold**: 1-of-2 multisig
- **Controls**: Automated top-up from parent wallet
- **Limits**: <100 UDYA per transaction
- **Monitoring**: Real-time alerts on any activity

## Treasury Income Streams

| Source | Rate | Annual Estimate | Governance |
|--------|------|----------------|------------|
| Block Subsidy (2%) | 1 UDYA/block | ~52,560 UDYA | Protocol-enforced |
| Donations | Variable | N/A | Voluntary |
| Partnership Fees | Per agreement | Variable | Council-approved |

## Treasury Expenditure Categories

| Category | Allocation | Annual Budget | Approval |
|----------|-----------|---------------|----------|
| Protocol Development | 40% | ~21,024 UDYA | Council vote |
| Security & Audits | 25% | ~13,140 UDYA | Council vote |
| Community Growth | 20% | ~10,512 UDYA | Grants Committee |
| Operations | 10% | ~5,256 UDYA | Council vote |
| Reserve | 5% | ~2,628 UDYA | Emergency only |

## PSBT Transaction Workflow

1. **Proposal**: Treasury Committee creates a spending proposal with rationale
2. **PSBT Creation**: Unsigned PSBT created with destination address and amount
3. **Multi-Signer Approval**: Required signers independently verify on hardware wallets
4. **PSBT Finalization**: Signatures collected and combined into finalized PSBT
5. **Broadcast**: Finalized transaction broadcast to the Udaya network
6. **Verification**: Transaction visible on explorer within seconds
7. **Logging**: Transaction details recorded in public treasury log

## Monitoring & Alerting

All treasury wallets are monitored 24/7 with:
- Real-time balance tracking via Prometheus
- Immediate alerts on any outgoing transaction
- Daily balance summary reports
- Monthly comprehensive reconciliation
- Quarterly third-party audit reports

## Transparency Principles

1. **All wallets are public**: Every treasury and founder wallet address is published
2. **All transactions are visible**: Every movement of funds is on-chain and explorer-verifiable
3. **All approvals are recorded**: Multisig approval signatures are logged
4. **All audits are published**: Quarterly audit reports are publicly available
5. **All governance is on-chain**: Spending decisions are made through transparent governance processes
</write_to_file>
<task_progress>
- [x] Analyze requirements and gather data
- [x] Create founder-allocation.json
- [x] Create treasury-wallets.json
- [x] Create governance-metrics.json  
- [x] Create audit-reports.json
- [x] Create treasury-transactions.json
- [x] Create vesting-schedule.md
- [x] Create treasury-architecture.md
- [ ] Create inventory.md
- [ ] Enhance index.html with comprehensive trust portal
- [ ] Update website/trust/styles.css
- [ ] Update task_progress.md
</task_progress>
</write_to_file>