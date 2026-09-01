# Udaya Flutter Node Architecture

## Document Control

| Field | Value |
|-------|-------|
| Version | 0.1.0 |
| Status | Draft |
| Date | 2026-08-24 |
| Author | Udaya Architecture Team |

---

## 1. Critical Architecture Rule

**DO NOT rewrite Udaya consensus or other consensus-critical blockchain logic in Dart.**

The existing Rust implementation (`udaya-core`, `udaya-consensus`, `udaya-*`) remains the authoritative blockchain engine. Flutter is the native cross-platform client and node interface layer only.

Any consensus, validation, UTXO management, Merkle tree computation, chainwork calculation, or block validation logic that exists in Rust must continue to execute in Rust. Dart code must never:

- Implement or replicate PoW validation
- Reimplement transaction or script verification
- Maintain an independent copy of blockchain state
- Independently compute difficulty or chainwork
- Perform signature verification outside the Rust engine

---

## 2. Target Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Flutter Application                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│  │Dashboard │ │  Wallet  │ │ Send/Recv│ │Explorer  │ │   Node   │ │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘ │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│  │  Peers   │ │ Mining   │ │Network   │ │ Testnet  │ │ Settings │ │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘ │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Dart Application / API Layer                      │
│  - Strongly typed models (freezed, json_serializable)               │
│  - Riverpod state management                                        │
│  - Input validation, sanitization, error normalization              │
│  - No direct crypto, consensus, or blockchain logic                  │
│  - Structured logging / event streaming to UI                        │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    FFI / JNI Bridge (Stable C ABI)                   │
│  - dart:ffi for direct native binding                               │
│  - Optional JNI/Kotlin shim for Android-specific requirements       │
│  - Stable C-compatible function signatures                          │
│  - No Rust internal types exposed to Dart                            │
│  - Every boundary validates inputs and handles errors safely         │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Udaya Rust Core                               │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐  │
│  │  Consensus  │ │ SHA-256d PoW│ │    UTXO     │ │Transactions │  │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐  │
│  │Block Valid. │ │ Merkle Trees│ │  Chainwork  │ │Fork/Reorg   │  │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐  │
│  │  Mempool    │ │    P2P      │ │   Storage   │ │Wallet Crypto│  │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐                   │
│  │ Node Services│ │   RPC      │ │ Explorer    │ │                   │
│  └─────────────┘ └─────────────┘ └─────────────┘                   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 3. Rust Workspace Inventory

### 3.1 Core Crates

| Crate | Path | Responsibility |
|-------|------|----------------|
| `udaya-core` | `src/core/` | Consensus, types, transactions, scripts, addresses, validation, genesis, security, observability |
| `udaya-storage` | `src/storage/` | RocksDB blockchain database, column families, batch writes |
| `udaya-mempool` | `src/mempool/` | Transaction memory pool, fee indexing, orphan handling, validation |
| `udaya-p2p` | `src/p2p/` | Network protocol, peer management, message serialization, DNS seed discovery |
| `udaya-wallet` | `src/wallet/` | HD wallet (BIP32/39/44/49/84/86), PSBT, key derivation, zeroization, threat model |
| `udaya-mining` | `src/mining/` | Stratum V2 mining pool, share tracking, block template generation |
| `udaya-explorer` | `src/explorer/` | Blockchain explorer data structures, chain statistics, block/transaction summaries |
| `udaya-api` | `src/api/` | JSON-RPC request/response types, handler registry, RPC method definitions |

### 3.2 Binary Crates

| Binary | Path | Responsibility |
|--------|------|----------------|
| `udayad` | `src/main.rs` | Node daemon, lifecycle, P2P message loop, mining loop, RPC server |
| `udaya-miner` | `src/miner/` | Standalone miner CLI |
| `udaya-wallet-cli` | `src/wallet-cli/` | Wallet CLI operations |
| `udaya-faucet` | `src/faucet/` | Testnet faucet service |
| `udaya-pool` | `src/pool-server/` | Mining pool server |

### 3.3 Test Crates

| Crate | Path | Responsibility |
|-------|------|----------------|
| unit tests | `tests/unit/` | RPC security, core consensus, wallet, validation |
| integration tests | `tests/integration/` | Node integration scenarios |
| e2e-validation | `e2e-validation/` | Phase 10 end-to-end validation |
| stability-tests | `src/stability-tests/` | Long-running stability scenarios |

---

## 4. Consensus Implementation

