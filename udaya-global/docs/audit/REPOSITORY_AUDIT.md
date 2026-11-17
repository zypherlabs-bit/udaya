# Udaya Global — Repository Audit (Phase 0)

**Date:** 2026-11-16  
**Scope:** `udaya-global/` only. The pre-existing Udaya mainnet at the repository root is treated as protected and was not modified.  
**Auditor posture:** Principal Blockchain Architect / Senior Rust Engineer / Independent Security Auditor.  
**Status:** Baseline audit only. No Phase 1 implementation has been performed. Udaya Global is **not** production-ready.

---

## 1. Repository structure

Top-level `udaya-global/`:

```
udaya-global/
  Cargo.toml                 # workspace (4 active members)
  Cargo.lock
  README.md
  crates/
    ug-core/                 # canonical types, tx, UTXO state, blocks, genesis, mempool, crypto
    ug-consensus/            # U-ADC BFT consensus engine, validator sets, votes, double-sign guard
    ug-net/                  # TCP transport, Noise XX, handshake, framing, peer store, routing
    ug-node/                 # node binary, config, storage, event loop, verifier, cluster tests
  docs/
    PHASE4_PROTOTYPE.md
  prototype/
    uadc_core/__init__.py    # Python reference core (spec-traced)
    run_tests.py             # 13 spec-trace checks
  src/                      # EXCLUDED scaffold (not compiled): core/, p2p/, storage/, crates/*
  target/                   # build artifacts (not source)
```

Workspace members (`udaya-global/Cargo.toml`): `crates/ug-core`, `crates/ug-consensus`, `crates/ug-net`, `crates/ug-node`.  
Excluded from the workspace: `src/`, `prototype/`, and every `crates/udaya-*` stub directory. Those remain as historical/unverified research output and are **not** compiled into the node.

---

## 2. Rust workspace and Cargo configuration

- Resolver: `resolver = "2"`.
- Workspace package version: `0.1.0`, edition `2021`, license MIT.
- Toolchain observed: `rustc 1.100.0 (0ef1bc386)`, `cargo 1.100.0`, host `x86_64-pc-windows-gnu`.
- Required native tool for the GNU toolchain (`dlltool.exe`) is provided by `C:\msys64\mingw64\bin`; it is not on a default PATH in this environment.
- `clippy` and `rustfmt` components were not initially installed; both were added as standard Rust components to execute the baseline (no third-party dependencies added).
- `Cargo.lock` is committed (40,164 bytes).

Workspace dependencies are declared but several are unused at workspace scope: `anyhow`, `bincode`, `env_logger`, `parking_lot` (Cargo warning). `ug-core` declares `log` as a dependency but does not use it (Cargo warning). `workspace.package.description` is unused.

---

## 3. Existing blockchain architecture

Udaya Global is an **independent experimental Layer-1** with a UTXO transaction model and a Tendermint-style BFT consensus (U-ADC). It is **not** the existing Udaya proof-of-work chain and does not import its history, balances, peers, or genesis.

Consensus-critical implementation lives in `ug-core` + `ug-consensus` + `ug-net` + `ug-node`. A Python reference core (`prototype/uadc_core`) models candidate 2-chain BFT rules and passes 13/13 spec-trace checks.

---

## 4. Current consensus implementation

- Crate: `ug-consensus`.
- Engine: deterministic, transport-independent; produces `Action` values.
- Properties claimed in code/docs:
  - Double-sign protection in memory and persisted to an append-only, fsynced journal before signing.
  - Votes/proposals bound to chain ID and validator key.
  - Commit requires a quorum certificate with at least `2f+1` distinct pre-commits from a `3f+1` validator set.
  - Locks released only by a higher-round quorum (Tendermint-style).
- Explicitly **not** implemented: committee rotation, dynamic validator join/leave, VRF proposer selection, state sync, slashing/evidence handling beyond data structures, epoch/weight changes. Validator set is fixed at genesis.
- Limits: `MAX_HEIGHT = i64::MAX`, `MAX_ROUND = 10_000_000`, `MAX_VALIDATORS`/`MIN_VALIDATORS` defined in `validator.rs`.

---

## 5. P2P networking implementation

