# Udaya Phase 3 Testing Instructions

## Comprehensive Testing Guide for Public Alpha Testers

---

## Table of Contents

1. [Testing Overview](#testing-overview)
2. [Test Environment Setup](#test-environment-setup)
3. [Core Functionality Testing](#core-functionality-testing)
4. [Wallet Testing](#wallet-testing)
5. [Mining Testing](#mining-testing)
6. [API Testing](#api-testing)
7. [Network Testing](#network-testing)
8. [Performance Testing](#performance-testing)
9. [Security Testing](#security-testing)
10. [Cross-Platform Testing](#cross-platform-testing)
11. [Containerization Testing](#containerization-testing)
12. [Regression Testing](#regression-testing)
13. [Test Reporting](#test-reporting)
14. [Common Issues and Solutions](#common-issues-and-solutions)

---

## Testing Overview

### Testing Objectives

- **Validate stability**: Ensure node operates reliably over extended periods
- **Verify functionality**: Confirm all features work as intended
- **Assess performance**: Measure system performance under various conditions
- **Identify edge cases**: Find and document unusual behavior
- **Improve documentation**: Update guides based on real-world usage

### Testing Principles

✅ **Reproducibility**: Document exact steps to reproduce issues
✅ **Isolation**: Test one component at a time when possible
✅ **Documentation**: Record all findings, even if they seem minor
✅ **Collaboration**: Share findings with the community
✅ **Thoroughness**: Test both happy paths and error conditions

---

## Test Environment Setup

### Recommended Test Environments

| Environment | Purpose | Notes |
|-------------|---------|-------|
| **Local Development** | Initial testing, debugging | Use debug builds |
| **Dedicated VM** | Stability testing, performance | Use release builds |
| **Cloud Instance** | Network testing, scalability | AWS/GCP/Azure |
| **Bare Metal** | Mining performance, hardware | ASIC/GPU testing |
| **Containerized** | Deployment testing | Docker/Kubernetes |

### Test Environment Checklist

- [ ] Clean OS installation
- [ ] Required dependencies installed
- [ ] Rust toolchain properly configured
- [ ] Firewall/network properly configured
- [ ] Sufficient disk space (50GB+ recommended)
- [ ] Backup/Restore procedure documented

---

## Core Functionality Testing

### Node Synchronization

**Test Cases**:
1. **Initial Sync**: Time to sync from genesis
2. **Partial Sync**: Sync from intermediate block
3. **Re-sync**: Force re-sync and verify consistency
4. **Interruption Recovery**: Kill node during sync, verify recovery

**Commands**:
```bash
# Start node with sync debugging
./target/release/udayad --config config/udaya.conf --log-level debug

# Monitor sync progress
tail -f ~/.udaya/debug.log | grep "sync"

# Check sync status
curl -X POST http://localhost:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"getblockchaininfo","params":[],"id":1}'
```

### Block Validation

**Test Cases**:
1. **Valid Blocks**: Verify acceptance of valid blocks
2. **Invalid Blocks**: Verify rejection of invalid blocks
3. **Orphan Blocks**: Test handling of orphaned blocks
4. **Reorganization**: Force chain reorganization

**Test Script**:
```bash
# Generate test blocks (if available)
# Submit invalid block and verify rejection
# Monitor logs for validation messages
```

---

## Wallet Testing

### Address Generation

**Test Cases**:
1. **P2PKH Addresses**: Legacy address format
2. **P2SH Addresses**: Script hash addresses
3. **P2WPKH Addresses**: Native SegWit
4. **P2TR Addresses**: Taproot addresses
5. **HD Derivation**: BIP-32/39/44/49/84 paths

**Commands**:
```bash
# Generate different address types
./target/release/udaya-cli getnewaddress "legacy"
./target/release/udaya-cli getnewaddress "p2sh-segwit"
./target/release/udaya-cli getnewaddress "bech32"
./target/release/udaya-cli getnewaddress "taproot"

# Verify address validity
./target/release/udaya-cli validateaddress <address>
```

### Transaction Testing

**Test Cases**:
1. **Simple Transactions**: Single input/output
2. **Multi-Input Transactions**: Multiple UTXOs
3. **Multi-Output Transactions**: Multiple recipients
4. **Change Outputs**: Verify change calculation
5. **Fee Calculation**: Verify fee estimation

**Test Workflow**:
1. Generate receiving addresses
2. Create transactions with various parameters
3. Sign and broadcast transactions
4. Verify transaction confirmation
5. Check balance updates

---

## Mining Testing

### Stratum V2 Protocol

**Test Cases**:
1. **Connection Handshake**: Verify protocol negotiation
2. **Mining Subscribe**: Test subscription process
3. **Block Template**: Verify template generation
4. **Share Submission**: Test share acceptance/rejection
5. **Difficulty Adjustment**: Verify dynamic difficulty

**Mining Test Setup**:
```bash
# Start mining pool server
./target/release/udaya-pool-server --config config/pool.conf

# Connect miner client
./target/release/udaya-miner -o stratum+tcp://localhost:3333 -u <wallet> -p x
```

### Block Template Testing

**Test Cases**:
1. **Template Validity**: Verify template structure
2. **Transaction Inclusion**: Verify tx selection
3. **Coinbase Transaction**: Verify coinbase format
4. **Target Difficulty**: Verify difficulty setting
5. **Template Updates**: Test template refresh

**Commands**:
```bash
# Get block template
curl -X POST http://localhost:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"getblocktemplate","params":[],"id":1}'

# Submit solved block (if mining)
curl -X POST http://localhost:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"submitblock","params":["<hexdata>"],"id":1}'
```

---

## API Testing

### JSON-RPC Testing

**Test Cases**:
1. **Method Availability**: Test all documented methods
2. **Parameter Validation**: Test invalid parameters
3. **Error Handling**: Verify error responses
4. **Concurrency**: Test multiple simultaneous requests
5. **Authentication**: Verify RPC security

**API Test Script**:
```bash
# Test each RPC method
methods=("getblockchaininfo" "getblockcount" "getbalance" "getnewaddress")

for method in "${methods[@]}"; do
  echo "Testing $method..."
  curl -X POST http://localhost:8332 \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":[],\"id\":1}"
  echo ""
done
```

### REST API Testing

**Test Cases**:
1. **Health Endpoints**: `/healthz`, `/readyz`, `/health`
2. **Metrics Endpoint**: `/metrics` format validation
3. **CORS Headers**: Verify cross-origin support
4. **Rate Limiting**: Test request throttling

**REST Test Commands**:
```bash
# Test health endpoints
curl http://localhost:8332/healthz
curl http://localhost:8332/readyz
curl http://localhost:8332/health

# Test metrics endpoint
curl http://localhost:8332/metrics
```

---

## Network Testing

### P2P Network Testing

**Test Cases**:
1. **Peer Connection**: Test inbound/outbound connections
2. **Peer Discovery**: Verify DNS seed functionality
3. **Message Propagation**: Test block/tx propagation
4. **Network Partition**: Simulate network splits
5. **Reconnection**: Test automatic reconnection

**Network Test Commands**:
```bash
# Check peer information
curl -X POST http://localhost:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"getpeerinfo","params":[],"id":1}'

# Check network information
curl -X POST http://localhost:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"getnetworkinfo","params":[],"id":1}'
```

### Bandwidth Testing

**Test Cases**:
1. **Initial Sync Bandwidth**: Measure data usage
2. **Ongoing Bandwidth**: Measure steady-state usage
3. **Peak Usage**: Test during high activity
4. **Bandwidth Throttling**: Test with limited bandwidth

**Bandwidth Monitoring**:
```bash
# Use tools like nload, iftop, or Wireshark
nload
iftop -i eth0
```

---

## Performance Testing

### Benchmarking

**Test Cases**:
1. **Block Validation**: Blocks per second
2. **Transaction Validation**: Transactions per second
3. **Signature Verification**: Operations per second
4. **Database Operations**: Read/write performance
5. **Memory Usage**: Peak and average memory

**Performance Test Commands**:
```bash
# Run built-in benchmarks
cargo bench --all

# Monitor system resources
top -p $(pgrep udayad)
vmstat 1
iostat -x 1
```

### Stress Testing

**Test Cases**:
1. **High Transaction Volume**: Flood with transactions
2. **Rapid Block Production**: Simulate high difficulty
3. **Memory Pressure**: Test with limited memory
4. **Disk I/O**: Test with slow storage
5. **CPU Saturation**: Test with limited CPU

**Stress Test Tools**:
```bash
# Use tools like vegeta for HTTP load testing
vegeta attack -duration=60s -rate=100 -targets=targets.txt | vegeta report

# Create test targets file
echo "POST http://localhost:8332" > targets.txt
echo "Content-Type: application/json" >> targets.txt
echo "@rpc-request.json" >> targets.txt
```

---

## Security Testing

### Vulnerability Testing

**Test Cases**:
1. **Input Validation**: Test malformed inputs
2. **Buffer Overflows**: Test large inputs
3. **SQL Injection**: Test database queries
4. **Memory Corruption**: Test edge cases
5. **Race Conditions**: Test concurrent access

**Security Test Tools**:
```bash
# Use cargo-fuzz for fuzzing (nightly only)
cargo +nightly fuzz run fuzz_target

# Use address sanitizer
RUSTFLAGS="-Z sanitizer=address" cargo build
```

### Penetration Testing

**Test Cases**:
1. **RPC Security**: Test authentication bypass
2. **Network Attacks**: Test DoS vectors
3. **Cryptographic Validation**: Test signature verification
4. **Privacy Leaks**: Test information disclosure
5. **Configuration Security**: Test secure defaults

**Security Checklist**:
- [ ] RPC interface secured
- [ ] No sensitive data in logs
- [ ] Proper error handling
- [ ] Input validation
- [ ] Secure defaults

---

## Cross-Platform Testing

### Platform-Specific Testing

**Test Matrix**:

| Platform | Architecture | Test Focus |
|----------|--------------|------------|
| Ubuntu 22.04 | x86_64 | Primary Linux target |
| Ubuntu 22.04 | aarch64 | ARM64 compatibility |
| Debian 12 | x86_64 | Debian compatibility |
| Fedora 38+ | x86_64 | Fedora compatibility |
| Windows 10/11 | x86_64 | Primary Windows target |
| Windows 10/11 | aarch64 | Windows ARM support |
| macOS 13+ | x86_64 | Intel Mac support |
| macOS 13+ | aarch64 | Apple Silicon support |

**Cross-Platform Test Script**:
```bash
# Test on each platform
./scripts/validate-installation.sh

# Verify platform-specific behavior
uname -a
rustc --version
./target/release/udayad --version
```

---

## Containerization Testing

### Docker Testing

**Test Cases**:
1. **Image Build**: Verify Dockerfile works
2. **Image Size**: Optimize for minimal size
3. **Multi-Arch**: Test ARM64 and AMD64
4. **Runtime**: Test container execution
5. **Configuration**: Test environment variables

**Docker Test Commands**:
```bash
# Build Docker image
docker build -t udaya-node -f deployments/docker/Dockerfile .

# Run container
docker run -d -p 8332:8332 -p 9798:9798 --name udaya-node udaya-node:latest

# Test container
docker exec udaya-node udayad --version
curl http://localhost:8332/healthz
```

### Kubernetes Testing

**Test Cases**:
1. **Manifest Validation**: Test YAML syntax
2. **Deployment**: Test pod creation
3. **Scaling**: Test horizontal scaling
4. **Persistence**: Test volume mounting
5. **Networking**: Test service exposure

**Kubernetes Test Commands**:
```bash
# Validate manifests
kubectl apply --dry-run=client -f deployments/k8s/

# Deploy to cluster
kubectl apply -f deployments/k8s/

# Check status
kubectl get pods
kubectl logs <pod-name>
```

---

## Regression Testing

### Test Suite Execution

**Test Cases**:
1. **Unit Tests**: Verify individual components
2. **Integration Tests**: Verify component interaction
3. **End-to-End Tests**: Verify complete workflows
4. **Performance Tests**: Verify no regressions
5. **Security Tests**: Verify no new vulnerabilities

**Regression Test Commands**:
```bash
# Run all tests
cargo test --all-features

# Run specific test suites
cargo test --lib
cargo test --test '*'
cargo test --doc

# Check test coverage
cargo llvm-cov --all-features --workspace --html
```

---

## Test Reporting

### Bug Report Template

```markdown
## Bug Report

**Title**: [Concise description of the issue]

**Severity**: [Critical/High/Medium/Low]

**Environment**:
- OS: [Windows/Linux/macOS - include version]
- Architecture: [x86_64/aarch64/other]
- Rust version: [`rustc --version` output]
- Commit hash: [git commit hash]
- Build type: [release/debug]

**Steps to Reproduce**:
1. [Step 1]
2. [Step 2]
3. [Step 3]

**Expected Behavior**:
[What should happen]

**Actual Behavior**:
[What actually happens]

**Logs/Error Messages**:
```
[Paste relevant logs here]
```

**Additional Context**:
- [ ] This is a regression (worked in previous version)
- [ ] I can provide a minimal reproduction case
- [ ] I'm available to help debug further

**Possible Solution**:
[If you have ideas for a fix]
```

### Test Result Documentation

**Test Result Template**:
```markdown
## Test Results - [Date] - [Your Name]

### Environment
- **Platform**: [OS/Version]
- **Architecture**: [x86_64/aarch64]
- **Rust Version**: [rustc --version]
- **Commit**: [git commit hash]
- **Build**: [release/debug]

### Tests Executed
| Test Category | Tests Passed | Tests Failed | Notes |
|---------------|--------------|--------------|-------|
| Core Functionality | X | Y | [issues found] |
| Wallet Operations | X | Y | [issues found] |
| Mining | X | Y | [issues found] |
| API | X | Y | [issues found] |
| Network | X | Y | [issues found] |

### Issues Found
1. **[Issue #1]**: [Brief description]
   - Severity: [Critical/High/Medium/Low]
   - Steps to reproduce: [steps]
   - GitHub Issue: [link]

2. **[Issue #2]**: [Brief description]
   - Severity: [Critical/High/Medium/Low]
   - Steps to reproduce: [steps]
   - GitHub Issue: [link]

### Performance Metrics
| Metric | Value | Notes |
|--------|-------|-------|
| Sync Time | X minutes | [conditions] |
| TPS | Y tx/sec | [test parameters] |
| Memory Usage | Z MB | [peak/average] |
| CPU Usage | W% | [peak/average] |

### Overall Assessment
- **Stability**: [Excellent/Good/Fair/Poor]
- **Performance**: [Excellent/Good/Fair/Poor]
- **Usability**: [Excellent/Good/Fair/Poor]
- **Documentation**: [Excellent/Good/Fair/Poor]

### Recommendations
1. [Recommendation 1]
2. [Recommendation 2]
3. [Recommendation 3]
```

---

## Common Issues and Solutions

### Build Issues

| Issue | Solution |
|-------|----------|
| `rustc not found` | Install Rust toolchain properly |
| `linker not found` | Install build-essential (Linux) or Visual Studio (Windows) |
| `OpenSSL not found` | Install libssl-dev (Linux) or OpenSSL (macOS/Windows) |
| `Out of memory` | Close other applications or add swap space |
| `Slow builds` | Use `cargo build --release -j$(nproc)` |

### Runtime Issues

| Issue | Solution |
|-------|----------|
| `Port already in use` | Change port in config or kill existing process |
| `Permission denied` | Check file permissions and data directory access |
| `Configuration error` | Validate your config file syntax |
| `Connection refused` | Check if node is running and firewall settings |
| `Sync stalled` | Check network connection and peer count |

### Performance Issues

| Issue | Solution |
|-------|----------|
| Slow sync | Increase database cache size in config |
| High CPU usage | Limit connections in config |
| High memory usage | Reduce cache size or use pruned mode |
| Disk I/O bottleneck | Use faster storage or optimize database |
| Network latency | Choose closer peers or adjust timeout settings |

---

## Conclusion

This comprehensive testing guide provides the foundation for thorough validation of Udaya during the Phase 3 Public Alpha Testing period. By following these instructions and documenting your findings, you'll make a significant contribution to the stability, security, and usability of the Udaya blockchain.

**Remember**:
- Document everything, even minor issues
- Share your findings with the community
- Ask for help when needed
- Have fun exploring the system!

**Happy Testing!** 🚀

**The Udaya Testing Team**