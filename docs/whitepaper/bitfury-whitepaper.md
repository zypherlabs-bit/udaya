# Udaya: A Next-Generation Proof-of-Work Cryptocurrency

## Abstract

Udaya (UDYA) is a decentralized, permissionless, peer-to-peer cryptocurrency that extends the foundational principles of Bitcoin while incorporating modern cryptographic primitives, enhanced security models, and scalable infrastructure. Udaya utilizes a SHA-256d Proof-of-Work (PoW) consensus mechanism with dynamic difficulty adjustment, support for SegWit, Taproot, and Schnorr signatures, and a robust Layer-2 scaling roadmap. The system is built entirely in Rust for memory safety and performance, with modular architecture supporting enterprise-grade deployments.

---

## 1. Introduction

### 1.1 Motivation

Since the introduction of Bitcoin in 2008, the cryptocurrency landscape has evolved significantly. While Bitcoin remains the gold standard for decentralized value transfer, there is room for a next-generation chain that:

1. Maintains Bitcoin's core security model (PoW, UTXO, full nodes)
2. Integrates modern cryptography (Schnorr, Taproot, post-quantum readiness)
3. Provides enterprise-grade tooling and APIs
4. Enables real-world merchant adoption through Layer-2 scaling
5. Achieves high performance through optimized implementations

Udaya is built to fill this gap — a production-grade blockchain that inherits Bitcoin's battle-tested architecture while advancing into the future.

### 1.2 Design Principles

- **Security First**: Every component is designed with adversarial thinking.
- **Decentralization**: Anyone can run a full node; no privileged participants.
- **Memory Safety**: Rust eliminates entire classes of memory corruption vulnerabilities.
- **Usability**: Built-in wallet, CLI, RPC, and Docker support for easy operation.
- **Scalability**: Layer-2 ready with Lightning-style payment channels.
- **Future-Proof**: Taproot, Schnorr, and post-quantum cryptography preparation.

---

## 2. Consensus Mechanism

### 2.1 Proof-of-Work

Udaya uses **SHA-256d Proof-of-Work**, identical to Bitcoin. Miners search for a nonce such that:

```
SHA256(SHA256(block_header)) <= target
```

The **target** is derived from the `bits` field in the block header, encoded in compact form (32-bit floating-point representation).

### 2.2 Difficulty Adjustment

Difficulty is adjusted every **2,016 blocks** (~2 weeks) using the Bitcoin algorithm:

```
new_target = previous_target * actual_timespan / target_timespan
```

- `target_timespan` = 2,016 × 600 = 1,209,600 seconds (2 weeks)
- `actual_timespan` is clamped to [target/4, target×4]

This ensures the block interval stays close to 10 minutes regardless of hashrate fluctuations.

### 2.3 Block Reward Halving

The block reward follows a **discrete halving schedule** every 210,000 blocks (~4 years):

```
block_reward(height) = 50 * 10^8 >> (height / 210000)
```

Total supply is capped at **21 million UDYA**, matching Bitcoin's monetary policy.

| Halving | Block Height | Reward (UDYA) |
|---------|-------------|---------------|
| 0       | 0           | 50            |
| 1       | 210,000     | 25            |
| 2       | 420,000     | 12.5          |
| ...     | ...         | ...           |
| 63      | 13,230,000  | ~0.000000005  |

### 2.4 Anti-Selfish Mining Protection

Udaya implements detection mechanisms for selfish mining attacks:

1. **Timestamp validation**: Block timestamps must be greater than the median of the last 11 blocks.
2. **Orphan detection**: Rapid block propagation patterns are monitored.
3. **Stale block tracking**: Nodes track mining pools that produce excessive stale blocks.

### 2.5 Chain Finality Checkpoints

Checkpoints provide protection against deep chain reorganizations:

- Hard-coded checkpoints at known-valid block heights.
- Nodes will not accept a reorganization that changes checkpointed blocks.
- Checkpoints are updated with each major software release.
- Maximum reorg depth is limited to 6 blocks by default.

---

## 3. Transaction Model

### 3.1 UTXO Model

Udaya uses the **Unspent Transaction Output (UTXO)** model:

- Each transaction consumes UTXOs as inputs and creates new UTXOs as outputs.
- An input references a previous output via `(txid, vout)` — an OutPoint.
- A user's balance is the sum of all UTXOs they can spend.

### 3.2 Transaction Structure