### 4.1 Algorithm

Udaya uses SHA-256d (double SHA-256) Proof-of-Work, Bitcoin-compatible:

- **Block target time**: 600 seconds (10 minutes)
- **Difficulty adjustment**: Every 2,016 blocks (~14 days)
- **Maximum target (minimum difficulty)**: `0x1D00FFFF`
- **Genesis bits**: `0x1D00FFFF`
- **Halving interval**: 210,000 blocks
- **Initial block reward**: 50 UDYA (5,000,000,000 satoshis)
- **Maximum supply**: 21,000,000 UDYA
- **Coinbase maturity**: 100 blocks

### 4.2 Key Consensus Parameters

```rust
pub const SATS_PER_COIN: u64 = 100_000_000;
pub const BLOCK_TARGET_TIME_SECS: u64 = 600;
pub const INITIAL_BLOCK_REWARD: u64 = 50 * SATS_PER_COIN;
pub const HALVING_INTERVAL: u64 = 210_000;
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 2_016;
pub const MAX_BLOCK_WEIGHT: u64 = 4_000_000;
pub const MAX_BLOCK_SIZE: usize = 1_000_000;
pub const COINBASE_MATURITY: u64 = 100;
pub const PROTOCOL_VERSION: u32 = 70016;
```

### 4.3 Difficulty Adjustment

Bitcoin-style DAA with 4x constraint:

```
actual_timespan = clamp(target_timespan / 4, actual, target_timespan * 4)
new_target = current_target * actual_timespan / target_timespan
new_target = min(new_target, pow_limit)
```

Fast emergency adjustment available every 6 blocks.

### 4.4 Block Validation Chain

```
Block received
    │
    ▼
Structure validation (size, version, merkle root)
    │
    ▼
PoW validation (SHA-256d hash <= target)
    │
    ▼
Contextual validation (timestamp, difficulty, checkpoint)
    │
    ▼
Transaction validation (scripts, signatures, UTXO)
    │
    ▼
State update (UTXO set, chain tip, chainwork)
```

### 4.5 Anti-Selfish Mining

Minimum stale threshold of 2 blocks before propagation. Maximum reorg depth: 6 blocks. Finality depth: 100 blocks.

---

## 5. P2P Implementation

### 5.1 Protocol

Custom binary protocol with 24-byte message headers:

- Magic bytes: `[0xBF, 0x59, 0x1A, 0xE7]`
- Maximum payload: 32 MB
- Checksum: First 4 bytes of double SHA-256 of payload

### 5.2 Message Types

| Message | Direction | Purpose |
|---------|-----------|---------|
| `version` | Both | Handshake with version, user agent, start height |
| `verack` | Both | Handshake acknowledgment |
| `inv` | Both | Inventory announcement (blocks/txs) |
| `getdata` | Both | Request blocks or transactions |
| `getheaders` | Both | Request block headers for sync |
| `headers` | Both | Block header batch response |
| `block` | Both | Full block transmission |
| `tx` | Both | Transaction transmission |
| `addr` | Both | Peer address exchange |
| `reject` | Both | Rejection with reason |
| `sendheaders` | Both | Prefer header relay |

### 5.3 Connection Management

- Default max peers: 125 (configurable)
- Unlimited inbound connections (practical limits apply)
- Peer scoring and ban system (IP-based)
- Ban duration: 86400 seconds (24 hours) default

### 5.4 Discovery

- DNS seed nodes: `testnet-seed1.udaya.org`, `testnet-seed2.udaya.org`
- Peer exchange via `addr` messages
- Preferred peers configuration

### 5.5 Sync Protocol

Headers-first synchronization:

1. Connect to peer, exchange version/verack
2. If peer height > local height, send `getheaders` with block locator
3. Validate received headers (PoW, timestamp, checkpoint)
4. Request missing blocks via `getdata`
5. Validate and store blocks, update UTXO set
6. Relay valid blocks to other peers
7. Process orphan blocks when parent becomes available

---

## 6. Storage / RocksDB

### 6.1 Column Families

| Column Family | Key | Value |
|---------------|-----|-------|
| `blocks` | Block hash | Serialized block |
| `block_hashes` | Height (u64 BE) | Block hash |
| `block_headers` | Block hash | Serialized header |
| `block_height_by_hash` | Block hash | Height (u64 BE) |
| `chain_work` | Block hash | Cumulative chainwork (BigUint BE) |
| `transactions` | Txid | Serialized transaction |
| `utxo_set` | Outpoint key | UTXO entry |
| `chain_state` | Fixed keys (`chain_tip`, `chain_height`) | State values |

