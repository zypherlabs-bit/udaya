# Udaya Global — Independent Layer-1 Blockchain

**Status:** Phase 4 research prototype (gate-constrained). **NO-GO for production/live validators.**  
See `docs/release/UDAYA_GLOBAL_FINAL_GO_NO_GO_REPORT.md` for the final certification verdict.

## What this is

Udaya Global is an independent Layer-1 blockchain built as a separate Rust workspace from the pre-existing
Udaya proof-of-work mainnet. It uses **U-ADC** (Udaya Adaptive Distributed Consensus), a Tendermint-style
Byzantine fault tolerant consensus engine, and a UTXO transaction model.

## Isolation from the existing Udaya mainnet (protected)

Udaya Global maintains its own identity and never touches the existing Udaya mainnet:

| Parameter | Udaya Global | Existing mainnet (protected) |
|---|---|---|
| Chain ID | `udaya-global-*` | `mainnet` |
| Network magic | `0x55 0x47 0x4C 0x01` | `0xBF 0x59 0x1A 0xE7` |
| P2P port | `29798` | `9798` |
| Address HRP | `ugdev` | `UDYA` |
| Consensus | U-ADC BFT | SHA-256d Proof-of-Work |
| Max supply | 21,000,000 coins | 21,000,000 coins |

The root `src/` mainnet (genesis, chain identity, consensus, emission rules, 21,000,000 UDYA cap, 500,000
UDYA promoter allocation) is **not modified**. Udaya Global does not import mainnet balances or history.

## Compiled workspace

| Crate | Purpose |
|---|---|
| `crates/ug-core` | Canonical encoding, transactions, UTXO state, blocks, genesis, mempool, fee policy, reward accounting |
| `crates/ug-consensus` | U-ADC BFT engine, validator set, votes, quorum certificates, double-sign guard |
| `crates/ug-net` | TCP + Noise XX transport, authenticated handshake, framing, rate limiting, peer store, routing |
| `crates/ug-node` | Node runtime, config, storage, verifier, cluster |

The `src/`, `prototype/` and `crates/udaya-*` directories are excluded from the workspace and remain
historical research output.

## Quality gates

- `cargo fmt --check --all` — PASS
- `cargo build --workspace` — PASS
- `cargo test --workspace` — PASS (150 tests)
- `cargo clippy --workspace --all-targets` — PASS (0 warnings)
- `cargo audit` — PASS (0 vulnerabilities)
- `cargo deny check` — PASS
- SBOM — generated (CycloneDX JSON per crate)

## Release artifacts

- `docs/release/UDAYA_GLOBAL_FINAL_GO_NO_GO_REPORT.md` — final certification (NO-GO)
- `docs/release/FINAL_PROJECT_COMPLETION_REPORT.md` — completion report

## Not yet production-ready

Wallet, RPC/explorer, QUIC/NAT/IPv6 transport hardening, committee rotation, slashing, an independent
external security audit, and a signed published release are not yet complete. Production rewards are not
activated.

## Repository

`https://github.com/zypherlabs-bit/udaya` (workspace: `udaya-global/`)

## License

MIT
