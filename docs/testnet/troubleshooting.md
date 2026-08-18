# Udaya Testnet - Troubleshooting Guide

## Common Issues

### Node won't start

**Symptoms**: `udayad start` exits immediately or fails

**Solutions**:
1. Check data directory permissions:
   ```bash
   ls -la /var/lib/udaya/
   sudo chown -R udaya:udaya /var/lib/udaya
   ```

2. Check if port is already in use:
   ```bash
   sudo lsof -i :19798
   sudo systemctl stop udaya-node1 udaya-node2 udaya-node3
   ```

3. Check logs:
   ```bash
   journalctl -u udaya-node1 -n 100 --no-pager
   ```

### Can't connect to peers

**Symptoms**: `getpeerinfo` returns 0 peers

**Solutions**:
1. Verify firewall allows port 19798:
   ```bash
   sudo ufw allow 19798/tcp
   ```

2. Check if seed nodes are reachable:
   ```bash
   nc -zv seed1.testnet.udaya.net 19798
   ```

3. Verify `external_ip` in config is set correctly

### Sync stuck / Chain divergence

**Symptoms**: Node height doesn't increase, or differs from peers

**Solutions**:
1. Check chain tip matches network:
   ```bash
   curl -u udaya:pass -X POST http://127.0.0.1:18332/ \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","id":1,"method":"getblockchaininfo","params":[]}'
   ```

2. If chain is corrupted, delete and resync:
   ```bash
   sudo systemctl stop udaya-node1
   sudo rm -rf /var/lib/udaya/node1/blockchain
   sudo systemctl start udaya-node1
   ```

### RPC not responding

**Symptoms**: Connection refused on port 18332

**Solutions**:
1. Verify RPC is enabled in config:
   ```toml
   [rpc]
   enable = true
   listen_addr = "127.0.0.1"
   listen_port = 18332
   ```

2. Check if another process is using the port:
   ```bash
   sudo lsof -i :18332
   ```

### Out of disk space

**Symptoms**: Node crashes with I/O errors

**Solutions**:
1. Check disk usage:
   ```bash
   du -sh /var/lib/udaya/
   df -h /var/lib/udaya/
   ```

2. Enable pruning (if supported):
   ```toml
   [storage]
   prune_blocks = true
   prune_target_gb = 20
   ```

### High memory usage

**Symptoms**: Node uses > 4GB RAM

**Solutions**:
1. Reduce database cache:
   ```toml
   [storage]
   db_cache_size_mb = 512
   ```

2. Reduce max peers:
   ```toml
   [network]
   max_peers = 25
   ```

### Wallet issues

**Symptoms**: Can't send transactions, balance incorrect

**Solutions**:
1. Verify wallet file exists and is readable
2. Check coinbase maturity (100 blocks)
3. Rescan wallet:
   ```bash
   ./target/release/udaya-wallet-cli --rescan
   ```

## Getting Help

- GitHub Issues: https://github.com/udayafoundation/udaya/issues
- Discord: https://discord.gg/udaya
- Documentation: https://docs.udaya.org