### 6.2 Performance Configuration

- Compression: LZ4 (configurable Zstd)
- Write buffer: 256 MB
- Max write buffer number: 8
- Background jobs: 4
- Parallelism: 4
- Block cache: Configurable (default 512 MB)
- Fsync disabled for performance (configurable)
- Bytes per sync: 1 MB

### 6.3 Batch Operations

All block storage uses write batches for atomicity:

```rust
store_block(&block, height) -> Writes block, header, transactions, chain state in single batch
store_block_batch(&blocks, &heights) -> Bulk batch write for sync
update_utxo_set_for_block(&block, height) -> Applies UTXO changes
```

---

## 7. Wallet Implementation

### 7.1 HD Wallet Structure

Udaya wallet supports BIP44, BIP49, BIP84, and BIP86 derivation paths:

- **BIP44 (Legacy P2PKH)**: `m/44'/257'/0'/0/0`
- **BIP49 (P2SH-SegWit)**: `m/49'/257'/0'/0/0`
- **BIP84 (Native SegWit)**: `m/84'/257'/0'/0/0`
- **BIP86 (Taproot)**: `m/86'/257'/0'/0/0`

Coin type: 257 (Udaya)

### 7.2 Address Formats

| Type | Bech32/Bech32m | Base58 Prefix |
|------|----------------|---------------|
| P2PKH | - | `U` |
| P2WPKH | `udaya1...` | - |
| P2TR | `udaya1...` | - |

### 7.3 Security Features

- **Zeroization**: Sensitive key material is securely wiped from memory (`zeroize` crate)
- **Secure random**: `rand` with `OsRng` for key generation
- **Threat model**: 20 defined threats with mitigations documented in `src/wallet/src/threat_model.rs`
- **PSBT support**: Partially signed Bitcoin transactions for multisig workflows
- **Treasury**: Multi-signature treasury management with daily limits

### 7.4 Wallet State

```rust
pub struct WalletState {
    pub name: String,
    pub version: String,
    pub network: String,
    pub encrypted: bool,
    pub accounts: Vec<WalletAccount>,
    pub active_account: u32,
    pub transactions: Vec<WalletTx>,
    pub utxos: Vec<WalletUTXO>,
    pub created_at: u64,
    pub last_updated: u64,
}
```

---

## 8. Mining Implementation

### 8.1 Standalone Miner

- CPU-based SHA-256d mining
- Configurable thread count
- Block template generation from mempool
- Testnet-optimized difficulty (`0x207FFFFF`)
- Automatic coinbase reward registration with wallet

### 8.2 Mining Pool

- Stratum V2 compatible protocol
- Miner registration and authentication
- Share difficulty and vardiff
- Block template distribution
- Payout calculation and processing

### 8.3 Mining Metrics

Prometheus metrics exposed for:
- Hashrate (H/s)
- Blocks found
- Miner thread count
- Current difficulty

---

## 9. Mempool

### 9.1 Configuration

```rust
pub struct MempoolConfig {
    pub max_tx_count: usize,        // 50,000
    pub max_tx_size: usize,         // 1,000,000 bytes
    pub min_fee_rate: u64,          // 1 sat/vbyte
    pub max_mempool_size_mb: usize, // 300 MB
    pub expiry_hours: u64,          // 72 hours
    pub max_orphan_tx_count: usize, // 10,000
}
```

### 9.2 Features

- Fee index (BTreeMap by fee rate)
- Orphan transaction handling
- Ancestor/descendant tracking
- Limit by count and size
- Expiration of stale transactions
- Transaction validation before acceptance

---

## 10. RPC / API Layer

### 10.1 JSON-RPC Methods

| Category | Methods |
|----------|---------|
| Blockchain | `getblockchaininfo`, `getblockcount`, `getblockhash`, `getblock`, `gettransaction`, `gettxout` |
| Mempool | `getmempoolinfo`, `getmempoolentry` |
| Wallet | `getbalance`, `getnewaddress`, `getaddressesbylabel`, `sendtoaddress`, `listunspent`, `listtransactions` |
| Mining | `getmininginfo`, `getblocktemplate`, `submitblock` |
| Network | `getpeerinfo`, `getnetworkinfo`, `addnode`, `getconnectioncount` |
| Control | `stop`, `uptime`, `getinfo`, `ping` |

