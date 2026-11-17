# Udaya Global — Phase 1 Implementation Plan (Planning Only)

**Status:** Plan only. No implementation has been performed in Phase 0.  
**Goal:** Establish a clean, verified, isolated foundation before adding features.

---

## 1. Phase 1 objectives

1. Make all quality gates green (`build`, `test`, `clippy --all-targets`, `fmt`).
2. Remove dead code and unused dependencies.
3. Resolve identity inconsistency between compiled workspace and excluded scaffold.
4. Add missing test layers (fuzz, property, adversarial, soak).
5. Define and document the storage backend and wallet/RPC scope decisions.
6. Keep mainnet fully isolated and unmodified.

---

## 2. Work packages

### WP-1: Tooling and CI baseline
- Add CI pipeline running `cargo build`, `cargo test`, `cargo clippy --all-targets`, `cargo fmt --check`.
- Install `cargo-audit`/`cargo-deny` in CI and add `deny.toml` to `udaya-global/`.
- Add coverage reporting.

### WP-2: Code hygiene
- Fix `ug-net` benchmark compile error (E0507) by avoiding move of non-`Copy` config in `b.iter`.
- Apply `cargo fmt` to all crates.
- Resolve clippy warnings (unused deps, too-many-args, clone-on-copy, large enum variants, identical `if` blocks, unused `pubkey`).
- Remove unused workspace dependencies (`anyhow`, `bincode`, `env_logger`, `parking_lot`) and `log` from `ug-core`.

### WP-3: Identity and isolation hardening
- Reconcile scaffold (port 19799) vs compiled workspace (port 29798) or archive the scaffold.
- Single source of truth for protected mainnet constants.
- Add CI invariant test that `GLOBAL_MAGIC != PROTECTED_MAINNET_MAGIC` and port != 9798.

### WP-4: Testing expansion
- Add fuzz targets for `codec`, `tx`, `block`, message decode.
- Add property tests for tx validation and state transitions.
- Add adversarial P2P tests (flooding, malformed frames, magic mismatch).
- Add soak/multi-node tests beyond the 4-validator cluster.

### WP-5: Storage decision
- Decide between filesystem-only vs RocksDB backend; document.
- Add pruning, backup/restore, integrity verification.
- Define snapshot/state-sync format.

### WP-6: Wallet and RPC scope
- Decide whether wallet/RPC are in scope for Udaya Global Phase 2.
- If yes, design `ug-wallet` crate and a minimal JSON-RPC surface; if no, document deferral.

### WP-7: Consensus extensions (design first)
- Design committee rotation, VRF proposer selection, slashing/evidence processing.
- No implementation until design review and formal model update.

---

## 3. Phase 1 exit criteria

- All workspace crates build with 0 warnings (or documented exceptions).
- `cargo test --workspace` passes with expanded tests.
- `cargo clippy --workspace --all-targets` passes with 0 warnings.
- `cargo fmt --check` passes.
- `cargo audit`/`cargo deny` clean (or documented accepted advisories).
- Mainnet isolation tests pass and no mainnet artifacts modified.
- Documentation updated for any scope decisions.

---

## 4. Critical blockers before Phase 1 code work

1. Benchmark compile error (E0507) in `ug-net`.
2. Formatting non-compliance across all crates.
3. Absence of dependency audit tooling.
4. No fuzz/adversarial test infrastructure.
5. Unresolved scaffold vs compiled identity inconsistency.
6. No wallet/RPC/explorer implementation in Udaya Global (scope decision required).

---

## 5. Explicit non-goals for Phase 1

- No mainnet modification.
- No genesis changes for any existing network.
- No production validator onboarding.
- No claim of production readiness.
- No new consensus features without design review.
