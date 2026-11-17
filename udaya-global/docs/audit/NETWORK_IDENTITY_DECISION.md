# Udaya Global — Network Identity Decision

## Authoritative identity (compiled workspace)
- Chain ID prefix: `udaya-global` (default `udaya-global-devnet-1`)
- Network magic: `0x55 0x47 0x4C 0x01` (`GLOBAL_MAGIC`)
- P2P port: `29798` (`DEVNET_P2P_PORT`)
- Address HRP: `ugdev`
- Supply cap: 21,000,000 UDYA (2,100,000,000,000,000 sats)

## Excluded scaffold identity (obsolete)
- `udaya-global/src` and `crates/udaya-*` use port `19799` and an older experimental identity.
- These files are not compiled into the workspace and are archived under `archive/excluded-scaffold-2026-11-16/`.
- They are historical research material only and must not be used for production.

## Mainnet isolation
- Protected magic `0xBF 0x59 0x1A 0xE7`, chain IDs `mainnet`/`udaya-mainnet`/`udaya`, and port `9798` are explicitly refused by `ChainParams::validate()`.
- Tests: `default_devnet_is_valid_and_isolated`, `protected_mainnet_identity_is_refused`.

## Decision
The compiled workspace identity (port 29798) is authoritative. The scaffold (port 19799) is archived and treated as obsolete.