### 10.2 REST Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/` | POST | JSON-RPC endpoint |
| `/health` | GET | Detailed health check |
| `/healthz` | GET | Liveness probe |
| `/readyz` | GET | Readiness probe |
| `/metrics` | GET | Prometheus metrics |

### 10.3 Security

- HTTP Basic Authentication (configurable)
- Rate limiting (IP-based, configurable RPS and burst)
- Request size limiting
- Restricted methods (require auth)
- TLS support (rustls)
- CORS configuration

---

## 11. Configuration

### 11.1 Configuration Structure

```rust
pub struct UdayaConfig {
    pub network: NetworkConfig,    // P2P ports, peers, UPnP
    pub storage: StorageConfig,    // Data dir, cache, pruning
    pub consensus: ConsensusConfig, // Network type, fees, block size
    pub mining: MiningConfig,      // Enable, threads, coinbase address
    pub wallet: WalletConfig,      // Enable, file path, fee rate
    pub rpc: RPCConfig,            // Auth, TLS, rate limits, CORS
    pub logging: LoggingConfig,    // Level, file, JSON format
}
```

### 11.2 Network Presets

| Network | P2P Port | RPC Port | Data Dir |
|---------|----------|----------|----------|
| mainnet | 9798 | 8332 | `data/udaya` |
| testnet | 19798 | 18332 | `temp_testnet_data` |
| regtest | 19798 | 18332 | `temp_testnet_data` |

### 11.3 Environment Variables

- `RPC_USER` - RPC username
- `RPC_PASSWORD` - RPC password

---

## 12. Security Controls

### 12.1 Cryptographic Primitives

| Component | Algorithm | Rust Crate |
|-----------|-----------|------------|
| Hashing | SHA-256d | `sha2` |
| Signatures | ECDSA secp256k1 | `secp256k1` |
| Key Derivation | PBKDF2, HMAC | `hmac`, `pbkdf2` |
| Encryption | AES-256-GCM | `aes-gcm` |
| Password Hashing | Argon2 | `argon2` |
| Fast Hash | BLAKE3 | `blake3` |
| Ed25519 | Ed25519-Dalek | `ed25519-dalek` |
| X25519 | X25519-Dalek | `x25519-dalek` |

### 12.2 Memory Safety

- Rust ownership model prevents memory corruption
- `zeroize` crate for secure memory wiping
- `subtle` crate for constant-time comparisons

### 12.3 Dependency Security

- `cargo-audit` for vulnerability scanning
- `cargo-deny` for license and advisory management
- `deny.toml` configured with strict policies

### 12.4 Fuzzing

- Built-in fuzzing engine (`FuzzingEngine`)
- Adversarial simulator (`AdversarialSimulator`)
- Targets: consensus, serialization, validation, mempool, networking

### 12.5 RPC Security

- HTTP Basic Auth
- IP-based rate limiting
- Request size limits
- Method restriction (unauthenticated access denied for sensitive methods)
- TLS optional

---

## 13. Flutter ↔ Rust Bridge Design

### 13.1 Bridge Strategy

Preferred approach: `dart:ffi` with a C-compatible stable API layer.

Android-specific: JNI/Kotlin shim where required (e.g., foreground service integration, Android Keystore).

### 13.2 FFI Boundary Rules

1. **No Rust internal types exposed**: All data crossing the FFI boundary must use C-compatible types (primitives, C strings, or opaque pointers with accessor functions).
2. **Input validation at every boundary**: Dart inputs are validated before reaching Rust. Rust validates again.
3. **Error handling**: All FFI functions return error codes or nullable pointers. Errors are converted to structured error objects in Dart.
4. **Memory ownership**: Clear ownership semantics. Rust allocates, Rust frees (or uses reference counting with explicit release).
5. **Thread safety**: All FFI functions must be thread-safe. Rust handles synchronization internally.

### 13.3 Stable API Categories

#### Node Lifecycle

```c
// Opaque node handle
typedef struct UdayaNode UdayaNode;

// Lifecycle
UdayaNode* udaya_node_create(const char* config_path);
int32_t udaya_node_start(UdayaNode* node);
int32_t udaya_node_stop(UdayaNode* node);
void udaya_node_destroy(UdayaNode* node);
int32_t udaya_node_is_running(UdayaNode* node);
```

#### Node Status

