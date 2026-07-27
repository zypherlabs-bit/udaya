# Udaya API Documentation

Comprehensive reference for the Udaya node JSON-RPC and REST APIs.

## Base URL

```
http://127.0.0.1:8332
```

## Authentication

All RPC requests require HTTP Basic Authentication. Configure credentials via environment variables:

```bash
export RPC_USER=your_secure_username
export RPC_PASSWORD=your_strong_random_password
```

## Content-Type

```
Content-Type: application/json
```

---

## JSON-RPC API

Udaya implements a Bitcoin Core-compatible JSON-RPC interface.

### Request Format

```json
{
  "jsonrpc": "2.0",
  "method": "METHOD_NAME",
  "params": [],
  "id": 1
}
```

### Response Format

```json
{
  "jsonrpc": "2.0",
  "result": {},
  "error": null,
  "id": 1
}
```

---

## Blockchain Methods

### getblockchaininfo

Returns information about the current blockchain state.

**Parameters**: None

**Response**:
```json
{
  "chain": "mainnet",
  "blocks": 1000,
  "headers": 1000,
  "bestblockhash": "0000000000000000000...",
  "difficulty": 1.0,
  "mediantime": 1234567890,
  "chainwork": "0000000000000000000...",
  "size_on_disk": 1500000000,
  "pruned": false,
  "softforks": []
}
```

### getblockcount

Returns the current block height.

**Parameters**: None

**Response**: `1000` (integer)

### getblockhash

Returns the hash of a block at a given height.

**Parameters**:
1. `height` (number, required) - Block height

**Response**: `"0000000000000000000..."` (block hash string)

**Example**:
```bash
curl -u $RPC_USER:$RPC_PASSWORD -X POST http://127.0.0.1:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"getblockhash","params":[1000],"id":1}'
```

### getblock

Returns detailed information about a block.

**Parameters**:
1. `blockhash` (string, required) - Block hash
2. `verbosity` (number, optional, default=1) - Detail level (0=hex, 1=txids, 2=full)

**Response** (verbosity=1):
```json
{
  "hash": "0000000000000000000...",
  "confirmations": 100,
  "height": 1000,
  "version": 1,
  "versionHex": "00000001",
  "merkleroot": "0000000000000000000...",
  "time": 1234567890,
  "mediantime": 1234567800,
  "nonce": 12345,
  "bits": "1d00ffff",
  "difficulty": 1.0,
  "chainwork": "0000000000000000000...",
  "previousblockhash": "0000000000000000000...",
  "tx": ["txid1", "txid2", ...],
  "size": 500,
  "weight": 2000
}
```

### gettransaction

Returns transaction details by txid.

**Parameters**:
1. `txid` (string, required) - Transaction ID
2. `verbose` (boolean, optional, default=true) - Return detailed info

**Response**:
```json
{
  "txid": "0000000000000000000...",
  "version": 1,
  "locktime": 0,
  "size": 250,
  "vsize": 150,
  "hex": "0100000001...",
  "confirmations": 10,
  "blockhash": "0000000000000000000...",
  "blocktime": 1234567890,
  "time": 1234567880,
  "vin": [],
  "vout": []
}
```

### gettxout

Returns information about an unspent transaction output.

**Parameters**:
1. `txid` (string, required) - Transaction ID
2. `vout` (number, required) - Output index
3. `include_mempool` (boolean, optional, default=true) - Include mempool

**Response**:
```json
{
  "bestblock": "0000000000000000000...",
  "confirmations": 50,
  "value": 1.5,
  "scriptPubKey": {
    "asm": "",
    "hex": "76a914...",
    "reqSigs": 1,
    "type": "pubkeyhash",
    "addresses": ["U9x..."]
  },
  "coinbase": false
}
```

---

## Mempool Methods

### getmempoolinfo

Returns mempool statistics.

**Parameters**: None

**Response**:
```json
{
  "size": 1000,
  "bytes": 500000,
  "usage": 600000,
  "maxmempool": 300000000,
  "mempoolminfee": 0.00001,
  "minrelaytxfee": 0.00001
}
```

### getmempoolentry

Returns mempool entry details.

**Parameters**:
1. `txid` (string, required) - Transaction ID

