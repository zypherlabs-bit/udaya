# Udaya Architecture Overview

## Introduction

Udaya is a production-grade, SHA-256d Proof-of-Work Layer-1 blockchain implemented in Rust. This document provides a comprehensive overview of Udaya's architecture, design principles, and technical components.

---

## Table of Contents

1. [High-Level Architecture](#high-level-architecture)
2. [Core Components](#core-components)
3. [Consensus Mechanism](#consensus-mechanism)
4. [Network Architecture](#network-architecture)
5. [Storage Layer](#storage-layer)
6. [Wallet System](#wallet-system)
7. [API Layer](#api-layer)
8. [Security Architecture](#security-architecture)
9. [Performance Considerations](#performance-considerations)
10. [Future Architecture Evolution](#future-architecture-evolution)

---

## High-Level Architecture

```
┌───────────────────────────────────────────────────────────────────────────────┐
│                                Udaya Blockchain                                │
├───────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  ┌─────────────┐    ┌─────────────┐    ┌───────────────────────────────────┐  │
│  │   Clients   │    │   Miners    │    │           Exchanges             │  │
│  └─────────────┘    └─────────────┘    └───────────────────────────────────┘  │
│          │                 │                      │                         │
│          ▼                 ▼                      ▼                         │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                        External Interface Layer                      │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐  ┌─────────┐  │  │
│  │  │ JSON-RPC │  │  REST  │  │ WebSocket│  │  Explorer  │  │  CLI    │  │  │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────────┘  └─────────┘  │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                               │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                        Node Daemon (udayad)                          │  │
│  │                                                                       │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │  │
│  │  │  Core       │  │  Wallet     │  │  Mining     │  │  P2P        │  │  │
│  │  │  Engine     │  │  Engine     │  │  Pool       │  │  Network    │  │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │  │
│  │                                                                       │  │
│  │  ┌───────────────────────────────────────────────────────────────┐  │  │
│  │  │                    Shared Infrastructure                     │  │  │
│  │  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐    │  │  │
│  │  │  │ Storage │  │  Mempool │  │  API     │  │  Security   │    │  │  │
│  │  │  │ (RocksDB)│  │  Mgmt   │  │  Layer   │  │  Framework  │    │  │  │
│  │  │  └─────────┘  └─────────┘  └─────────┘  └─────────────┘    │  │  │
│  │  └───────────────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                               │
└───────────────────────────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. Core Engine

The heart of Udaya, responsible for:

- **Consensus**: SHA-256d Proof-of-Work implementation
- **Blockchain Management**: Block validation, chain organization
- **Transaction Processing**: UTXO management, script validation
- **Genesis**: Network initialization and bootstrap
- **Governance**: On-chain voting and parameter changes

**Key Files**:
- `src/core/src/consensus.rs` - Consensus rules
- `src/core/src/genesis.rs` - Genesis block definition
- `src/core/src/validation.rs` - Transaction validation

### 2. Wallet Engine

Secure wallet implementation with:

- **Key Management**: BIP-32/39/44/49/84/86 HD wallets
- **Address Generation**: Bech32, P2PKH, P2SH, P2WPKH, P2TR
- **Transaction Construction**: PSBT support
- **Security**: Zeroization, hardware wallet integration

**Key Files**:
- `src/wallet/src/lib.rs` - Main wallet logic
- `src/wallet/src/crypto.rs` - Cryptographic operations
- `src/wallet/src/psbt.rs` - PSBT implementation

### 3. Mining Pool

Stratum V2 compatible mining infrastructure:

- **Pool Management**: Miner registration, share tracking
- **Block Template Generation**: GetBlockTemplate (GBT) support
- **Reward Distribution**: Payout calculation and processing
- **Telemetry**: Miner performance monitoring

**Key Files**:
- `src/mining/src/lib.rs` - Mining pool core
- `src/pool-server/src/main.rs` - Pool server

### 4. P2P Network

Decentralized peer-to-peer networking:

- **Discovery**: DNS seeds, peer exchange
- **Protocol**: Custom binary protocol
- **Message Types**: Version, Verack, Inv, GetData, etc.
- **Security**: Flood protection, ban scores

**Key Files**:
- `src/p2p/src/network.rs` - Network protocol
- `src/p2p/src/discovery.rs` - Peer discovery

---

## Consensus Mechanism

### SHA-256d Proof-of-Work

Udaya uses a Bitcoin-compatible SHA-256d consensus algorithm:

```rust
// Simplified consensus flow
fn validate_block(block: &Block, previous_hash: &BlockHash) -> Result<()> {
    // 1. Check block structure
    validate_block_structure(block)?;

    // 2. Verify proof-of-work
    verify_pow(&block.header)?;

    // 3. Validate transactions
    validate_transactions(block.transactions(), previous_hash)?;

    // 4. Check difficulty adjustment
    validate_difficulty(&block.header, previous_hash)?;

    Ok(())
}
```

### Key Consensus Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| Block Time | 600 seconds | Target block interval |
| Difficulty Adjustment | Every 2016 blocks | Retargeting interval |
| Max Block Size | 4MB | Maximum block size |
| Subsidy Halving | Every 210,000 blocks | Block reward halving |
| Genesis Block | Hardcoded | Network foundation |

### Difficulty Adjustment Algorithm

Udaya implements a modified Bitcoin-style difficulty adjustment:

1. **Target Time**: 2016 blocks × 600 seconds = ~14 days
2. **Actual Time**: Time taken to mine last 2016 blocks
3. **Adjustment**: `NewDifficulty = OldDifficulty × (TargetTime / ActualTime)`

---

## Network Architecture

### P2P Protocol

Udaya's network protocol includes:

- **Message Types**:
  - `Version` - Initial handshake
  - `Verack` - Handshake acknowledgment
  - `Inv` - Inventory announcement
  - `GetData` - Data request
  - `Block` - Block transmission
  - `Tx` - Transaction transmission

- **Connection Management**:
  - Maximum 125 outbound connections
  - Unlimited inbound connections
  - Peer scoring and banning

- **Security Features**:
  - Flood protection
  - Rate limiting
  - IP ban scores

### Network Topology

```
[Seed Nodes]
    │
    ▼
[Well-Connected Nodes]
    │
    ▼
[Regular Nodes] ←→ [SPV Clients]
    │
    ▼
[Mining Nodes] ←→ [Mining Pools]
```

---

## Storage Layer

### RocksDB Integration

Udaya uses RocksDB for persistent storage:

- **Column Families**:
  - `blocks` - Block headers and data
  - `tx_index` - Transaction index
  - `utxo_set` - Unspent transaction outputs
  - `chain_state` - Chain metadata

- **Performance Optimizations**:
  - Block cache (1GB default)
  - Write-ahead logging
  - Compression
  - Periodic compaction

### Database Schema

```rust
// Simplified storage structure
struct BlockchainDB {
    rocksdb: RocksDB,
    cf_blocks: ColumnFamily,
    cf_tx_index: ColumnFamily,
    cf_utxo: ColumnFamily,
    cf_chain: ColumnFamily,
}

impl BlockchainDB {
    fn put_block(&self, height: u64, block: &Block) -> Result<()> {
        let height_key = height.to_be_bytes();
        let block_data = bincode::serialize(block)?;
        self.rocksdb.put_cf(&self.cf_blocks, &height_key, &block_data)
    }

    fn get_block(&self, height: u64) -> Result<Option<Block>> {
        let height_key = height.to_be_bytes();
        match self.rocksdb.get_cf(&self.cf_blocks, &height_key)? {
            Some(data) => Ok(Some(bincode::deserialize(&data)?)),
            None => Ok(None),
        }
    }
}
```

---

## Wallet System

### HD Wallet Architecture

Udaya implements hierarchical deterministic wallets:

```
m (Master)
├─ 44' (BIP44 Purpose)
│  └─ 257' (Udaya Coin Type)
│     └─ 0' (Account)
│        ├─ 0 (External Chain)
│        │  ├─ 0 (Address 0)
│        │  ├─ 1 (Address 1)
│        │  └─ ...
│        └─ 1 (Internal Chain)
│           ├─ 0 (Change Address 0)
│           ├─ 1 (Change Address 1)
│           └─ ...
└─ 84' (BIP84 Purpose)
   └─ 257' (Udaya Coin Type)
      └─ 0' (Account)
         └─ 0 (External Chain)
            ├─ 0 (Native SegWit Address 0)
            └─ ...
```

### Address Formats

| Type | Format | Prefix | Script Type |
|------|--------|--------|-------------|
| P2PKH | Base58 | `U` | `OP_DUP OP_HASH160 <pubkeyhash> OP_EQUALVERIFY OP_CHECKSIG` |
| P2SH | Base58 | `3` | `OP_HASH160 <redeemScriptHash> OP_EQUAL` |
| P2WPKH | Bech32 | `udaya` | `0 <pubkeyhash>` |
| P2TR | Bech32m | `udaya` | `1 <pubkey>` |

---

## API Layer

### JSON-RPC API

Udaya provides a comprehensive JSON-RPC interface:

| Category | Methods | Description |
|----------|---------|-------------|
| **Blockchain** | `getblockchaininfo`, `getblockcount`, `getblockhash`, `getblock` | Blockchain queries |
| **Network** | `getnetworkinfo`, `getpeerinfo`, `addnode` | Network management |
| **Wallet** | `getbalance`, `getnewaddress`, `sendtoaddress`, `listunspent` | Wallet operations |
| **Mining** | `getmininginfo`, `getblocktemplate`, `submitblock` | Mining operations |
| **Mempool** | `getmempoolinfo`, `getmempoolentry` | Mempool inspection |
| **Utility** | `ping`, `getinfo`, `stop` | Utility functions |

### REST API

Additional REST endpoints for monitoring:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/healthz` | GET | Liveness check |
| `/readyz` | GET | Readiness check |
| `/health` | GET | Detailed health report |
| `/metrics` | GET | Prometheus metrics |

---

## Security Architecture

### Cryptographic Primitives

| Component | Algorithm | Implementation |
|-----------|-----------|----------------|
| Hashing | SHA-256d | `sha2` crate |
| Signatures | ECDSA (secp256k1) | `secp256k1` crate |
| Key Derivation | HKDF | `hkdf` crate |
| Encryption | AES-256-GCM | `aes-gcm` crate |
| Password Hashing | Argon2 | `argon2` crate |

### Security Features

1. **Memory Safety**: Rust's ownership model prevents memory corruption
2. **Zeroization**: Sensitive data is securely wiped from memory
3. **Input Validation**: Comprehensive validation of all external inputs
4. **Rate Limiting**: Protection against DoS attacks
5. **Fuzzing**: Continuous fuzz testing of critical components
6. **Adversarial Simulation**: Regular security stress testing

---

## Performance Considerations

### Benchmark Results

| Operation | Throughput | Latency |
|-----------|------------|---------|
| Block Validation | 10-20 blocks/sec | 50-100ms |
| Transaction Validation | 1000-2000 tx/sec | 0.5-1ms |
| Signature Verification | 5000-10000 ops/sec | 0.1-0.2ms |
| Database Reads | 5000-10000 ops/sec | 0.1-0.5ms |
| Database Writes | 1000-2000 ops/sec | 0.5-2ms |

### Optimization Techniques

1. **Parallel Processing**: Rayon for parallel transaction validation
2. **Caching**: LRU caches for frequently accessed data
3. **Batch Processing**: Batch database operations
4. **Async I/O**: Tokio for asynchronous networking
5. **Memory Pools**: Reusable memory buffers

---

## Future Architecture Evolution

### Phase 2 Enhancements

- **Lightning Network**: Layer 2 scaling solution
- **Smart Contracts**: Scripting enhancements
- **Cross-Chain Bridges**: Interoperability features
- **Advanced Privacy**: Confidential transactions

### Phase 3 Features

- **Sharding**: Horizontal scalability
- **DPoS Hybrid**: Consensus algorithm evolution
- **Enterprise Features**: Institutional-grade tools
- **DeFi Primitives**: Native financial instruments

---

## Conclusion

Udaya's architecture represents a modern, secure, and performant blockchain implementation that builds upon the proven foundations of Bitcoin while incorporating contemporary advancements in cryptography, networking, and systems design. The modular architecture allows for continuous evolution while maintaining stability and security.

**Last Updated**: 2026-07-27
**Architecture Version**: 1.0.0-alpha.1
**Status**: Phase 3 Public Alpha Testing Preparation