```c
typedef struct {
    uint64_t height;
    char best_block_hash[65];      // hex + null terminator
    char chainwork[65];             // hex + null terminator
    uint32_t peer_count;
    uint32_t sync_percentage;       // 0-100
    char network[32];               // "mainnet", "testnet", "regtest"
    char status[32];                // "stopped", "starting", "running", "stopping"
    uint64_t last_block_time;
} UdayaNodeStatus;

int32_t udaya_node_get_status(UdayaNode* node, UdayaNodeStatus* status);
```

#### Blockchain Queries

```c
int32_t udaya_blockchain_get_height(UdayaNode* node, uint64_t* height);
int32_t udaya_blockchain_get_block_hash(UdayaNode* node, uint64_t height, char* out_hash, size_t out_len);
int32_t udaya_blockchain_get_block(UdayaNode* node, uint64_t height, char* out_hex, size_t out_len, size_t* out_size);
int32_t udaya_blockchain_get_block_by_hash(UdayaNode* node, const char* hash, char* out_hex, size_t out_len, size_t* out_size);
```

#### Peer Information

```c
typedef struct {
    char address[64];
    char user_agent[128];
    uint64_t height;
    uint32_t ping_time_ms;
    uint8_t is_outbound;
    uint8_t is_connected;
} UdayaPeerInfo;

int32_t udaya_peer_get_count(UdayaNode* node, uint32_t* count);
int32_t udaya_peer_get_list(UdayaNode* node, UdayaPeerInfo* out_peers, uint32_t max_peers, uint32_t* actual_count);
```

#### Wallet Operations

```c
// Wallet handle (separate from node handle)
typedef struct UdayaWallet UdayaWallet;

UdayaWallet* udaya_wallet_create(const char* network);
int32_t udaya_wallet_create_new(UdayaWallet* wallet, const char* passphrase, char* out_seed_phrase, size_t out_len);
int32_t udaya_wallet_restore(UdayaWallet* wallet, const char* seed_phrase, const char* passphrase);
int32_t udaya_wallet_generate_address(UdayaWallet* wallet, char* out_address, size_t out_len);
int64_t udaya_wallet_get_balance(UdayaWallet* wallet);
int32_t udaya_wallet_get_transactions(UdayaWallet* wallet, char* out_json, size_t out_len, uint32_t max_count);
```

#### Transaction Operations

```c
typedef struct {
    char txid[65];
    char hex[4096];
    size_t hex_size;
    int32_t error_code;
    char error_message[256];
} UdayaTxResult;

int32_t udaya_wallet_send(UdayaWallet* wallet, const char* address, uint64_t satoshis, uint64_t fee_rate, UdayaTxResult* result);
int32_t udaya_transaction_broadcast(UdayaNode* node, const char* tx_hex, UdayaTxResult* result);
```

#### Mining Operations

```c
int32_t udaya_mining_get_status(UdayaNode* node, uint8_t* is_mining, double* hashrate, uint64_t* blocks_found);
int32_t udaya_mining_start(UdayaNode* node, uint32_t threads);
int32_t udaya_mining_stop(UdayaNode* node);
```

### 13.4 String Handling

All strings crossing the FFI boundary are null-terminated UTF-8 C strings.

- Output strings: caller provides buffer and length. Function returns required size or writes up to buffer size.
- Input strings: validated length (max 4096 for most inputs).
- Binary data (hex): encoded as null-terminated hex strings.

### 13.5 Error Model

All FFI functions return `int32_t`:

- `0`: Success
- `>0`: Warning (operation completed with caveats)
- `<0`: Error code (see error codes table)

Error details retrieved via `udaya_get_last_error(char* out_msg, size_t out_len)`.

### 13.6 Threading Model

- FFI functions are thread-safe.
- Node operations run on internal Tokio runtime.
- Dart receives results via callbacks or stream channels (see 13.8).
- Long-running operations (sync, mining) do not block the Dart isolate.

### 13.7 JNI/Kotlin Shim (Android)

For Android-specific features, a thin Kotlin layer sits between Dart FFI and Rust:

```kotlin
// Example: foreground service binding
class UdayaNodeService : Service() {
    private lateinit var nativeBridge: UdayaNativeBridge
    
    fun startNode(config: String): Int {
        return nativeBridge.nodeStart(config)
    }
    
    fun stopNode(): Int {
        return nativeBridge.nodeStop()
    }
    
    fun getNodeStatus(): NodeStatus {
        return nativeBridge.nodeStatus.toNodeStatus()
    }
}
```

