# Udaya Genesis Block Verification

## Genesis Block
The Udaya genesis block is the foundation of the blockchain. All nodes verify it at initialization.

## Verification Steps
1. Download genesis-manifest.json from https://github.com/Udaya/Udaya/releases
2. Verify the genesis block hash matches the published value
3. Verify the merkle root matches the coinbase transaction
4. Verify the proof-of-work is valid (SHA-256d)
5. Verify the timestamp is reasonable (±2 hours of network launch)

## Verification Script
```bash
# Download genesis block
wget https://github.com/Udaya/Udaya/releases/latest/download/genesis-block-mainnet.dat

# Run verification
udayad verify-genesis genesis-block-mainnet.dat

# Expected output:
# Genesis block verified successfully
# Block hash: 000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f
# Timestamp: 2026-05-28 12:00:00 UTC
```

## Programmatic Verification
```javascript
const Udaya = require('Udaya-js');
const fs = require('fs');
const genesis = fs.readFileSync('genesis-block-mainnet.dat');
const verified = Udaya.verifyGenesisBlock(genesis);
console.log('Genesis verified:', verified);
```

## Exchanges
Exchanges should verify the genesis block before integrating UDYA to ensure chain authenticity.