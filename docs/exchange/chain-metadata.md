# Udaya Chain Metadata Specification

## Network Parameters
| Parameter | Mainnet | Testnet | Regtest |
|-----------|---------|---------|---------|
| Ticker | UDYA | tUDYA | rUDYA |
| P2P Port | 9798 | 19798 | 29798 |
| RPC Port | 8332 | 18332 | 28332 |
| Network Magic | 0xBF591AE7 | 0xBF591AE8 | 0xDAF0A1B2 |
| Bech32 HRP | UDYA | tUDYA | rUDYA |
| Genesis Hash | [See manifest] | [See manifest] | [See manifest] |

## Address Prefixes
| Type | Mainnet | Testnet |
|------|---------|---------|
| P2PKH (Legacy) | 1 | m/n |
| P2SH | 3 | 2 |
| Bech32 (SegWit) | UDYA1 | tUDYA1 |
| Bech32m (Taproot) | UDYA1p | tUDYA1p |

## Consensus Rules
- Algorithm: SHA-256d (double SHA-256)
- Block Time: 600 seconds (10 minutes)
- Difficulty Adjustment: Every 2016 blocks (~2 weeks)
- Halving: Every 210,000 blocks (~4 years)
- Max Supply: 21,000,000 UDYA
- Initial Reward: 50 UDYA
- Coinbase Maturity: 100 confirmations
- Max Reorg Depth: 6 blocks

## Transaction Parameters
- Version: 2
- Locktime: Block height or Unix timestamp
- Min Relay Fee: 1 sat/vbyte
- Standard Tx: Yes (OP_RETURN ≤ 220 bytes)
- SegWit: Activated from block 1
- Taproot: Activated from block 1

## BIP Support
| BIP | Description | Status |
|-----|-------------|--------|
| BIP-32 | HD Wallets | ✅ |
| BIP-39 | Mnemonic Seeds | ✅ |
| BIP-44 | Multi-account HD | ✅ |
| BIP-49 | P2WPKH-in-P2SH | ✅ |
| BIP-84 | Native SegWit (bech32) | ✅ |
| BIP-86 | Taproot (bech32m) | ✅ |
| BIP-141 | Segregated Witness | ✅ |
| BIP-340 | Schnorr Signatures | ✅ |
| BIP-341 | Taproot | ✅ |
| BIP-342 | Tapscript | ✅ |