The Kotlin shim handles:
- Android foreground service lifecycle
- Android Keystore integration for seed storage
- Battery optimization requests
- Network change callbacks
- Storage permission handling

### 13.8 Event Streaming

For real-time updates (block found, peer connected, sync progress), Rust pushes events through a callback registered at node creation:

```c
typedef void (*UdayaEventCallback)(const char* event_type, const char* event_data_json, void* user_data);

int32_t udaya_node_set_event_callback(UdayaNode* node, UdayaEventCallback callback, void* user_data);
```

Event types:
- `block_connected`
- `block_disconnected`
- `peer_connected`
- `peer_disconnected`
- `sync_progress`
- `wallet_balance_changed`
- `transaction_confirmed`
- `transaction_received`
- `mining_block_found`
- `node_error`

---

## 14. Node Operating Modes

### 14.1 Mode 1: Wallet / Light Client (Default)

**Purpose**: Ordinary users who want wallet functionality without running a full node.

**Behavior**:
- Connects to remote Udaya full nodes via P2P or RPC
- Provides wallet, balances, transaction history, address generation
- Does not store blockchain locally
- Does not validate blocks or transactions independently
- Relies on server-side validation

**Resource Profile**:
- Storage: Minimal (wallet data only, ~MB)
- Bandwidth: Moderate (RPC/P2P queries)
- Battery: Low
- CPU: Minimal

**Limitations**:
- Trusts remote nodes for validation
- Privacy: remote node knows addresses and balances

### 14.2 Mode 2: Light Node

**Purpose**: Users who want local verification without full blockchain storage.

**Behavior**:
- Downloads and validates block headers (not full blocks)
- Validates PoW and chainwork for headers
- Requests full blocks/transactions on demand from full nodes
- Clearly documents what is and is not verified locally

**Resource Profile**:
- Storage: ~100 MB (headers only)
- Bandwidth: Moderate to high
- Battery: Moderate
- CPU: Moderate (header validation)

**Limitations**:
- Does not validate full block transactions
- Requires trusted full node connections for transaction data
- SPV-style security model

### 14.3 Mode 3: Full Node

**Purpose**: Technically capable users running the complete Udaya Rust node on Android.

**Behavior**:
- Synchronizes complete blockchain from peers
- Validates all blocks and transactions
- Maintains full UTXO set in RocksDB
- Participates in P2P network
- Exposes full node statistics

**Requirements**:
- Device: 8 GB RAM minimum, 16 GB recommended
- Storage: 50 GB+ free space
- CPU: Modern ARM64 processor
- Android 10+ (API 29+)
- Persistent storage (not removable SD card for blockchain data)

**Resource Profile**:
- Storage: 50 GB+ (grows with blockchain)
- Bandwidth: High (block/tx relay)
- Battery: High (requires charging for continuous operation)
- CPU: High (validation, P2P, database writes)

**Warnings Required**:
- Continuous full-node operation consumes battery, CPU, RAM, storage, and network bandwidth
- Device heat and throttling possible
- Background execution restricted by Android OS
- Not recommended on devices without adequate thermal design

---

## 15. Android Background Node

### 15.1 Foreground Service Architecture

```kotlin
class UdayaForegroundService : Service() {
    private val binder = LocalBinder()
    private var nodeThread: Thread? = null
    
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        startForeground(NOTIFICATION_ID, buildNotification("Udaya Node Starting"))
        startNode()
        return START_STICKY
    }
    
    override fun onDestroy() {
        stopNodeGracefully()
        super.onDestroy()
    }
    
    inner class LocalBinder : Binder() {
        fun getService(): UdayaForegroundService = this@UdayaForegroundService
    }
}
```

### 15.2 Notification Requirements

- Persistent notification with node status
- Action buttons: Start, Stop, View Status
- Status channel for foreground service
- Must not be dismissible by user while node is running

### 15.3 Battery-Aware Controls

- Detect charging state via `BatteryManager`
- Pause mining on battery (optional)
- Reduce P2P connection count on battery
- Throttle sync speed on battery
- Alert user when battery is low

### 15.4 Network Resilience

- Monitor connectivity via `ConnectivityManager`
- Pause node on network loss, resume on reconnect
- Prefer Wi-Fi for full node (configurable)
- Handle mobile/Wi-Fi transitions gracefully
- Queue transactions during network outage

### 15.5 Storage Management

