# Udaya Phase 3 Public Alpha Testing Announcement

## 🚀 Udaya Public Alpha Testing is Now Open!

**Date**: [Insert Date]
**Version**: 1.0.0-alpha.1
**Testing Period**: 8 weeks

---

## Welcome to Udaya's Public Alpha Testing Phase!

We're excited to announce that Udaya is now ready for public alpha testing! This is your opportunity to help shape the future of this production-grade SHA-256d Proof-of-Work blockchain by testing, providing feedback, and contributing to the ecosystem.

---

## What is Udaya?

Udaya is a next-generation Layer-1 blockchain that combines the proven security of Bitcoin's SHA-256d Proof-of-Work consensus with modern infrastructure and developer-friendly features:

- **Production-Grade**: Built for reliability and performance
- **SHA-256d PoW**: Bitcoin-compatible consensus algorithm
- **UTXO Model**: Full UTXO set management
- **Modern Features**: SegWit, Taproot, HD Wallets
- **Developer-Friendly**: Comprehensive JSON-RPC and REST APIs
- **Enterprise-Ready**: Docker, Kubernetes, and cloud-native support

---

## Why Participate in Public Alpha Testing?

### For Developers
- **Early Access**: Be among the first to build on Udaya
- **Shape the Platform**: Your feedback directly influences the final product
- **Bug Bounties**: Earn rewards for finding and reporting issues
- **Community Recognition**: Top contributors will be recognized

### For Miners
- **Test Mining**: Help validate our Stratum V2 mining pool implementation
- **Optimization**: Provide feedback on mining performance
- **Early Setup**: Get your infrastructure ready for mainnet

### For Enthusiasts
- **Learn**: Gain hands-on experience with blockchain technology
- **Contribute**: Help improve documentation and user experience
- **Network**: Connect with other blockchain enthusiasts

---

## Getting Started

### 1. System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **CPU** | 2 cores | 4+ cores |
| **RAM** | 4GB | 8GB+ |
| **Storage** | 50GB SSD | 100GB+ SSD |
| **OS** | Windows 10+/Linux/macOS | Latest stable OS |
| **Network** | Stable connection | High-speed, low-latency |

### 2. Installation

#### Quick Start (Linux/macOS)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Clone and build Udaya
git clone https://github.com/UdayaFoundation/Udaya.git
cd Udaya
cargo build --release

# Start your node
./target/release/udayad --config config/bitfury.conf
```

#### Windows
```powershell
# Install Rust
winget install Rustlang.Rustup

# Clone and build Udaya
git clone https://github.com/UdayaFoundation/Udaya.git
cd Udaya
cargo build --release

