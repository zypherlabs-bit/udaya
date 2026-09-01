#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TRUST_DIR="$PROJECT_ROOT/docs/trust-center"

mkdir -p "$TRUST_DIR"/{security,reports,genesis,keys}

echo "=== Publishing Trust Center ==="
echo "Timestamp: $(date -u +%Y-%m-%dT%H:%M:%SZ)"

if [ -f "$PROJECT_ROOT/EXTERNAL_SECURITY_REVIEW_SUMMARY.md" ]; then
    cp "$PROJECT_ROOT/EXTERNAL_SECURITY_REVIEW_SUMMARY.md" "$TRUST_DIR/security/external-security-review.md"
fi

if [ -f "$PROJECT_ROOT/SECURITY_AUDIT_REPORT.md" ]; then
    cp "$PROJECT_ROOT/SECURITY_AUDIT_REPORT.md" "$TRUST_DIR/security/audit-report.md"
fi

if [ -f "$PROJECT_ROOT/CRYPTOGRAPHIC_SECURITY_ANALYSIS.md" ]; then
    cp "$PROJECT_ROOT/CRYPTOGRAPHIC_SECURITY_ANALYSIS.md" "$TRUST_DIR/security/crypto-analysis.md"
fi

if [ -f "$PROJECT_ROOT/FINAL_CERTIFICATION_REPORT.md" ]; then
    cp "$PROJECT_ROOT/FINAL_CERTIFICATION_REPORT.md" "$TRUST_DIR/reports/certification.md"
fi

if [ -f "$PROJECT_ROOT/FINAL_PUBLICATION_REPORT.md" ]; then
    cp "$PROJECT_ROOT/FINAL_PUBLICATION_REPORT.md" "$TRUST_DIR/reports/publication.md"
fi

if [ -f "$PROJECT_ROOT/docs/genesis/genesis-manifest.json" ]; then
    cp "$PROJECT_ROOT/docs/genesis/genesis-manifest.json" "$TRUST_DIR/genesis/genesis-manifest.json"
fi

if [ -f "$PROJECT_ROOT/PHASE15_MONITORING_INCIDENT_RESPONSE_REPORT.md" ]; then
    cp "$PROJECT_ROOT/PHASE15_MONITORING_INCIDENT_RESPONSE_REPORT.md" "$TRUST_DIR/reports/monitoring-incident-response.md"
fi

cat > "$TRUST_DIR/README.md" <<EOF
# Udaya Trust Center

This directory contains verifiable trust artifacts for Udaya mainnet.

## Contents
- security/ - Security audits, reviews, and cryptographic analysis
- reports/ - Certification, publication, and monitoring reports
- genesis/ - Genesis block manifest and verification data
- keys/ - Public verification keys and signatures

## Verification
All files are checksummed and versioned. Verify integrity before use.
EOF

echo "Trust center published."
