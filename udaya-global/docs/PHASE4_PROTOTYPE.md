# Phase 4 — Isolated Prototype (in progress, gate-constrained)

**Date:** 2026-11-15  
**Scope:** `udaya-global/` only (isolated). **No changes to existing Udaya mainnet/genesis/consensus/emission/supply/history/production.**  
**Gate status:** **NO-GO** for release/live validators (mandatory preconditions from Phase 3 not fully satisfied: no mechanized proof-grade verification, no independent formal/security review/audit, no spec-traced production implementation). Phase 4 work proceeds as **research prototype engineering only**, per explicit direction, with isolation and safety constraints enforced.

## 1. Isolation and protection controls
- Distinct experimental identity: chain `uadc-experimental-0001`, network magic `[0x55,0x41,0x44,0x43]` ("UADC"), default P2P port `19799`, default datadir `data/udaya-global-experimental`, local-only defaults, DNS seeds disabled by default.
- **Refusal of protected mainnet:** helpers reject protected mainnet name and protected magic `[0xBF,0x59,0x1A,0xE7]` (see `udaya-global/src/core/src/consensus.rs`, `udaya-global/src/p2p/src/lib.rs`, prototype `ensure_not_protected_magic`).
- No balance/history transfer, no connection to existing mainnet peers, no modification of protected mainnet artifacts.

## 2. Prototype artifacts delivered
### Rust scaffolding (isolation-hardened, not build-verified)
- `udaya-global/src/core/src/{consensus.rs,lib.rs,types.rs}`: experimental-only consensus/params/identity scaffold, mainnet-refusal guards, placeholder experimental genesis marker (not a verified genesis).
- `udaya-global/src/p2p/src/lib.rs`: isolated P2P config with experimental magic/port/local bind, DNS seeds off, `validate_isolation()` refusing protected magic.
- Existing crate/workspace scaffolds remain present (`udaya-global/Cargo.toml`, `src/crates/*`, `src/storage`, etc.) as **unverified stubs**; workspace completeness/build not validated.

### Spec-traced reference core (executable)
- `udaya-global/prototype/uadc_core/__init__.py`: reference implementation of finalized candidate 2-chain rules (PREPARE/PRECOMMIT, `q=floor(2n/3)+1`, `safeNode = extends(B,locked) OR justify.round>locked.round`, lock/highQC update on QC1, 2-chain commit on `QC2(B,r)` + certified child `QC1(B',r+1)`, vote uniqueness) with isolation guards. Trace: `uadc-research/phase3/UADC_BFT_SPEC_FINALIZED.md`, `uadc-research/FORMAL_CONSENSUS_MODEL.md`.
- `udaya-global/prototype/run_tests.py`: spec-trace tests (isolation refusal, quorum, happy-path commit n=4, safe-node conflict rejection, higher-justify exception, vote uniqueness).

## 3. Evidence (executed)
- **Prototype tests:** PASS `13/13` (Python 3.13.17, 2026-11-15). Log: `test-results/phase4/prototype_core_tests.log`.
- **Rust build/test:** **NOT executed** — `cargo`/`rustc` unavailable in environment; `udaya-global` Rust workspace not build-verified or spec-traced to implementation.

## 4. Limitations / gaps (blocking progression beyond prototype)
- No mechanized proof-grade verification or exhaustive checking; no independent formal/security review/audit; prior evidence bounded/abstracted only.
- Reference core is a **model-level prototype**, not a networked, cryptographically complete, persistent, or production consensus implementation (crypto/VRF/aggregation, networking, storage/WAL, membership/epochs/weights, execution/fees, slashing/evidence, ops not implemented/verified).
- Rust scaffolding unbuilt/unverified; no spec-to-Rust traceability tests; no integration/adversarial/perf/benchmark campaigns; no reproducible build/SBOM/vuln attestation.

## 5. Status
Phase 4 **started (isolated prototype scaffolding + spec-traced reference core with passing unit checks)**. **Live validators, production release, and mainnet interaction remain NOT PERMITTED** until all mandatory gates in `documentation/RELEASE_READINESS.md` are satisfied with reviewed evidence.
