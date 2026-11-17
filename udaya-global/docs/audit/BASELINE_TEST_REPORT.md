# Udaya Global — Baseline Test Report (Phase 0)

**Date:** 2026-11-16  
**Environment:** Windows 11, `rustc 1.100.0`, `cargo 1.100.0`, host `x86_64-pc-windows-gnu`, MSYS2 MinGW64 (`dlltool`), Python 3.13.

---

## 1. Compilation

Command: `cargo build --workspace` (in `udaya-global/`)

Result: **PASS**  
Duration: ~4.15 s (incremental cache present).  
Warnings:
- Workspace manifest: unused workspace dependencies `anyhow`, `bincode`, `env_logger`, `parking_lot`; unused field `workspace.package.description` (5 warnings).
- `ug-core` manifest: unused dependency `log` (1 warning).

No errors.

---

## 2. Tests

Command: `cargo test --workspace`

Result: **PASS — 129 tests, 0 failures, 0 ignored**

Breakdown:
| Crate/target | Tests |
|---|---:|
| `ug-consensus` lib | 34 |
| `ug-core` lib | 55 |
| `ug-net` lib | 27 |
| `ug-node` lib | 11 |
| `ug-node` cluster integration | 2 |
| Doc tests (all crates) | 0 |
| **Total** | **129** |

Command: `python prototype/run_tests.py`

Result: **PASS — 13/13**  
Checks: isolation refusal (protected magic, chain ID), quorum sizes (n=4, n=7), happy-path commit for 4 nodes, safe-node conflict rejection, higher-justify exception, vote uniqueness.

---

## 3. Clippy

Command: `cargo clippy --workspace` (default targets)

Result: **PASS with warnings — 0 errors, 23 warning lines**  
Lint highlights:
- `ug-core`: unused dependency `log`; `sort_by_key` suggestion.
- `ug-consensus`: too many arguments (9/7); `if` with identical blocks; unnecessary closure for `Option::None`; match for single pattern; clone on `Copy` types; large size difference between enum variants; redundant redefinition of `entries`.
- `ug-net`: large size difference between variants; clone on `Copy`.
- `ug-node`: field reassignment with `Default::default()` (2); `pubkey` never used.
- Workspace manifest warnings as above.

Command: `cargo clippy --workspace --all-targets` (includes benches)

Result: **FAIL**  
Error: `ug-net` bench (`crates/ug-net/benches/net.rs:126`) — `E0507: cannot move out of server, a captured variable in an FnMut closure`. The bench captures `NetworkConfig` (non-`Copy`) by move inside `b.iter`. This is a real compile error in benchmark code, not a lint.

---

## 4. Formatting

Command: `cargo fmt --check`

Result: **FAIL** — many formatting differences across `ug-consensus`, `ug-core`, `ug-net`, `ug-node` (line-wrapping and import ordering). No code behavior change required; indicates formatting was not enforced.

---

## 5. Dependency audit

Tools: `cargo audit`, `cargo deny`, `cargo outdated` — **not installed** in this environment. No automated CVE scan was performed. See `DEPENDENCY_SECURITY_REPORT.md`.

---

## 6. Compiler warnings inventory

- Unused workspace dependencies: `anyhow`, `bincode`, `env_logger`, `parking_lot`.
- Unused crate dependency: `log` in `ug-core`.
- Unused workspace.package field: `description`.
- Clippy warnings listed above (non-blocking).

---

## 7. Incomplete modules verified

- Excluded `src/` scaffold and `crates/udaya-*` stubs are not compiled or tested.
- No wallet/RPC/explorer/fuzz/soak tests exist in the workspace.

---

## 8. Baseline conclusion

The compiled Udaya Global workspace builds and passes its unit/integration tests. Quality gates are not fully clean: clippy fails on benchmark code, formatting is non-compliant, and dependency auditing tooling is absent. These are recorded as blockers for Phase 1 readiness but do not alter mainnet artifacts.
