# Udaya Phase 3 - Public Alpha Testing Plan

## Overview

This document outlines the comprehensive testing plan for Udaya's Phase 3 - Public Alpha Testing. The goal is to validate that the project works reliably across different platforms and environments outside the core development team's setup.

## Objectives

1. **Cross-platform compilation validation**: Ensure Udaya compiles successfully on Windows, Linux, and macOS
2. **Bug reporting framework**: Establish structured bug reporting for external developers
3. **Installation validation**: Verify installation instructions work on clean machines
4. **Release artifact reproducibility**: Confirm others can reproduce all build artifacts

## Testing Framework

### 1. Cross-Platform Compilation Testing

#### Test Matrix

| Platform | Architecture | Rust Version | Test Status | Notes |
|----------|--------------|--------------|-------------|-------|
| Windows 10/11 | x86_64 | 1.75+ | ❌ Not Tested | |
| Windows 10/11 | aarch64 | 1.75+ | ❌ Not Tested | ARM support validation |
| Ubuntu 22.04 | x86_64 | 1.75+ | ❌ Not Tested | |
| Ubuntu 22.04 | aarch64 | 1.75+ | ❌ Not Tested | Raspberry Pi, cloud ARM |
| Debian 12 | x86_64 | 1.75+ | ❌ Not Tested | |
| Fedora 38+ | x86_64 | 1.75+ | ❌ Not Tested | |
| macOS 13+ | x86_64 | 1.75+ | ❌ Not Tested | Intel Macs |
| macOS 13+ | aarch64 | 1.75+ | ❌ Not Tested | M1/M2 Macs |
| Alpine Linux | x86_64 | 1.75+ | ❌ Not Tested | Docker base |

#### Build Commands to Test

```bash
# Basic build
cargo build --release

# Full test suite
cargo test --all-features

# Specific component builds
cargo build -p udaya-core
cargo build -p udaya-wallet
cargo build -p udaya-mining
```

#### Expected Build Artifacts

- `target/release/udayad` (main daemon)
- `target/release/udaya-cli` (CLI wallet)
- `target/release/udaya-faucet`
- `target/release/udaya-explorer`
- `target/release/udaya-pool-server`

### 2. Bug Reporting Framework

#### Bug Report Template

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

#### Bug Triage Process

1. **Critical**: Crashes, security vulnerabilities, data corruption
2. **High**: Major functionality broken, severe performance issues
3. **Medium**: Minor functionality issues, usability problems
4. **Low**: Cosmetic issues, documentation typos

### 3. Installation Instructions Validation

#### Clean Machine Test Procedure

1. **Prerequisites Installation**:
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"

   # Verify Rust version
   rustc --version  # Should be 1.75+

   # Install dependencies
   sudo apt-get update
   sudo apt-get install -y build-essential pkg-config libssl-dev
   ```

2. **Clone and Build**:
   ```bash
   git clone https://github.com/UdayaFoundation/Udaya.git
   cd Udaya
   cargo build --release
   ```

3. **Basic Functionality Test**:
   ```bash
   # Start node with test configuration
   ./target/release/udayad --config config/testnet.conf --testnet

   # Verify API endpoints
   curl http://localhost:8332/healthz

   # Test wallet functionality
   ./target/release/udaya-cli getnewaddress
   ```

#### Expected Validation Results

- [ ] Rust toolchain installs correctly
- [ ] All dependencies resolve without errors
- [ ] Build completes without warnings/errors
- [ ] Node starts and responds to API calls
- [ ] Basic wallet operations work
- [ ] Configuration files are properly structured

### 4. Release Artifact Reproducibility

#### Artifacts to Validate

1. **Binary Artifacts**:
   - Main daemon (`udayad`)
   - CLI tools (`udaya-cli`, `udaya-faucet`, etc.)
   - All component binaries

2. **Docker Images**:
   - Base image build reproducibility
   - Multi-arch support validation
   - Image size optimization

3. **Kubernetes Manifests**:
   - Helm chart validation
   - Deployment configuration
   - Resource requirements

#### Reproducibility Test Commands

```bash
# Clean build from scratch
git clean -fd
cargo clean
cargo build --release

# Verify binary hashes match expected
sha256sum target/release/udayad

# Docker build reproducibility
docker build --no-cache -t udaya-node -f deployments/docker/Dockerfile .

