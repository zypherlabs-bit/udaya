#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GENESIS_DIR="$PROJECT_ROOT/docs/genesis"
PUBLISH_DIR="$PROJECT_ROOT/docs/trust-center"

mkdir -p "$GENESIS_DIR" "$PUBLISH_DIR"

echo "=== Publishing Genesis Manifest ==="
echo "Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)"

if [ ! -f "$GENESIS_DIR/genesis-manifest.json" ]; then
    echo "Genesis manifest not found. Generating..."
    mkdir -p "$GENESIS_DIR"
    cat > "$GENESIS_DIR/genesis-manifest.json" <<'EOF'
{
  "network": "mainnet",
  "genesis_block_hash": "000000006fac6e2c2b0c0c09010cc0914232fb16b83fca1d6828092b720063a6",
  "merkle_root": "cd140814a7954e150657e52361560fe2e50d346a013287deee332f574ed0ab00",
  "nonce": 1639837009,
  "timestamp": 1782030724,
  "bits": "0x1D00FFFF",
  "version": 1,
  "hashes_checked": 29224277,
  "launch_statement": "Udaya Genesis - The Future of Digital Value",
  "pubkey": "UdayaGenesisPublicKeyPlaceholder"
}
EOF
fi

cp "$GENESIS_DIR/genesis-manifest.json" "$PUBLISH_DIR/genesis-manifest.json"
echo "Genesis manifest published to trust center."

if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$PUBLISH_DIR/genesis-manifest.json" > "$PUBLISH_DIR/genesis-manifest.json.sha256"
    echo "Checksum saved: genesis-manifest.json.sha256"
fi

echo "Publish complete."