```
Transaction {
    version:      i32       // Transaction version (currently 2)
    inputs:       [TxIn]   // List of inputs
    outputs:      [TxOut]  // List of outputs
    lock_time:    u32      // Earliest time/height when tx can be mined
}
```

Each **TxIn** contains:
- `previous_output`: OutPoint (txid + vout)
- `script_sig`: Unlocking script (signature)
- `sequence`: Relative locktime field
- `witness`: SegWit witness data

Each **TxOut** contains:
- `value`: Amount in satoshis (1 UDYA = 10^8 satoshis)
- `script_pubkey`: Locking script (recipient conditions)

### 3.3 Transaction Validation

Transactions are validated in two phases:

**Stateless validation:**
- Well-formed structure (non-empty inputs/outputs)
- No duplicate inputs
- Output values within range
- Transaction size ≤ max block size

**Contextual validation:**
- All referenced UTXOs exist and are unspent
- Input values ≥ output values (no inflation)
- Coinbase maturity (100 confirmations)
- Locktime/sequence requirements met
- Valid signatures

### 3.4 Script System

Udaya supports Bitcoin-compatible script opcodes:

- **P2PKH**: Pay to Public Key Hash
- **P2SH**: Pay to Script Hash
- **P2WPKH**: Pay to Witness Public Key Hash (SegWit)
- **P2TR**: Pay to Taproot (key path and script path)
- **Multisig**: M-of-N signature schemes
- **OP_RETURN**: Data outputs (up to 220 bytes)

### 3.5 Fee Market

Transaction fees operate on a market basis:

```
fee = vsize × fee_rate
```

- `vsize` = weight-adjusted size (base size × 3 + total size) / 4
- Miners select highest `fee_rate` transactions first (fee rate = fee / vsize)
- Minimum relay fee: 1 sat/vbyte

---

## 4. Network Architecture

### 4.1 Node Types

| Node Type | Full Blockchain | Validates Blocks | Serves Peers | Light Clients |
|-----------|----------------|------------------|--------------|---------------|
| Full Node | ✅ | ✅ | ✅ | ✅ |
| Light Node | ❌ (headers) | ✅ | ❌ | ✅ |
| Archive Node | ✅ (full index) | ✅ | ✅ | ✅ |
| Miner Node | ✅ | ✅ | Optional | ✅ |
| Bootstrap | ✅ | Partial | ✅ | ❌ |

### 4.2 P2P Protocol

The P2P layer uses a TCP-based protocol with Bitcoin-compatible message structure:

- **Port**: 9798 (mainnet), 19798 (testnet)
- **Magic**: `0xBF591AE7`
- **Message types**: version, verack, ping/pong, getaddr, addr, inv, getdata, block, tx, headers

### 4.3 Peer Discovery & Management

- **DNS seeds**: Hard-coded seed nodes for initial discovery
- **Address relay**: addr messages propagate known peers
- **Peer scoring**: Each peer has a score; misbehavior increases ban score
- **Ban threshold**: Peers exceeding ban threshold are disconnected and banned
- **Connection management**: Target 8 outbound, up to 125 total connections

### 4.4 Block Propagation

- **Headers-first sync**: Download headers, then request blocks
- **Compact blocks**: BIP-152 compact block relay
- **Thin blocks**: Relay transaction short IDs instead of full transactions
- **Parallel downloads**: Request blocks from multiple peers

---

## 5. Security Architecture

### 5.1 Network Security

| Threat | Mitigation |
|--------|-----------|
| DDoS | Rate limiting, connection limits, ban scoring |
| Eclipse attack | Diverse peer selection, random connections |
| Sybil attack | Peer scoring, trusted seeds |
| Replay attack | Strong fork detection, distinct network magic |
| Routing attacks | Multiple connections, redundant paths |

### 5.2 Blockchain Security

| Threat | Mitigation |
|--------|-----------|
| Double spend | Full transaction validation, UTXO model |
| 51% attack | Checkpoints, node diversity |
| Selfish mining | Timestamp verification, orphan detection |
| Long-range attack | Checkpoints, finality rules |
| Invalid blocks | Full validation, merkle proofs |

### 5.3 Cryptographic Primitives

- **Hashing**: SHA-256d (double SHA-256)
- **Signatures**: ECDSA (secp256k1), Schnorr
- **Address hashing**: RIPEMD-160(SHA-256(pubkey))
- **Key generation**: RFC-6979 deterministic ECDSA
- **Entropy**: Secure random number generation

### 5.4 Memory Safety

Udaya is written in **Rust**, providing:

