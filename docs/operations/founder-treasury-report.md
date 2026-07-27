# Udaya Founder Treasury Security Report

## Treasury Overview

**Total Allocation: 500,000 UDYA** (5% of supply at genesis)

### Treasury Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    FOUNDER WALLET                        │
│              (500,000 UDYA Total Allocation)             │
└──────────┬──────────────────────────────┬───────────────┘
           │                              │
           ▼                              ▼
┌─────────────────────┐     ┌─────────────────────────────┐
│   COLD STORAGE      │     │    OPERATIONS WALLET        │
│   475,000 UDYA      │     │    20,000 UDYA              │
│   (95%)             │     │    (4%)                     │
│                     │     │                             │
│   3-of-5 Multisig   │     │   2-of-3 Multisig           │
│   Hardware Wallets  │     │   Daily Operations          │
│   Offline Storage   │     │   Exchange Liquidity        │
└─────────────────────┘     └──┬──────────────────────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │   DAILY WALLET      │
                    │   5,000 UDYA        │
                    │   (1%)              │
                    │                     │
                    │   1-of-2 Multisig   │
                    │   Daily Expenses    │
                    │   Developer Grants  │
                    └─────────────────────┘
```

## Security Controls

### 1. Cold Storage (95% - 475,000 UDYA)

**Configuration: 3-of-5 Multisig**
- 5 authorized signers with geographically diverse locations
- 3 signatures required for any withdrawal
- Hardware wallets (Coldcard/Ledger) for each signer
- Quarterly physical verification of seed backups
- No single point of failure

**Security Controls:**
- PSBT-based multi-device signing workflow
- Each signer verifies transaction on their hardware wallet
- Air-gapped signing when possible
- Time-locked withdrawals (48-hour delay on >10,000 UDYA)
- Quarterly third-party audit

### 2. Operations Wallet (4% - 20,000 UDYA)

**Configuration: 2-of-3 Multisig**
- 3 authorized operations personnel
- 2 signatures required for any withdrawal
- Covers: exchange listings, market making, operational expenses

**Security Controls:**
- PSBT workflow required for all transactions
- Monthly balance reconciliation
- Transaction limit: 5,000 UDYA per day
- Weekly multi-signer review

### 3. Daily Wallet (1% - 5,000 UDYA)

**Configuration: 1-of-2 Multisig**
- 2 authorized daily operators
- 1 signature sufficient for small transactions (<100 UDYA)
- Covers: developer grants, community rewards, operational costs

**Security Controls:**
- Automated top-up from Operations Wallet when below threshold (500 UDYA)
- Daily balance monitoring
- SMS/email alerts on transactions >50 UDYA
- Monthly expense reporting

## PSBT Workflow for Treasury Operations

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  CREATOR     │    │  SIGNER 1    │    │  SIGNER 2    │
│  (Proposal)  │    │ (Verify Tx)  │    │ (Verify Tx)  │
└──────┬───────┘    └──────┬───────┘    └──────┬───────┘
       │                   │                   │
       │   Create PSBT     │                   │
       │──────────────────>│                   │
       │                   │   Add Signature   │
       │                   │──────────────────>│
       │                   │                   │   Add Signature
       │                   │                   │<──────────────
       │                   │                   │
       │   Finalized PSBT  │                   │
       │<──────────────────────────────────────│
       │                   │                   │
       ▼                   ▼                   ▼
    Transaction Broadcast
```

## Founder Security Handbook

### Key Principles

1. **Never share private keys** - Each signer maintains their own key
2. **Verify every transaction** - Always verify address and amount on hardware wallet screen
3. **Use PSBT workflow** - Never use raw private key signing for treasury transactions
4. **Maintain geographic diversity** - Signers in different locations/countries
5. **Regular backups** - BIP39 seed phrases stored in fireproof safes
6. **Quarterly audits** - Independent third-party verification of balances
7. **Emergency procedure** - Predefined signer replacement protocol

### Signer Responsibilities

| Role | Responsibility | Backup Location |
|------|---------------|-----------------|
| Signer A | Primary key holder - US | Bank safe deposit box |
| Signer B | Primary key holder - EU | Home safe |
| Signer C | Primary key holder - AP | Law firm vault |
| Signer D | Backup key holder - NA | Safety deposit box |
| Signer E | Backup key holder - EU | Private vault |

### Emergency Procedures

#### Lost Key Protocol
1. Remaining signers authenticate via video call
2. PSBT created for key rotation transaction
3. 3-of-5 signers approve
4. Funds moved to new multisig set
5. Lost key signer removed from set

#### Security Breach Protocol
1. Emergency key activation (pre-signed PSBT)
2. Funds moved to emergency cold storage
3. All keys rotated
4. Full security audit conducted
5. Signer set reconstituted

### Treasury Monitoring

| Metric | Target | Alert Threshold |
|--------|--------|----------------|
| Cold Storage Balance | 475,000 UDYA | <470,000 UDYA |
| Operations Balance | 20,000 UDYA | <15,000 UDYA |
| Daily Wallet Balance | 5,000 UDYA | <500 UDYA |
| Monthly Withdrawals | Funded by budget | >Budget + 20% |
| Unauthorized Tx | 0 | Any |

## Audit Trail

All treasury transactions must be logged with:
- PSBT workflow ID
- Transaction ID (txid)
- Signers who approved
- Amount and destination
- Purpose of withdrawal
- Timestamp and block height

## Conclusion

The founder treasury is secured using industry best practices:
- 95% in geographically distributed cold storage
- 3-of-5 multisig for maximum security
- PSBT-based multi-device signing workflow
- Hardware wallet compatibility for all signers
- Comprehensive monitoring and alerting
- Clear emergency procedures
- Regular third-party audits
</write_to_file>