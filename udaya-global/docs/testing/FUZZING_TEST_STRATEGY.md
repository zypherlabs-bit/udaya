# Udaya Global — Fuzzing and Property Test Strategy

## Tools
- `proptest` for property-based tests (stable toolchain).
- `cargo-fuzz`/libFuzzer recommended for nightly CI once a nightly toolchain is available.

## Current coverage
- Transaction codec: `Transaction::decode` must never panic on arbitrary bytes; encode/decode structural roundtrip.
- Network messages: `Message::decode` must never panic on arbitrary bytes.
- Transaction validation: existing unit tests cover signature, conservation, duplicate inputs, fee, zero outputs, replay, chain binding.
- UTXO conservation, duplicate transaction, double-spend, mempool conflict: covered by `ug-core`/`ug-node` tests.
- Storage corruption and restart/recovery: `BlockStore` atomic write tests.
- Network partition simulations: cluster integration test.

## Reproducible seeds
- proptest default RNG seed is recorded by proptest in `proptest-regressions/` when a failure occurs.
- CI runs with `PROPTEST_CASES=256` for deeper coverage.

## Next steps
- Add cargo-fuzz targets for transaction codec, network framing, and handshake parsing.
- Add malformed network message tests at the node event loop.
- Add long-running soak tests.
