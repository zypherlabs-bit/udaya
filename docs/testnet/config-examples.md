# Udaya Testnet - Configuration Examples

## Minimal Node

Minimal configuration for a lightweight testnet node:

```toml
[network]
listen_port = 19798
max_peers = 8
enable_upnp = false

[storage]
data_dir = "/var/lib/udaya/data"
prune_blocks = false
db_cache_size_mb = 256

[consensus]
network = "testnet"
min_tx_fee = 100
max_block_size = 1000000

[wallet]
enable = true
wallet_file = "wallet.dat"
default_fee_rate = 50

[rpc]
enable = true
listen_addr = "127.0.0.1"
listen_port = 18332
username = "udaya"
password = "change_me"
enable_ws = false

[logging]
level = "info"
enable_json = false
```

## Mining Node

Configuration for a mining node:

```toml
[network]
listen_port = 19798
max_peers = 50
enable_upnp = false

[storage]
data_dir = "/var/lib/udaya/data"
prune_blocks = false
db_cache_size_mb = 2048

[consensus]
network = "testnet"
min_tx_fee = 100
max_block_size = 1000000

[mining]
enable = true
mine_on_startup = true
num_miner_threads = 4
coinbase_address = "YOUR_UDYA_ADDRESS"

[wallet]
enable = true
wallet_file = "miner-wallet.dat"
default_fee_rate = 50

[rpc]
enable = true
listen_addr = "127.0.0.1"
listen_port = 18332
username = "udaya"
password = "change_me"
enable_ws = false

[logging]
level = "info"
file = "/var/log/udaya/miner.log"
enable_json = true
```

## Seed Node

Configuration for a public seed node:

```toml
[network]
listen_port = 19798
external_ip = "YOUR_PUBLIC_IP"
max_peers = 500
enable_upnp = false
preferred_peers = [
    "seed2.testnet.udaya.net:19798",
    "seed3.testnet.udaya.net:19798",
]

[storage]
data_dir = "/var/lib/udaya/seed"
prune_blocks = false
prune_target_gb = 100
db_cache_size_mb = 4096

[consensus]
network = "testnet"
min_tx_fee = 100
max_block_size = 1000000

[mining]
enable = false
mine_on_startup = false

[wallet]
enable = false

[rpc]
enable = true
listen_addr = "127.0.0.1"
listen_port = 18332
username = "udaya"
password = "change_me"
enable_ws = false

[logging]
level = "info"
file = "/var/log/udaya/seed.log"
enable_json = true
```

## Explorer Node

Configuration for a block explorer node:

```toml
[network]
listen_port = 19798
max_peers = 50
enable_upnp = false

[storage]
data_dir = "/var/lib/udaya/explorer"
prune_blocks = false
db_cache_size_mb = 4096

[consensus]
network = "testnet"
min_tx_fee = 100
max_block_size = 1000000

[mining]
enable = false

[wallet]
enable = false

[rpc]
enable = true
listen_addr = "0.0.0.0"
listen_port = 18332
username = "explorer"
password = "change_me"
enable_ws = true
ws_port = 18333
cors_domains = ["https://explorer.testnet.udaya.net"]

[logging]
level = "info"
file = "/var/log/udaya/explorer.log"
enable_json = true
```

## Docker (Optional)

```yaml
version: '3.8'
services:
  udaya-node:
    image: udayafoundation/udaya:latest
    ports:
      - "19798:19798"
      - "18332:18332"
    volumes:
      - ./data:/var/lib/udaya
      - ./udaya.conf:/etc/udaya/udaya.conf
    environment:
      - RPC_USER=udaya
      - RPC_PASSWORD=change_me
    restart: unless-stopped
```
