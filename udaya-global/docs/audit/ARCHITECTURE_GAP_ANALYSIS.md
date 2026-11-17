# Udaya Global — Architecture Gap Analysis (Phase 0)

**Scope:** `udaya-global/` compiled workspace (`ug-core`, `ug-consensus`, `ug-net`, `ug-node`) plus excluded scaffolds.  
**Method:** Verified against source, tests, and build results only. No implementation changes made.

## Gap table

| Component | Existing status | Problems | Required changes | Priority |
|---|---|---|---|---|
| P2P networking | Implemented (TCP + Noise XX, framing, rate limits, peer store, Kademlia-style routing) | TCP only; no QUIC; bench does not compile (E0507); no NAT traversal; max 2 blocks/commits per message | Fix bench compile; add transport abstraction; evaluate QUIC; add peer scoring/banning tests; document protocol | High |
| Peer discovery | Partial (peer store, bootstrap list, routing table) | DNS seeds disabled by default; no DHT; no authenticated peer exchange beyond `Peers` message; limited scale testing | Implement authenticated peer discovery, optional DNS seeds, eclipse-resistance controls, discovery tests | High |
| Transaction relay | Implemented (mempool admission, fee ordering, templates, reconciliation) | No dedicated relay protocol/backpressure beyond rate limits; no transaction inventory (inv/getdata) pattern | Add inv/getdata-style relay, backpressure, propagation tests, mempool sync on reconnect | Medium |
| Block propagation | Implemented (Blocks/Commits messages, locator sync, out-of-order buffering) | Only 2 blocks/commits per message; no compact blocks; no announcement/relay optimization | Compact block relay, larger bounded batches, propagation benchmarks, partition tests | Medium |
| Consensus | Implemented U-ADC BFT engine with double-sign guard, QC, locks | Fixed validator set; no rotation, VRF, slashing enforcement, epochs, or state sync; net bench compile failure unrelated but blocks full validation | Committee rotation, evidence processing, slashing, VRF proposer selection, formal verification | Critical |
| Transaction validation | Implemented (structure, inputs, signatures, conservation, fees, replay protection) | No script/programmability; no fee market dynamics beyond min fee | Keep UTXO validation; add property tests; define fee market and limits | Medium |
| Double-spend protection | Implemented at state and mempool levels; consensus finality via BFT | Depends on correct consensus execution and finality; no evidence-based punishment yet | Add equivocation evidence handling and slashing; end-to-end tests under partitions | Critical |
| Storage | Implemented filesystem BlockStore + atomic writes + snapshots + peer store | No RocksDB in compiled workspace; no pruning; no archival strategy; no backup/restore tooling | Decide storage backend; add pruning, backup/restore, integrity checks, storage benchmarks | High |
| Node synchronization | Implemented event-loop catch-up via commits/blocks | No state sync/snapshot transfer; replay-only recovery; no fast sync | Add snapshot-based state sync, checkpointing, recovery tests | High |
| Cryptography | Ed25519, SHA-256, X25519, Noise XX, bech32, merkle | No post-quantum, threshold, VRF, or aggregate signatures; supply of randomness not audited for production | Crypto review, key rotation, CSPRNG audit, consider threshold/VRF for consensus | Critical |
| Wallet | Not implemented in Udaya Global | No wallet crate, no address tooling, no signing UX; only validator seed files | Design wallet crate, key management, HD derivation, signing, recovery | High |
| Security | Strong input validation, frame limits, rate limits, isolation guards | No fuzzing, no adversarial tests, no SBOM, no formal review; clippy warnings; formatting non-compliant | Fuzzing, property tests, security review, CI gates, SBOM, reproducible builds | Critical |
| Testing | 129 Rust tests + 13 Python tests passing | No fuzz/property/adversarial/soak tests; bench compile failure; no coverage metrics | Add fuzz, proptest, network fault injection, soak tests, coverage, CI | High |

---

## Detailed findings

### P2P networking
- **Verified:** `ug-net` provides encrypted/authenticated transport, strict framing, rate limiting, peer store, routing table; 27 unit tests pass.
- **Gap:** TCP-only; no QUIC; no NAT traversal; bench compile failure blocks performance validation.

### Peer discovery
- **Verified:** Bootstrap peers, peer store persistence, routing table bounded Kademlia-style buckets; tests for persistence/backoff/banning.
- **Gap:** DNS seeds disabled by default; no full DHT; eclipse resistance not tested.

### Transaction relay
- **Verified:** Mempool admission, parent/child ordering, deterministic templates, reconciliation; tests pass.
- **Gap:** No explicit relay protocol or propagation measurements; no inv/getdata inventory exchange.

### Block propagation
- **Verified:** `Blocks`/`Commits` messages with locators; out-of-order commit buffering; cluster catch-up test passes.
- **Gap:** Batch size limited to 2; no compact blocks; no propagation benchmarks.

### Consensus
- **Verified:** BFT engine, double-sign guard with fsynced journal, QC validation, lock rules; 34 tests pass; cluster reaches consensus with 4 validators and tolerates 1 failure.
- **Gap:** Fixed validator set; no rotation, VRF, slashing enforcement, or state sync; not formally verified.

### Transaction validation
- **Verified:** Structural and semantic checks, signatures, conservation, fees, replay protection; tests pass.
- **Gap:** No programmable scripts; fee market not defined beyond minimum.

### Double-spend protection
- **Verified:** State-level rejection; mempool conflict handling; consensus finality.
- **Gap:** Equivocation evidence handling and slashing not implemented.

### Storage
- **Verified:** Atomic fsync+rename writes, block/QC/state snapshot persistence, reopen verification; tests pass.
- **Gap:** Filesystem only; no pruning, backup/restore, or alternate backend decision.

### Node synchronization
- **Verified:** Locator-based block fetch, commit catch-up, buffering; cluster test passes.
- **Gap:** No snapshot/state sync; replay-only recovery.

### Cryptography
- **Verified:** Ed25519 signatures, SHA-256, X25519, Noise XX, domain separation, bech32; tests pass.
- **Gap:** No post-quantum, threshold, VRF, or aggregate schemes; production key management undefined.

### Wallet
- **Verified:** Not present in Udaya Global workspace.
- **Gap:** Entire wallet layer missing; must be designed before user-facing usage.

### Security
- **Verified:** Strict decoders, frame limits, rate limits, isolation guards, atomic writes, unknown-field rejection.
- **Gap:** No fuzzing/property/adversarial/soak tests; clippy warnings; formatting non-compliance; no SBOM.

### Testing
- **Verified:** 129 Rust tests + 13 Python tests pass.
- **Gap:** No fuzz/property/adversarial/soak/coverage; bench compile failure.

---

## Priority legend
- Critical: safety/consensus/security blockers for any validator use.
- High: required for robust node operation.
- Medium: performance/UX improvements.