**Response**:
```json
{
  "size": 250,
  "fee": 1000,
  "modifiedfee": 1000,
  "time": 1234567890,
  "height": 1000,
  "descendantcount": 5,
  "descendantsize": 1250,
  "ancestorcount": 2,
  "ancestorsize": 500,
  "depends": ["parent_txid"],
  "spentby": ["child_txid"]
}
```

---

## Wallet Methods

### getbalance

Returns the wallet balance.

**Parameters**: None

**Response**: `150.5` (balance in UDY)

### getnewaddress

Generates a new receiving address.

**Parameters**: None

**Response**: `"U9xQ..."` (address string)

### sendtoaddress

Sends UDY to an address.

**Parameters**:
1. `address` (string, required) - Destination address
2. `amount` (number, required) - Amount in UDY
3. `comment` (string, optional) - Comment
4. `comment_to` (string, optional) - Comment for recipient

**Response**: `"txid"` (transaction ID)

**Example**:
```bash
curl -u username:password -X POST http://127.0.0.1:8332 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"sendtoaddress",
    "params":["U9xQ...", 1.5],
    "id":1
  }'
```

### listunspent

Lists unspent transaction outputs.

**Parameters**:
1. `minconf` (number, optional) - Minimum confirmations
2. `maxconf` (number, optional) - Maximum confirmations
3. `addresses` (array, optional) - Filter by addresses

**Response**:
```json
[
  {
    "txid": "0000000000000000000...",
    "vout": 0,
    "address": "U9xQ...",
    "label": "",
    "scriptPubKey": "76a914...",
    "amount": 1.5,
    "confirmations": 50,
    "spendable": true,
    "safe": true
  }
]
```

### listtransactions

Lists recent transactions.

**Parameters**:
1. `count` (number, optional, default=10) - Number of transactions
2. `skip` (number, optional, default=0) - Skip transactions

**Response**:
```json
[
  {
    "txid": "0000000000000000000...",
    "time": 1234567890,
    "timereceived": 1234567880,
    "fee": -1000,
    "amount": 1.5,
    "confirmations": 10,
    "blockheight": 1000,
    "category": "receive"
  }
]
```

---

## Mining Methods

### getmininginfo

Returns mining statistics.

**Parameters**: None

**Response**:
```json
{
  "blocks": 1000,
  "currentblocksize": 0,
  "currentblocktx": 0,
  "difficulty": 1.0,
  "errors": "",
  "genproclimit": 1,
  "networkhashps": 0.0,
  "pooledtx": 500,
  "testnet": false,
  "chain": "mainnet"
}
```

### getblocktemplate

Returns a block template for mining.

**Parameters**:
1. `template_request` (object, optional) - Template options

**Response**:
```json
{
  "capabilities": ["proposal"],
  "version": 1,
  "previousblockhash": "0000000000000000000...",
  "transactions": [],
  "coinbasevalue": 5000000000,
  "coinbaseaux": {"flags": "mined by Udaya"},
  "target": "00000000ffff0000000000000000000000000000000000000000000000000000",
  "mintime": 1234567890,
  "mutable": ["time", "transactions", "prevblock"],
  "noncerange": "00000000ffffffff",
  "sigoplimit": 20000,
  "sizelimit": 1000000,
  "weightlimit": 4000000,
  "curtime": 1234567890,
  "bits": "1d00ffff",
  "height": 1001
}
```

### submitblock

Submits a mined block.

**Parameters**:
1. `hexdata` (string, required) - Block hex string
2. `dummy` (string, optional) - Dummy parameter for compatibility

**Response**: `null`

---

## Network Methods

### getpeerinfo

Returns connected peer information.

**Parameters**: None

**Response**:
```json
[
  {
    "id": 1,
    "addr": "192.168.1.100:9798",
    "version": 70016,
    "subver": "/Udaya:1.0.0/",
    "startingheight": 1000,
    "conntime": 1234567890,
    "lastsend": 1234567880,
    "lastrecv": 1234567880,
    "bytessent": 1024,
    "bytesrecv": 2048,
    "pingtime": 0.5,
    "relaytxes": true,
    "inbound": false
  }
]
```

### getnetworkinfo

Returns network configuration.