- Crate: `ug-net`.
- Transport: TCP with Noise XX (`Noise_XX_25519_ChaChaPoly_BLAKE2s`).
- Authentication: Ed25519 identity exchanged in an encrypted Hello; signature binds peer ID, chain ID, genesis hash, both Noise static keys, nonce, advertised listen address.
- Framing: length-prefixed, bounded before allocation (`MAX_FRAME_BYTES = 8 MiB`, `HANDSHAKE_FRAME_MAX = 4096`).
- Rate limiting: per-peer token buckets for messages and bytes; overflow is an error (disconnect).
- Peer store: atomic persistence, failure backoff, banning; routing table is a bounded Kademlia-style bucket structure.
- Not implemented: QUIC, NAT traversal/hole punching, full DHT. `MAX_BLOCKS_PER_MESSAGE = 2`, `MAX_COMMITS_PER_MESSAGE = 2`.
- README for the excluded scaffold (`src/p2p`) documents an older isolated config (port 19799, DNS seeds disabled); it is not part of the compiled workspace.

---

## 6. Transaction processing

- Crate: `ug-core` (`tx.rs`, `mempool.rs`, `state.rs`).
- Model: UTXO. Canonical encoding; txid is SHA-256d, domain-separated and bound to chain ID.
- Validation: duplicate inputs rejected, value conservation exact and overflow-safe, zero-value outputs rejected, fee minimum enforced, signatures bound to chain and key, replay protection via confirmed-transaction tracking.
- Mempool: parent/child admission, deterministic fee-ordered templates, reconciliation drops conflicting spends while keeping valid children, expiry, snapshot roundtrip with strict decode.
- Supply cap: `MAX_SUPPLY_SATS = 2_100_000_000_000_000` (21,000,000 coins at 1e8 sats/coin). Enforced in genesis and additions.

---

## 7. Block validation

- Crate: `ug-core` (`block.rs`).
- Checks: timestamp monotonicity, future drift bound (`MAX_FUTURE_DRIFT_SECS = 120`), size/tx count limits (`MAX_BLOCK_BYTES`, `MAX_BLOCK_TXS = 8192`), transaction validation, state transition application, state root computation.
- `Block::execute` validates against parent header and state; genesis execution distinct.
- Clippy flagged a large size difference between enum variants (warning, non-blocking).

---

## 8. Cryptographic implementation

- Primitives: SHA-256 (`sha2`), Ed25519 (`ed25519-dalek 2`), X25519 (`x25519-dalek 2`), Noise (`snow 0.10`), bech32 addresses.
- Merkle tree in `crypto.rs`; tagged hashing for domain separation.
- No post-quantum, no threshold signatures, no VRF, no aggregate signatures in the compiled crate set.
- `zeroize` used for key material in places; debug output of identity avoids printing secret material (tested).

---

## 9. Database and storage

- Compiled crate `ug-node` uses a filesystem-backed `BlockStore` (`store.rs`) plus JSON config and seed files; atomic writes with fsync + rename.
- Persisted: blocks, headers, quorum certificates, state snapshots, node config, peer store (in `ug-net`).
- No RocksDB in the compiled workspace despite the excluded scaffold referencing it.
- `data/` directories are git-ignored; chain data is local.
- Storage root path defaults to `data/udaya-global-experimental` in the scaffold and `data/` in the compiled store.

---

## 10. Wallet implementation

- No wallet crate is part of the Udaya Global workspace. The `ug-node` binary does not implement wallet RPC or key management beyond validator seed files (`validator.key`).
- Wallet functionality exists only in the pre-existing Udaya mainnet project (`src/wallet`, `src/wallet-cli`) and is **not** reused by Udaya Global.
- Gap: no wallet, no address derivation UI, no transaction signing tooling for end users in Udaya Global.

---

## 11. RPC interfaces

- None implemented in Udaya Global. The node exposes a CLI (`init`, `keygen`, `run`, `status`, `verify`) and an in-process event loop; there is no HTTP/JSON-RPC/gRPC server in the compiled workspace.
- Root Udaya mainnet has RPC/explorer; not reused.

---

## 12. Node synchronization