- No null pointer dereferences
- No buffer overflows
- No use-after-free bugs
- Thread safety guaranteed at compile time
- Bounds-checked array access

---

## 6. Storage Design

### 6.1 Database Engine

Udaya uses **RocksDB** as its primary storage engine:

- Column families for organized data storage
- LZ4 compression for reduced disk usage
- Configurable cache sizes (default 512MB)
- Efficient range queries for chain iteration

### 6.2 Data Layout

| Column Family | Content | Key | Value |
|--------------|---------|-----|-------|
| blocks | Full blocks | BlockHash | Block bytes |
| block_hashes | Height → Hash | Height (BE) | BlockHash |
| block_headers | Block headers | BlockHash | Header bytes |
| transactions | Transactions | Txid | Transaction |
| utxo_set | Unspent outputs | OutPoint | UTXO entry |
| chain_state | Chain metadata | String key | Bytes |

### 6.3 Pruning

Optional block pruning reduces storage requirements:

- Pruned nodes delete block data after validation
- UTXO set, headers, and chain state are preserved
- Target: ~10GB for fully pruned node

---

## 7. Mempool Design

### 7.1 Transaction Pool

The mempool manages unconfirmed transactions:

- Maximum 50,000 transactions by default
- Fee-based eviction (lowest fee rate first)
- 72-hour transaction expiry
- Orphan pool for transactions with missing inputs (up to 10,000)
- Ancestor/descendant tracking for CPFP (Child Pays For Parent)

### 7.2 Fee Estimation

The fee estimator uses historical block data:

- Tracks fee rates of included transactions
- Provides estimates for 2, 4, 6, 12, and 24-block targets
- Dynamic adjustment based on mempool pressure

### 7.3 Anti-Spam Protections

- Minimum transaction relay fee
- Transaction size limits
- Sigops limits per transaction and block
- OP_RETURN data limits (220 bytes)
- Standard transaction checks (not standard = not relayed)

---

## 8. Wallet Architecture

### 8.1 Key Management

- Hierarchical Deterministic (HD) wallet (BIP-32)
- Seed phrase backup (BIP-39)
- Secure key derivation with PBKDF2
- Support for multiple address types

### 8.2 Address Types

| Address Type | Prefix | Description |
|-------------|--------|-------------|
| P2PKH (Legacy) | 1 | Pay to Public Key Hash |
| P2SH | 3 | Pay to Script Hash |
| Bech32 (SegWit) | bc1 | Native SegWit |
| Bech32m (Taproot) | bc1p | Taproot |

### 8.3 Multi-Signature

- M-of-N multi-signature wallets
- Support for 1-of-2, 2-of-2, 2-of-3, 3-of-5 configurations
- P2SH wrapped multisig
- Future: Taproot multisig with key aggregation

### 8.4 Security Features

- Encrypted wallet storage (AES-256-GCM)
- Secure memory zeroing (Zeroize)
- Anti-brute-force key derivation
- Hardware wallet integration (future)

---

## 9. Tokenomics

### 9.1 Supply Schedule

- **Total supply**: 21,000,000 UDYA
- **Smallest unit**: 1 satoshi = 0.00000001 UDYA
- **Halving event**: Every 210,000 blocks (~4 years)
- **Final block**: ~2140 (last block with subsidy)

### 9.2 Block Reward Distribution

```
Year 1-4:    50 UDYA per block → 3,285,000 UDYA (~15.6% of total)
Year 4-8:    25 UDYA per block → 1,642,500 UDYA
Year 8-12:   12.5 UDYA per block → 821,250 UDYA
...continues halving until subsidy reaches 0
```

### 9.3 Fee Market

As block rewards decrease, transaction fees become the primary miner incentive:

- Estimated fee market equilibrium at block 6,930,000 (~2115)
- Fee-only blocks provide sustainable security budget

### 9.4 Economic Incentives

| Participant | Incentive |
|------------|-----------|
| Miners | Block rewards + transaction fees |
| Full nodes | Security, sovereignty, no financial incentive |
| Developers | Ecosystem growth, grants (future) |
| Merchants | Low fees, no chargebacks |

---

## 10. Governance

### 10.1 Off-Chain Governance

Udaya uses **Bitcoin-style off-chain governance**:

- **Udaya Improvement Proposals (BFIPs)**: Standardized proposals
- **Miner signaling**: Version bits for upgrade signaling
- **Node activation**: Users upgrade their software to adopt changes
- **Community consensus**: Discussions on forums and repositories