# Kubernetes manifest validation
kubectl apply --dry-run=client -f deployments/k8s/
```

## Testing Checklist

### Pre-Testing Preparation

- [ ] Create dedicated testing branch (`phase3-testing`)
- [ ] Tag current commit as pre-testing baseline
- [ ] Set up issue tracking with "Phase3-Testing" label
- [ ] Prepare test documentation and guides
- [ ] Announce testing period to community

### Cross-Platform Testing

- [ ] Windows x86_64 compilation
- [ ] Windows aarch64 compilation (if supported)
- [ ] Ubuntu 22.04 x86_64 compilation
- [ ] Ubuntu 22.04 aarch64 compilation
- [ ] macOS Intel compilation
- [ ] macOS Apple Silicon compilation
- [ ] Docker multi-arch build validation
- [ ] Kubernetes deployment testing

### Bug Reporting Setup

- [ ] Create GitHub issue templates for bugs
- [ ] Set up bug triage process documentation
- [ ] Establish community testing guidelines
- [ ] Create testing FAQ for contributors
- [ ] Set up dedicated testing discussion channel

### Installation Validation

- [ ] Test on clean Windows VM
- [ ] Test on clean Ubuntu VM
- [ ] Test on clean macOS VM
- [ ] Validate dependency installation scripts
- [ ] Test alternative installation methods
- [ ] Document common installation issues

### Artifact Reproducibility

- [ ] Verify release binary reproducibility
- [ ] Test Docker image build consistency
- [ ] Validate Kubernetes manifest compatibility
- [ ] Check Helm chart values and templates
- [ ] Verify configuration file templates
- [ ] Test backup/restore procedures

## Community Testing Guidelines

### For Testers

1. **System Requirements**:
   - Minimum 4GB RAM (8GB recommended)
   - 20GB free disk space
   - Rust 1.75+ toolchain
   - Git 2.30+

2. **Testing Focus Areas**:
   - Node synchronization and stability
   - Wallet functionality and security
   - Mining pool operations
   - API endpoint reliability
   - Configuration management
   - Error handling and recovery

3. **Reporting Issues**:
   - Use the provided bug report template
   - Include detailed reproduction steps
   - Attach relevant logs (with sensitive info redacted)
   - Specify exact environment details
   - Check for existing similar issues first

### For Maintainers

1. **Issue Triage**:
   - Daily review of new bug reports
   - Prioritize critical/high severity issues
   - Request additional info when needed
   - Label and categorize appropriately

2. **Community Support**:
   - Monitor testing discussion channels
   - Provide timely responses to questions
   - Offer guidance on debugging techniques
   - Acknowledge all contributions

3. **Testing Metrics**:
   - Track number of unique testers
   - Monitor issue resolution rate
   - Measure test coverage improvement
   - Document platform-specific findings

## Success Criteria

### Minimum Viable Testing

- [ ] ✅ 3+ unique testers from community
- [ ] ✅ All major platforms tested (Windows, Linux, macOS)
- [ ] ✅ 50+ hours of cumulative testing time
- [ ] ✅ Critical/high severity bugs addressed
- [ ] ✅ Installation instructions validated
- [ ] ✅ Basic functionality confirmed on all platforms

### Comprehensive Testing

- [ ] 10+ unique testers from community
- [ ] All platforms and architectures tested
- [ ] 200+ hours of cumulative testing time
- [ ] All reported bugs triaged and addressed
- [ ] Full release artifact reproducibility confirmed
- [ ] Documentation updated with testing findings

## Timeline

- **Week 1-2**: Community recruitment and setup
- **Week 3-4**: Active testing period
- **Week 5**: Bug triage and fixing
- **Week 6**: Final validation and reporting
- **Week 7**: Documentation updates
- **Week 8**: Phase 3 completion and transition to Phase 4

## Reporting and Documentation

All testing results, bug reports, and findings should be documented in:

- `docs/operations/phase3-testing-results.md`
- GitHub Issues with "Phase3-Testing" label
- Community discussion threads
- Weekly progress reports

## Appendix

### Common Testing Commands

```bash
# Basic health check
curl http://localhost:8332/healthz

# Get node info
curl -X POST http://localhost:8332 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"getinfo","params":[],"id":1}'

# Test transaction creation
./target/release/udaya-cli sendtoaddress <address> <amount>

# Monitor logs
tail -f ~/.udaya/debug.log
```

### Debugging Tips

1. **Build Issues**:
   - Run `cargo clean` and rebuild
   - Check Rust version compatibility
   - Verify all dependencies are installed
   - Increase cargo verbosity with `-v`

2. **Runtime Issues**:
   - Check configuration file permissions
   - Verify network connectivity
   - Monitor resource usage (CPU, memory, disk)
   - Enable debug logging

3. **Performance Issues**:
   - Test with different sync modes
   - Monitor disk I/O performance
   - Check for memory leaks
   - Profile with performance tools