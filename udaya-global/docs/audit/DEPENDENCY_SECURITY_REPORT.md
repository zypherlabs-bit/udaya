# Udaya Global — Dependency and Security Report (Phase 0)

**Scope:** `udaya-global/` workspace dependencies and security posture.  
**Method:** Manifest inspection + build/test/clippy baseline. No dependency changes made.

---

## 1. Workspace dependencies (udaya-global/Cargo.toml)

Core/runtime:
- `tokio 1.35` (full features)
- `serde 1.0`, `serde_json 1.0`
- `clap 4.4`
- `thiserror 1.0`, `anyhow 1.0` (unused at workspace scope)
- `log 0.4`, `env_logger 0.10` (unused at workspace scope)
- `hex 0.4`, `rand 0.8`, `sha2 0.10`
- `parking_lot 0.12` (unused at workspace scope)
- `bincode 1.3` (unused at workspace scope)
- `tempfile 3.8`, `criterion 0.5` (dev)

Cryptography/network:
- `ed25519-dalek 2` (rand_core, zeroize)
- `x25519-dalek 2` (static_secrets)
- `snow 0.10`
- `bech32 0.11`
- `zeroize 1`

Utilities:
- `chrono` not in global workspace; `toml` not in global workspace
- `serde`/`serde_json` as above

Crate-local:
- `ug-core` adds `bech32`, `ed25519-dalek`, `hex`, `log`, `rand`, `serde`, `serde_json`, `sha2`, `thiserror`, `zeroize`.
- `ug-net` adds `snow`, `tokio`, `x25519-dalek`.
- `ug-node` adds `clap`, `tokio`, `hex`, `rand`, `serde`, `serde_json`, `thiserror`.

---

## 2. Dependency hygiene findings

- Unused workspace dependencies: `anyhow`, `bincode`, `env_logger`, `parking_lot`.
- Unused crate dependency: `log` in `ug-core`.
- Unused workspace.package field: `description`.
- No `Cargo.lock` drift check was performed beyond successful build.
- No dependency pinning beyond Cargo's normal resolution; `Cargo.lock` is committed.

---

## 3. Security tooling

| Tool | Status |
|---|---|
| `cargo audit` | Not installed |
| `cargo deny` | Not installed |
| `cargo outdated` | Not installed |
| `deny.toml` in udaya-global | Absent |
| SBOM | Absent |
| Reproducible build attestation | Absent |

Root Udaya has a `deny.toml`, but it governs the mainnet workspace and was not applied to Udaya Global.

---

## 4. Security-relevant implementation observations

Positive:
- Strict decoders with trailing-byte rejection.
- Frame size limits enforced before allocation.
- Per-peer rate limits; overflow is a hard error.
- `serde(deny_unknown_fields)` on node config.
- Atomic file writes with fsync + rename.
- Isolation guards refusing protected mainnet magic/chain IDs/port.
- Key material zeroization in `ed25519-dalek` features.
- Debug output avoids printing secret material (tested).

Gaps:
- No fuzzing or property-based testing.
- No adversarial network tests (eclipse, partition, replay, flooding).
- No formal security review of the consensus implementation.
- No key rotation, threshold signing, or HSM integration.
- No SBOM/vulnerability attestation.
- Benchmark code does not compile under clippy (E0507), so performance claims are unverified.
- Formatting not enforced, increasing review noise.

---

## 5. Recommendations (not implemented)

1. Add `deny.toml` to `udaya-global/` mirroring license/advisory/source policies.
2. Install `cargo-audit`/`cargo-deny` in CI (not in this audit) and run against `Cargo.lock`.
3. Remove unused dependencies to reduce attack surface.
4. Add fuzz targets for codec, tx, block, and consensus message decoding.
5. Add property-based tests for transaction validation and state transitions.
6. Generate and publish an SBOM for release artifacts.
7. Establish reproducible build instructions and verify in CI.
8. Fix the `ug-net` benchmark compile error before relying on performance numbers.