### 10.2 Upgrade Mechanism

- **Soft forks**: Backward-compatible upgrades (miner-activated)
- **Hard forks**: Non-backward-compatible upgrades (user-activated)
- **Activation**: BIP-9 style version bits with threshold signaling

### 10.3 Future On-Chain Governance

Research is underway for on-chain governance mechanisms:

- Staked voting on protocol parameters
- Decentralized treasury management
- BFIP voting and ratification

---

## 11. Layer-2 Scaling

### 11.1 Payment Channels

Udaya's roadmap includes Lightning-style payment channels:

- Bi-directional payment channels with HTLCs (Hashed TimeLock Contracts)
- Multi-hop routing for payments across channels
- Watchtower support for offline monitoring
- Atomic swaps for cross-chain exchange

### 11.2 Sidechains

Future research areas:

- Federated sidechains for specialized use cases
- Drivechain-style two-way peg
- Compatibility with Bitcoin sidechains

---

## 12. Deployment

### 12.1 Hardware Requirements

| Node Type | CPU | RAM | Storage | Bandwidth |
|-----------|-----|-----|---------|-----------|
| Full Node | 2+ cores | 4GB+ | 100GB+ SSD | 100+ GB/month |
| Miner Node | 4+ cores | 8GB+ | 100GB+ SSD | 500+ GB/month |
| Archive Node | 4+ cores | 16GB+ | 1TB+ SSD | 1+ TB/month |

### 12.2 Containerized Deployment

Udaya supports Docker and Kubernetes:

```yaml
# Docker
docker run -d --name Udaya-node \
  -p 9798:9798 -p 8332:8332 \
  -v Udaya-data:/data/Udaya \
  Udaya/node:latest

# Docker Compose
docker-compose -f deployments/docker/docker-compose.yml up -d
```

### 12.3 Monitoring

- Prometheus metrics endpoint
- Grafana dashboards for node health
- Structured JSON logging
- Health check endpoint

---

## 13. Development Roadmap

### Phase 1: Blockchain Core (Current)
- ✅ SHA-256d Proof-of-Work consensus
- ✅ UTXO transaction model
- ✅ RocksDB persistent storage
- ✅ P2P networking layer
- ✅ Basic wallet functionality
- ✅ CLI node interface
- ✅ Mempool with fee estimation

### Phase 2: Mining & Infrastructure (In Progress)
- 🔄 Mining pool support (Stratum protocol)
- 🔄 GPU/CPU miner implementation
- 🔄 Block explorer (REST API + frontend)
- 🔄 RPC API server
- 🔄 WebSocket event streaming
- 🔄 Enterprise monitoring (Prometheus/Grafana)

### Phase 3: Ecosystem & Usability
- ⏳ Lightning-style payment channels
- ⏳ Mobile wallet applications
- ⏳ Merchant payment gateway
- ⏳ Browser wallet extension
- ⏳ SDKs for Python, JavaScript, Go, Java

### Phase 4: Global Scaling
- ⏳ Exchange integration toolkit
- ⏳ DAO governance activation
- ⏳ Sidechain compatibility
- ⏳ Post-quantum cryptography migration
- ⏳ Global node deployment network

---

## 14. Conclusion

Udaya represents the next evolution of Proof-of-Work cryptocurrencies — combining the security and decentralization of Bitcoin with modern engineering practices, cryptographic advances, and enterprise-ready infrastructure. Built in Rust with a modular architecture, Udaya is positioned for global adoption as a store of value, medium of exchange, and platform for decentralized finance.

The project is fully open-source, community-driven, and committed to the principles of decentralization, security, and usability that have made Bitcoin the most secure decentralized network in existence.

---

## References

1. Nakamoto, S. (2008). Bitcoin: A Peer-to-Peer Electronic Cash System
2. Back, A. (2002). Hashcash - A Denial of Service Counter-Measure
3. Maxwell, G. (2015). Deterministic Wallets (BIP-32)
4. Wuille, P. (2017). Segregated Witness (BIP-141)
5. Maxwell, G. et al. (2019). Taproot (BIP-340, BIP-341, BIP-342)
6. Poon, J. & Dryja, T. (2016). The Bitcoin Lightning Network
7. Ruffing, T. et al. (2014). CoinShuffle: Practical Decentralized Coin Mixing

---

*Udaya Whitepaper v1.0 — May 2026*

*"In a world of centralized finance, decentralized money is freedom."*