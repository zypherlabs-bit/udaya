# Udaya Global — Dependency Audit Results

## Tools
- cargo-audit 0.23.0: installed and operational.
- cargo-deny 0.20.2: installed and operational.
- SBOM: cargo-cyclonedx 0.5.10, per-crate `.cdx.json` generated.

## Results
- `cargo audit`: PASS (0 vulnerabilities; 197 dependencies scanned).
- `cargo deny check`: PASS (advisories ok, bans ok, licenses ok, sources ok).

## Configuration
- `deny.toml`
- `audit.toml`
- `scripts/generate-sbom.ps1`

## Notes
- No dependency upgrades performed. Unused workspace dependencies removed (anyhow, bincode, env_logger, parking_lot, log).
- deny.toml uses SPDX allow-list including Unicode-3.0.