- Check available storage before sync
- Warn user when storage falls below threshold
- Support pruning configuration
- Never delete blockchain data without explicit confirmation
- Report storage usage in dashboard

### 15.6 Crash Recovery

- Service auto-restarts via `START_STICKY`
- Node state persisted to disk (RocksDB)
- Wallet state encrypted and persisted
- On restart: resume from last known tip
- Validate chain continuity before accepting new blocks

### 15.7 Android Permissions

Required permissions:
- `INTERNET` - Network access
- `ACCESS_NETWORK_STATE` - Network change detection
- `FOREGROUND_SERVICE` - Background node operation
- `FOREGROUND_SERVICE_DATA_SYNC` - Data sync foreground service
- `POST_NOTIFICATIONS` - Node status notification
- `WAKE_LOCK` - Keep CPU awake during critical operations
- `RECEIVE_BOOT_COMPLETED` - Auto-start on device boot (optional)

Not permitted / not attempted:
- Bypassing Doze mode restrictions
- Background execution limits bypass
- Root access requirements

---

## 16. Flutter Application Structure

### 16.1 Directory Layout

```
apps/udaya_mobile/
├── lib/
│   ├── main.dart
│   ├── app.dart
│   ├── core/
│   │   ├── constants/
│   │   ├── errors/
│   │   ├── extensions/
│   │   └── theme/
│   ├── data/
│   │   ├── models/
│   │   ├── repositories/
│   │   └── datasources/
│   │       ├── ffi/
│   │       └── local/
│   ├── domain/
│   │   ├── entities/
│   │   ├── repositories/
│   │   └── usecases/
│   ├── presentation/
│   │   ├── screens/
│   │   ├── widgets/
│   │   ├── providers/
│   │   └── routes/
│   └── services/
│       ├── node_service.dart
│       ├── wallet_service.dart
│       └── secure_storage.dart
├── android/
│   ├── app/src/main/
│   │   ├── kotlin/.../MainActivity.kt
│   │   ├── java/.../UdayaNodeService.kt
│   │   └── AndroidManifest.xml
│   └── build.gradle
├── ios/
├── linux/
├── windows/
├── macos/
├── web/
├── pubspec.yaml
└── README.md
```

### 16.2 State Management

- **Riverpod** for global state (node status, wallet balance, network info)
- **Freezed** for immutable models
- **json_serializable** for JSON conversion
- **go_router** for navigation

### 16.3 Design System

- Material 3 as base
- Udaya custom theme colors
- Dark mode default (crypto users preference)
- Clear network state indicators (MAINNET/TESTNET/SYNCING)
- High contrast for critical information

---

## 17. Required Screens

| Screen | Purpose | Mode Support |
|--------|---------|--------------|
| Dashboard | Node status, balance, network overview | All |
| Wallet | Addresses, UTXOs, balance, account management | All |
| Send | Create and broadcast transaction | All |
| Receive | Address display, QR code | All |
| Transactions | Transaction history with filters | All |
| Blockchain Explorer | Block and transaction search | All (full node gets richer data) |
| Node | Node control, status, logs | Full node only |
| Peers | Connected peer list, latency, direction | Full node only |
| Mining | Hashrate, blocks found, mining address | Full node / testnet |
| Network | Network stats, difficulty, hash rate | All |
| Testnet | Testnet toggle, faucet, test coins warning | All |
| Security | Seed backup, encryption, security status | All |
| Settings | Configuration, network selection, preferences | All |
| Backup & Recovery | Seed phrase, wallet restore | All |
| Logs | Application and node logs | All |
| Developer/Diagnostics | FFI logs, bridge status, debug info | All |

---

## 18. Build System Integration

### 18.1 Rust to Android

Use `cargo-ndk` to build Rust libraries for Android:

```bash
cargo install cargo-ndk
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 build --release
```

### 18.2 Flutter FFI Setup

```yaml
# pubspec.yaml
dependencies:
  ffi: ^2.1.0
  flutter_secure_storage: ^9.0.0
  qr_flutter: ^4.1.0
  mobile_scanner: ^4.0.0
  riverpod: ^2.4.0
  freezed_annotation: ^2.4.0
  json_annotation: ^4.8.0
```

### 18.3 JNI Setup (Android)

```gradle
// android/app/build.gradle
android {
    defaultConfig {
        ndk {
            abiFilters 'arm64-v8a', 'armeabi-v7a', 'x86_64'
        }
        externalNativeBuild {
            cmake {
                cppFlags '-std=c++17'
            }
        }
    }
    externalNativeBuild {
        cmake {
            path 'src/main/cpp/CMakeLists.txt'
        }
    }
}
```