- Implemented in `ug-node` event loop: peer ready/closed, message handling, `GetBlocks`/`Blocks`, `GetCommits`/`Commits`, out-of-order commit buffering (`max_pending_commits`), locator-based block requests, height tracking.
- Catch-up verified by the cluster integration test (`four_validator_cluster_reaches_consensus_and_catches_up`).
- No state-sync (snapshot transfer) beyond replay; no fast sync; no light client.

---

## 13. Testing infrastructure

- Rust: `cargo test --workspace` → **129 tests passed, 0 failed** (34 consensus + 55 core + 27 net + 11 node unit + 2 cluster integration + doc tests 0).
- Python: `prototype/run_tests.py` → **13/13 passed**.
- Benchmarks: `benches/` exist for core, consensus, net, node (criterion), but the `ug-net` bench does not compile under clippy (`cannot move out of captured variable`).
- No fuzzing, no property-based tests, no adversarial network tests, no long-running soak tests in the compiled workspace.

---

## 14. Build and deployment scripts

- Build: standard `cargo build --workspace` / `cargo test --workspace`.
- Deployment scripts are not present in `udaya-global/` (no `deployment/` directory). Root Udaya mainnet has deployment scripts; not audited for Udaya Global reuse.
- `kind.exe` in the root is unrelated tooling.

---

## 15. Documentation

- `udaya-global/README.md` states NO-GO for production/live validators.
- `udaya-global/docs/PHASE4_PROTOTYPE.md` documents scope, evidence, limits.
- Research inputs referenced: `uadc-research/phase3/UADC_BFT_SPEC_FINALIZED.md`, `uadc-research/FORMAL_CONSENSUS_MODEL.md` (outside `udaya-global/`).
- No architecture doc for P2P/storage in `udaya-global/docs/` beyond the crate-level Rust doc comments.

---

## 16. Dependencies and security configurations

- No `deny.toml` in `udaya-global/` (root Udaya has one, but it governs the mainnet workspace).
- No `cargo-audit`, `cargo-deny`, or `cargo-outdated` tooling installed in this environment.
- Dependency set is small and mainstream; see `DEPENDENCY_SECURITY_REPORT.md`.
- Security-relevant config: `serde(deny_unknown_fields)` on node config; strict decoders; frame limits; rate limits; atomic file writes.

---

## 17. Verified evidence summary

| Check | Result |
|---|---|
| `cargo build --workspace` | PASS (4.15s, warnings only) |
| `cargo test --workspace` | PASS, 129 tests |
| `cargo clippy --workspace` (default targets) | PASS with warnings (23 warning lines) |
| `cargo clippy --workspace --all-targets` | FAIL on `ug-net` bench (E0507 move-out-of-captured-variable) |
| `cargo fmt --check` | FAIL (many formatting diffs) |
| Python `prototype/run_tests.py` | PASS 13/13 |
| `cargo audit` / `cargo deny` | NOT AVAILABLE |

---

## 18. Incomplete / placeholder modules

- Excluded `src/` scaffold (PoW-style experimental consensus, P2P, storage) is not compiled or spec-traced to the U-ADC implementation.
- Excluded `crates/udaya-*` stubs are empty module placeholders.
- No wallet, no RPC, no explorer, no light client, no state sync, no committee rotation, no slashing enforcement, no production key management, no deployment packaging for Udaya Global.
- `EXPERIMENTAL_GENESIS_HASH` is an explicit placeholder, not a verified genesis.

---

## 19. Dead code and warnings

- Clippy: unused dependency `log` in `ug-core`; unused workspace deps; `pubkey` never used (consensus); redundant closure; `if` with identical blocks; too-many-arguments (9/7); clone on `Copy` types; large enum variant size difference; match for single pattern; sort_by_key suggestion.
- These are non-blocking but indicate unfinished cleanup.

---

## 20. Mainnet isolation evidence

See `MAINNET_PROTECTION_REPORT.md`. Summary: distinct chain ID prefix `udaya-global`, magic `0x55 0x47 0x4C 0x01`, port `29798`, address HRP `ugdev`, supply cap 21,000,000 coins. Explicit refusal of protected mainnet magic `0xBF 0x59 0x1A 0xE7`, chain IDs `mainnet`/`udaya-mainnet`/`udaya`, and port `9798`. No connection to or modification of existing mainnet artifacts.
