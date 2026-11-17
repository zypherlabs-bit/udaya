# Udaya Global — Technical Decisions (Phase 0.5)

## Storage: filesystem vs RocksDB
- **Decision:** retain the current filesystem-backed `BlockStore` with atomic fsync+rename for Phase 0.5/1.
- **Rationale:** fewer native dependencies, deterministic behaviour, easier recovery testing, and matches the compiled workspace. RocksDB remains an option for later performance work but is not required for correctness.
- **Migration trigger:** measured I/O bottleneck or need for column-family iteration/state-sync.

## Wallet architecture
- **Decision:** no wallet in the compiled workspace yet. Wallet functionality stays outside Udaya Global scope until RPC/key-management design is reviewed. Validator seed files (`validator.key`) remain the only key material.
- **Next step:** design HD wallet + address derivation in a separate crate with reviewed threat model.

## RPC architecture
- **Decision:** no RPC server in Phase 0.5. Node remains CLI/event-loop driven. Future RPC should be a separate crate with authenticated endpoints and rate limits.

## Explorer architecture
- **Decision:** defer. Explorer can consume a future read-only API; no implementation now.

## Light client architecture
- **Decision:** defer. Requires state-sync and compact proofs not yet implemented.

## State synchronization
- **Decision:** block replay from peers remains the only sync mode. State snapshots are persisted but not transferred.

## Pruning and backups
- **Decision:** no pruning in Phase 0.5. Backups are operator-managed copies of the data directory. Atomic writes + fsync remain mandatory.

## Committee rotation
- **Decision:** fixed validator set at genesis. No rotation until a reviewed design for dynamic membership exists.

## VRF proposer selection
- **Decision:** deterministic round-robin/height-based proposer selection remains. VRF deferred.

## Validator slashing and evidence handling
- **Decision:** double-sign evidence data structures exist; automatic slashing/enforcement deferred pending reviewed economics.