# Start your node
.\target\release\udayad --config config\bitfury.conf
```

### 3. Configuration

Copy the template configuration and customize it:
```bash
cp config/bitfury.conf config/udaya.conf
# Edit config/udaya.conf with your preferred editor
```

Set your RPC credentials:
```bash
export RPC_USER=your_secure_username
export RPC_PASSWORD=your_strong_random_password
```

---

## Testing Focus Areas

We need your help testing these critical components:

### 🔍 Core Functionality
- [ ] Node synchronization and stability
- [ ] Block validation and chain organization
- [ ] Transaction processing and UTXO management
- [ ] Difficulty adjustment algorithm

### 💳 Wallet Operations
- [ ] Address generation (P2PKH, P2SH, P2WPKH, P2TR)
- [ ] Transaction signing and broadcasting
- [ ] Balance tracking and UTXO management
- [ ] HD wallet derivation paths

### ⛏️ Mining
- [ ] Stratum V2 mining protocol
- [ ] Block template generation
- [ ] Share submission and validation
- [ ] Reward distribution

### 🌐 Networking
- [ ] P2P connection management
- [ ] Peer discovery and handshaking
- [ ] Block and transaction propagation
- [ ] Network resilience and recovery

### 📡 API Endpoints
- [ ] JSON-RPC method reliability
- [ ] REST API endpoint stability
- [ ] WebSocket connection handling
- [ ] Rate limiting and security

### 🐳 Containerization
- [ ] Docker image functionality
- [ ] Kubernetes deployment
- [ ] Helm chart installation
- [ ] Multi-arch support

---

## How to Report Issues

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

### Where to Report

1. **GitHub Issues**: [https://github.com/UdayaFoundation/Udaya/issues](https://github.com/UdayaFoundation/Udaya/issues)
   - Use the "Bug Report" template
   - Label with "Phase3-Testing"

2. **Security Issues**: [security@udaya.org](mailto:security@udaya.org)
   - Do NOT open public issues for security vulnerabilities
   - Follow our [Security Policy](https://github.com/UdayaFoundation/Udaya/blob/main/SECURITY.md)

---

## Testing Resources

### Documentation
- [Getting Started Guide](https://github.com/UdayaFoundation/Udaya/blob/main/docs/operations/getting-started.md)
- [Phase 3 Testing Plan](https://github.com/UdayaFoundation/Udaya/blob/main/docs/operations/phase3-testing-plan.md)
- [API Documentation](https://github.com/UdayaFoundation/Udaya/blob/main/docs/api/README.md)
- [Architecture Overview](https://github.com/UdayaFoundation/Udaya/blob/main/docs/architecture/README.md)

### Community Support
- **GitHub Discussions**: [https://github.com/UdayaFoundation/Udaya/discussions](https://github.com/UdayaFoundation/Udaya/discussions)
- **Discord**: [https://discord.gg/udaya](https://discord.gg/udaya)
- **Twitter**: [@UdayaFoundation](https://twitter.com/UdayaFoundation)

### Testing Tools
- **Installation Validation Script**: `scripts/validate-installation.sh`
- **Artifact Verification Script**: `scripts/verify-artifacts.sh`
- **Test Coverage Reports**: Available in GitHub Actions

---

## Bug Bounty Program

We offer rewards for qualifying bug reports:

| Severity | Reward Range | Examples |
|----------|--------------|----------|
| **Critical** | $500 - $5,000 | Remote code execution, consensus failures, fund loss |
| **High** | $200 - $2,000 | Denial of service, privacy violations, network partitions |
| **Medium** | $50 - $500 | Logic errors, edge cases, performance issues |
| **Low** | $20 - $100 | UI issues, documentation errors, minor bugs |

**Eligibility**:
- Must be a previously unreported issue
- Must include clear reproduction steps
- Must not be caused by unsupported configurations
- Reward amount determined by Udaya Foundation

---

## Success Criteria

### Minimum Viable Testing (Week 4)
- ✅ 3+ unique testers from community
- ✅ All major platforms tested (Windows, Linux, macOS)
- ✅ 50+ hours of cumulative testing time
- ✅ Critical/high severity bugs addressed
- ✅ Installation instructions validated
- ✅ Basic functionality confirmed on all platforms

### Comprehensive Testing (Week 8)
- 🎯 10+ unique testers from community
- 🎯 All platforms and architectures tested
- 🎯 200+ hours of cumulative testing time
- 🎯 All reported bugs triaged and addressed
- 🎯 Full release artifact reproducibility confirmed
- 🎯 Documentation updated with testing findings

---

## Testing Timeline

| Week | Focus | Activities |
|------|-------|------------|
| 1-2 | Community Onboarding | Recruit testers, setup infrastructure, initial testing |
| 3-4 | Core Functionality | Node sync, wallet operations, basic mining |
| 5-6 | Advanced Features | API testing, networking, containerization |
| 7 | Bug Triage & Fixing | Address critical issues, final validation |
| 8 | Documentation & Reporting | Update docs, final reports, community feedback |

---

## Recognition Program

### Top Contributor Rewards

1. **Alpha Tester Badge**: GitHub profile badge for all participants
2. **Top Tester Awards**: Special recognition for most active testers
3. **Community Spotlight**: Featured in Udaya blog and social media
4. **Early Access**: Priority access to future programs
5. **Swag Packs**: Udaya merchandise for top contributors

### Leaderboard

We'll maintain a public leaderboard tracking:
- Number of valid bug reports
- Quality of feedback
- Community engagement
- Documentation contributions

---

## Important Notes

### What to Expect
- **Stability**: This is alpha software - expect bugs and issues
- **Performance**: Not yet optimized for production use
- **Breaking Changes**: API and configuration may change
- **Data Loss**: Do not use with real funds

### What NOT to Do
- ❌ Don't use mainnet configuration
- ❌ Don't expose RPC interfaces publicly without authentication
- ❌ Don't run on production systems
- ❌ Don't expect 100% uptime

### Data Collection
- We collect anonymous usage metrics to improve the software
- No personal or sensitive data is collected
- You can opt-out by disabling telemetry in configuration

---

## How to Stay Updated

1. **Watch the Repository**: Get notifications for all updates
2. **Join Discord**: Real-time discussions and support
3. **Follow on Twitter**: Announcements and progress updates
4. **Subscribe to Newsletter**: [udaya.org/newsletter](https://udaya.org/newsletter)

---

## Next Steps

1. **Join the Community**: Introduce yourself in our Discord server
2. **Set Up Your Node**: Follow the getting started guide
3. **Start Testing**: Focus on the areas that interest you most
4. **Report Findings**: Share your experiences and issues
5. **Provide Feedback**: Help us improve the platform

---

## Contact Information

- **General Inquiries**: [info@udaya.org](mailto:info@udaya.org)
- **Technical Support**: [GitHub Discussions](https://github.com/UdayaFoundation/Udaya/discussions)
- **Security Issues**: [security@udaya.org](mailto:security@udaya.org)
- **Press & Media**: [press@udaya.org](mailto:press@udaya.org)

---

## Conclusion

This public alpha testing phase is a crucial milestone for Udaya. Your participation helps ensure that we deliver a robust, secure, and user-friendly blockchain platform. We're excited to have you on this journey and look forward to your valuable contributions!

**Let's build the future of decentralized finance together!** 🚀

**The Udaya Foundation Team**

---

**Document Status**: Draft for Phase 3 Public Alpha Testing
**Last Updated**: 2026-07-27
**Version**: 1.0.0-alpha.1
**Contact**: [testing@udaya.org](mailto:testing@udaya.org)