**Parameters**: None

**Response**:
```json
{
  "version": "1.0.0",
  "subversion": "/Udaya:1.0.0/",
  "protocolversion": 70016,
  "localservices": "000000000000040d",
  "localrelay": true,
  "timeoffset": 0,
  "networkactive": true,
  "connections": 8,
  "relayfee": 0.00001,
  "incrementalfee": 0.00001
}
```

### addnode

Adds or removes a peer.

**Parameters**:
1. `node` (string, required) - Node address
2. `command` (string, required) - Command: "add", "remove", "onetry"

**Response**: `null`

---

## Utility Methods

### ping

Pings the server (used for connectivity tests).

**Parameters**: None

**Response**: `"pong"`

### getinfo

Returns general node information.

**Parameters**: None

**Response**:
```json
{
  "version": "1.0.0",
  "protocolversion": 70016,
  "walletversion": 169900,
  "balance": 150.5,
  "blocks": 1000,
  "timeoffset": 0,
  "connections": 8,
  "difficulty": 1.0,
  "testnet": false,
  "keypoololdest": 0,
  "keypoolsize": 100,
  "paytxfee": 0.0,
  "relayfee": 0.00001,
  "errors": "",
  "mempooltxs": 500,
  "uptime": 86400
}
```

### stop

Initiates graceful node shutdown.

**Parameters**: None

**Response**: `"Udaya server stopping"`

---

## REST API Endpoints

In addition to JSON-RPC, the following REST endpoints are available:

### Health Checks

```
GET /health
GET /healthz
GET /readyz
```

**Response**:
```json
{
  "status": "healthy",
  "service": "udayad",
  "version": "1.0.0",
  "timestamp": "2026-07-26T14:30:00Z"
}
```

### Metrics

```
GET /metrics
```

Returns Prometheus-format metrics.

---

## Error Codes

| Code | Meaning |
|------|---------|
| -32700 | Parse error |
| -32600 | Invalid request |
| -32601 | Method not found |
| -32602 | Invalid parameters |
| -32603 | Internal error |
| -1 | General error |
| -4 | Transaction error |
| -5 | Block/transaction not found |
| -8 | Invalid parameter |
| -25 | Block submission rejected |

---

## Rate Limiting

RPC endpoints implement rate limiting to prevent abuse. Limits are per-IP and per-user.

**Default Limits**:
- 100 requests per minute
- 10 concurrent requests

**Rate Limit Headers** (included in responses):
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1439414400
```

---

## Examples

### Using curl

```bash
# Get blockchain info
curl -u username:password -X POST http://127.0.0.1:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"getblockchaininfo","id":1}'

# Generate new address
curl -u username:password -X POST http://127.0.0.1:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"getnewaddress","id":1}'

# Send transaction
curl -u username:password -X POST http://127.0.0.1:8332 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"sendtoaddress",
    "params":["U9xQ...", 1.0],
    "id":1
  }'
```

### Using Python

```python
import requests
from requests.auth import HTTPBasicAuth

auth = HTTPBasicAuth(os.getenv('RPC_USER'), os.getenv('RPC_PASSWORD'))
url = 'http://127.0.0.1:8332'

# Get blockchain info
response = requests.post(url, json={
    "jsonrpc": "2.0",
    "method": "getblockchaininfo",
    "id": 1
}, auth=auth)

print(response.json())
```

### Using Node.js

```javascript
const axios = require('axios');

const auth = {
  username: process.env.RPC_USER,
  password: process.env.RPC_PASSWORD
};

async function getBlockchainInfo() {
  const response = await axios.post('http://127.0.0.1:8332', {
    jsonrpc: '2.0',
    method: 'getblockchaininfo',
    id: 1
  }, { auth });

  console.log(response.data.result);
}
```

---

## Troubleshooting

### Common Issues

1. **401 Unauthorized**: Check username/password in config
2. **403 Forbidden**: Check CORS settings
3. **429 Too Many Requests**: Implement backoff/retry logic
4. **500 Internal Server Error**: Check node logs

### Debug Mode

Enable debug logging for detailed RPC logs:

```toml
[logging]
level = "debug"
```

---

*API Version: 1.0.0*
*Last Updated: 2026-07-26*