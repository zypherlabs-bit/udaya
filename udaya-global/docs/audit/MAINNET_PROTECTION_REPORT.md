# Udaya Global — Mainnet Protection Report (Phase 0)

**Scope:** Protect the pre-existing Udaya mainnet. Udaya Global must remain fully isolated.  
**Rule:** No modification to existing Udaya mainnet genesis, chain ID, consensus rules, max supply, historical data, mining/emission rules.

---

## 1. Existing Udaya mainnet identity (protected)

Verified from the root Udaya project (`src/core/src/lib.rs`, `config/mainnet/bitfury.conf`, `docs/exchange/chain-metadata.md`):

| Parameter | Protected value |
|---|---|
| Ticker | UDYA |
| Network magic | `0xBF 0x59 0x1A 0xE7` (`0xBF591AE7`) |
| P2P port | 9798 |
| RPC port | 8332 |
| Bech32 HRP | UDYA (mainnet) |
| Max supply | 21,000,000 UDYA (2,100,000,000,000,000 sats) |
| Block time | 600 s |
| Difficulty adjustment | every 2,016 blocks |
| Halving | every 210,000 blocks |
| Initial reward | 50 UDYA |
| Coinbase maturity | 100 blocks |
| Consensus algorithm | SHA-256d PoW |
| Protocol version | 70016 |

Genesis/historical chain data is not present as a single file in this repository snapshot; the mainnet database lives under data directories and is git-ignored. The mainnet configuration and constants above are treated as the protected identity.

---

## 2. Files/modules/configurations associated with existing mainnet

- `src/core/src/lib.rs` — `NETWORK_MAGIC`, `MAX_SUPPLY`, `MAX_SUPPLY_SATS`, emission constants, `MAINNET` name.
- `src/core/src/consensus.rs` — PoW consensus engine and subsidy curve.
- `src/core/src/genesis.rs` — mainnet genesis construction.
- `config/mainnet/*` — mainnet node configuration (magic `0xBF591AE7`, port 9798).
- `config/node*.conf`, `config/bitfury.conf` — mainnet-oriented defaults.
- `docs/exchange/chain-metadata.md` — canonical mainnet metadata.
- `docs/FLUTTER_NODE_ARCHITECTURE.md` — documents mainnet magic.
- `src/wallet/*`, `src/wallet-cli/*` — mainnet wallet stack.

These were **not** modified by this audit.

---

## 3. Udaya Global isolation identity (independent)

Verified from `udaya-global/crates/ug-core/src/lib.rs` and tests:

| Parameter | Udaya Global value |
|---|---|
| Chain ID prefix | `udaya-global` (default `udaya-global-devnet-1`) |
| Network magic | `0x55 0x47 0x4C 0x01` (`GLOBAL_MAGIC`) |
| P2P port | 29798 (`DEVNET_P2P_PORT`) |
| Address HRP | `ugdev` (`DEVNET_ADDRESS_HRP`) |
| Supply cap | 2,100,000,000,000,000 sats (21,000,000 coins) |
| Protocol version | 1 |
| Max block bytes | 4 MiB |
| Max tx bytes | 256 KiB |

Isolation guards (verified by tests):
- `ChainParams::validate()` rejects:
  - chain IDs `mainnet`, `udaya-mainnet`, `udaya`;
  - chain IDs not starting with `udaya-global`;
  - network magic equal to `PROTECTED_MAINNET_MAGIC` (`0xBF 0x59 0x1A 0xE7`);
  - zero magic;
  - P2P port `9798`;
  - zero port.
- Scaffold helpers (`udaya-global/src/...`) additionally refuse protected magic and unknown network names.

---

## 4. Excluded scaffold identity (research-only)

`udaya-global/src/core/src/consensus.rs` defines an even more isolated experimental identity:
- Chain ID: `uadc-experimental-0001`
- Magic: `0x55 0x41 0x44 0x43` ("UADC")
- P2P port: 19799
- RPC port: 28332
- Datadir: `data/udaya-global-experimental`

This scaffold is not compiled into the workspace and is not the production identity of the compiled node. It remains research material.

---

## 5. Mainnet modification evidence

This audit performed **no** writes to:
- root `src/`, `config/`, `data/`, genesis, or mainnet configuration;
- root `Cargo.*`;
- any mainnet database or historical data.

Commands executed were read-only with respect to the mainnet (`cargo build/test/clippy/fmt` inside `udaya-global/`, Python prototype tests). No mainnet node was started. The only files created are under `udaya-global/docs/audit/`.

---

## 6. Isolation strategy for Udaya Global

1. **Distinct identity:** chain ID prefix `udaya-global`, magic `GLOBAL_MAGIC`, port `29798`, HRP `ugdev`. Never reuse mainnet values.
2. **Explicit refusal:** every entry point validates against protected mainnet magic, chain IDs, and port; tests enforce this.
3. **No history import:** no balance, UTXO, block, or peer import from mainnet; genesis allocations are explicit per-network.
4. **No peer connectivity:** default DNS seeds disabled; bootstrap peers are explicit opt-in; magic mismatch fails handshake.
5. **No shared datadir:** distinct data directories; `verify_data_dir`/`read_disk_status` operate on Udaya Global stores only.
6. **Supply independence:** Udaya Global enforces its own cap (`MAX_SUPPLY_SATS`) and never mints beyond it; no coupling to mainnet emission curve.
7. **Build isolation:** Udaya Global is a separate workspace; mainnet crates are not dependencies.
8. **Documentation:** isolation invariants documented in crate docs and tests; any change to identity requires review.

---

## 7. Residual risk

- The excluded `src/` scaffold uses a different experimental identity (port 19799) than the compiled workspace (port 29798). This inconsistency must be resolved or the scaffold archived before production.
- Mainnet identity constants are duplicated in scaffold and compiled crates; a single source of truth for protected values is recommended.
- No automated check prevents a future PR from changing `GLOBAL_MAGIC` to the protected value beyond unit tests; consider a CI invariant test and code review gate.
