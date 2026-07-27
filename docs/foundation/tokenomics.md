# Udaya (UDYA) Tokenomics Framework

## Maximum Supply: 21,000,000 UDYA

The total supply is permanently capped at 21,000,000 UDYA, matching Bitcoin's monetary policy to ensure provable scarcity.

## Allocation Structure

| Category | Amount (UDYA) | % of Supply | Lockup/Vesting |
|----------|---------------|-------------|----------------|
| Mining Rewards | 20,500,000 | 97.62% | Distributed over ~120 years via halving schedule |
| Founder Allocation | 500,000 | 2.38% | 6-year linear vesting with 1-year cliff |
| **Total** | **21,000,000** | **100%** | |

## Founder Allocation: 500,000 UDYA

### Requirements

1. **Publicly Disclosed** ✅ - This document serves as public disclosure
2. **Transparent Wallet** ✅ - Founder wallet address: `[To be published at mainnet launch]`
3. **6-Year Vesting Schedule**:
   - **1-year cliff**: No tokens released before year 1
   - **Linear vesting**: 83,333 UDYA released per year from year 1-6
   - **Full vesting**: Complete after 6 years
4. **Vesting Contract**: Enforced via governance lock or immutable smart contract
5. **Explorer Visibility**: All founder transactions will be visible on the public explorer

### Vesting Schedule

| Year | Tokens Released (UDYA) | Cumulative (UDYA) |
|------|----------------------|-------------------|
| 0 | 0 | 0 |
| 1 | 83,333 | 83,333 |
| 2 | 83,333 | 166,666 |
| 3 | 83,333 | 250,000 |
| 4 | 83,333 | 333,333 |
| 5 | 83,334 | 416,667 |
| 6 | 83,333 | 500,000 |

## Mining Rewards: 20,500,000 UDYA

### Block Reward Schedule

| Halving Epoch | Blocks | Reward per Block (UDYA) | Total UDYA Mined |
|---------------|--------|------------------------|-----------------|
| 0 | 0 - 209,999 | 50.0 | 10,500,000 |
| 1 | 210,000 - 419,999 | 25.0 | 5,250,000 |
| 2 | 420,000 - 629,999 | 12.5 | 2,625,000 |
| 3 | 630,000 - 839,999 | 6.25 | 1,312,500 |
| 4 | 840,000 - 1,049,999 | 3.125 | 656,250 |
| 5 | 1,050,000 - 1,259,999 | 1.5625 | 328,125 |
| 6 | 1,260,000 - 1,469,999 | 0.78125 | 164,063 |
| ... | ... | ... | ... |
| 32+ | ~Year 2140 | < 0.00000001 | ~0 UDYA |

### Emission Rate

- **Year 1**: ~2,628,000 UDYA (12.5% of total supply)
- **Year 2**: ~2,628,000 UDYA 
- **Year 4 (halving)**: ~1,314,000 UDYA
- **After 50 years**: < 0.1% annual inflation
- **Final block**: Approximately year 2140

## Treasury & Ecosystem Fund

In addition to mining rewards, the network incorporates:

1. **Treasury Reserve**: 0 UDYA pre-mine (no additional pre-mine beyond founder allocation)
2. **Community Fund**: 10% of block rewards allocated via governance voting
3. **Development Grants**: Funded through governance-approved treasury spending proposals
4. **Bug Bounty Program**: Rewards funded from treasury (up to $50,000 USD in UDYA)

## Treasury Sustainability Simulation

### Year 1 Treasury Projection
- **Annual mining issuance**: 2,628,000 UDYA
- **Community fund (10%)**: 262,800 UDYA
- **Treasury balance**: 262,800 UDYA (if no spending)
- **Governance operational costs**: Variable based on proposals

### Year 5 Treasury Projection (post-2 halvings)
- **Annual mining issuance**: 1,314,000 UDYA
- **Community fund (10%)**: 131,400 UDYA
- **Treasury balance**: Accumulated from years 1-5

## Inflation Analysis

| Year | Circulating Supply (UDYA) | Inflation Rate |
|------|--------------------------|----------------|
| 1 | 2,628,000 | - |
| 2 | 5,256,000 | 100% |
| 4 | 10,512,000 | 33% (post-halving) |
| 10 | ~16,000,000 | ~8% |
| 20 | ~19,000,000 | ~2% |
| 50 | ~20,900,000 | <0.1% |

## Market Liquidity Considerations

### Initial Liquidity Requirements
- Target: Market cap of $1M+ USD equivalent at launch
- Liquidity pool: $50,000 - $100,000 USD equivalent initial
- Exchange listing fees: Covered by community fund

### Liquidity Provider Incentives
- Trading fee discounts for early LPs
- Governance voting weight for LP tokens
- Marketing support from treasury

## Governance of Tokenomics
All tokenomics changes require:
1. Community proposal submission (1 UDYA fee)
2. 10% quorum of circulating supply
3. 60% supermajority approval
4. 14-day voting period

This ensures no single entity can alter the monetary policy without broad community consensus.