---

## 19. Security Considerations

### 19.1 FFI Boundary Security

- All inputs from Dart are validated in Rust
- Buffer sizes enforced, no unbounded reads
- No raw pointers exposed to Dart (opaque handles only)
- Thread-local error state, no shared mutable state across FFI

### 19.2 Secret Management

- **Never expose private keys or seed phrases in logs**
- Use Android Keystore for sensitive application secrets
- Seed phrases stored encrypted with hardware-backed keystore
- Clipboard cleared after copy-to-clipboard operations
- Screenshot prevention on sensitive screens (FLAG_SECURE)

### 19.3 Network Security

- TLS for all RPC connections when available
- Certificate pinning for known seed nodes
- No plaintext transmission of credentials
- Local RPC bound to localhost only (127.0.0.1)

### 19.4 Dependency Security

- `cargo audit` run in CI
- `cargo deny` enforced in CI
- Flutter dependency scanning
- Regular secret scanning

### 19.5 Input Validation

- QR code data validated before use
- Address format validation before sending
- Amount validation (no negative, no overflow)
- Transaction confirmation dialogs for large amounts

---

## 20. Testing Strategy

### 20.1 Rust Tests

```bash
cargo test --lib --all-features
cargo test --test '*' --all-features
cargo test --doc --all-features
cargo bench
```

### 20.2 Flutter Tests

```bash
flutter analyze
flutter test
flutter test integration_test/
```

### 20.3 Bridge Tests

- FFI unit tests for each exported function
- Memory lifecycle tests (create/use/destroy)
- Error path coverage
- Thread safety tests
- Android integration tests

### 20.4 End-to-End Flow

```
Create wallet → Backup seed → Restore wallet → Generate address → Receive UDYA → 
Send UDYA → Broadcast transaction → Mine block (testnet) → Confirm transaction → 
Restart node → Synchronize → Verify balance
```

---

## 21. Performance Targets

| Metric | Target |
|--------|--------|
| Cold start (wallet mode) | < 3 seconds |
| Node startup (full node) | < 30 seconds |
| Transaction creation | < 500ms |
| Balance query | < 100ms |
| Block lookup | < 200ms |
| Peer list query | < 150ms |
| FFI overhead per call | < 5ms |
| Memory overhead (Flutter) | < 150 MB |
| Battery (full node, 1h) | < 15% |

---

## 22. Known Limitations

1. **Full node on Android requires 8+ GB RAM and 50+ GB storage**
2. **Phone mining is not economically viable** - display warnings
3. **Light node mode does not validate full transactions** - document clearly
4. **Android background execution limits apply** - foreground service required
5. **RocksDB requires NDK build** - increases APK size
6. **Full node sync can take hours on mobile data** - Wi-Fi preferred
7. **Consensus-critical code never in Dart** - FFI overhead acceptable

---

## 23. Anti-Patterns (Do Not Implement)

| Anti-Pattern | Why It's Prohibited |
|--------------|---------------------|
| PoW validation in Dart | Consensus-critical, must be in Rust |
| Independent blockchain state in Dart | Risk of fork from canonical chain |
| Key derivation in Dart | Must reuse audited Rust crypto |
| Transaction script interpreter in Dart | Consensus-critical |
| Difficulty calculation in Dart | Consensus-critical |
| Light client claiming to be full node | Misleading, security risk |
| Seed phrase stored in SharedPreferences | Insecure storage |
| RPC credentials in plain text | Security vulnerability |
| Blocking Dart isolate for node ops | ANR, poor UX |

---

## 24. Next Steps

1. Define exact FFI function signatures with error codes
2. Implement C-compatible wrapper crate (`udaya-ffi`)
3. Set up `cargo-ndk` build pipeline
4. Create Flutter project structure (`apps/udaya_mobile/`)
5. Implement secure storage layer (Android Keystore)
6. Implement node service (Flutter side)
7. Implement FFI bindings (Dart side)
8. Create Dashboard screen
9. Implement wallet screens (create, backup, restore, send, receive)
10. Implement node screens (status, peers, logs)
11. Implement explorer screens
12. Implement mining screens
13. Implement testnet mode
14. Implement security screens
15. Implement settings and diagnostics
16. Write tests
17. Build and test on physical Android devices

---

*End of Architecture Document*