# 🚀 Udaya Public Testnet - Phase 4 Invitation

**Welcome to the Udaya Public Testnet!**

We're excited to invite the community to participate in testing the Udaya blockchain network. This is your opportunity to help shape the future of decentralized finance and contribute to the Udaya ecosystem.

## 📅 Testnet Details

**Start Date:** Immediately
**Network:** Udaya Testnet
**Duration:** Ongoing until mainnet launch

## 🌍 How to Participate

### 1. **Run a Node**
Connect to our seed nodes and run your own Udaya node:

**Seed Nodes:**
- `seed1.testnet.udaya.net:19798`
- `seed2.testnet.udaya.net:19798`
- `seed3.testnet.udaya.net:19798`

**Configuration:**
```toml
[network]
preferred_peers = [
    "seed1.testnet.udaya.net:19798",
    "seed2.testnet.udaya.net:19798",
    "seed3.testnet.udaya.net:19798"
]
network = "testnet"
```

### 2. **Mine Test Coins**
Join our mining pool to earn test Udaya coins:

**Mining Pool:**
- **URL:** `pool.testnet.udaya.net:3000`
- **Stratum:** `stratum+tcp://pool.testnet.udaya.net:3001`
- **Fee:** 1% (all proceeds go to community rewards)

### 3. **Use the Block Explorer**
Monitor transactions and blocks in real-time:

**Block Explorer:** [https://explorer.testnet.udaya.net](https://explorer.testnet.udaya.net)

### 4. **Get Test Coins from Faucet**
Need test coins for development? Use our faucet:

**Faucet:** [https://faucet.testnet.udaya.net](https://faucet.testnet.udaya.net)
- **Amount:** 10 UDAYA per request
- **Interval:** 1 hour between requests

### 5. **Develop & Test Applications**
Build and test your dApps on the Udaya testnet:

**Public RPC Endpoints:**
- `https://rpc.testnet.udaya.net:18335`
- `wss://rpc.testnet.udaya.net:18336` (WebSocket)

**Chain ID:** `udaya-testnet-1`

## 🎯 Test Focus Areas

We need your help testing:

1. **Network Stability** - Run nodes for extended periods
2. **Transaction Performance** - Submit high volumes of transactions
3. **Consensus Mechanisms** - Validate block production and finality
4. **Wallet Integration** - Test with various wallet implementations
5. **Smart Contracts** - Deploy and interact with test contracts
6. **Cross-Platform Compatibility** - Test on different OS and hardware

## 🐛 Bug Reporting

Found an issue? Please report it!

**GitHub Issues:** [https://github.com/contactzypherlabs-cell/udaya/issues](https://github.com/contactzypherlabs-cell/udaya/issues)

**Bug Report Template:**
```markdown
### Description
[Clear description of the issue]

### Steps to Reproduce
1. [Step 1]
2. [Step 2]
3. [Step 3]

### Expected Behavior
[What should happen]

### Actual Behavior
[What actually happens]

### Environment
- OS: [Windows/Linux/Mac]
- Version: [Udaya version]
- Hardware: [CPU, RAM, Disk]
- Network: [Connection type]

### Logs
[Relevant log output]
```

## 🏆 Rewards & Recognition

Active participants will be recognized and may receive:

- **Early Access** to mainnet features
- **Community Badges** for top contributors
- **Testnet Rewards** (converted to mainnet tokens)
- **Exclusive NFTs** for bug hunters
- **Feature Naming Rights** for major contributors

## 📚 Resources

**Documentation:** [https://docs.testnet.udaya.net](https://docs.testnet.udaya.net)

**Developer Guide:** [https://docs.testnet.udaya.net/developer](https://docs.testnet.udaya.net/developer)

**API Reference:** [https://docs.testnet.udaya.net/api](https://docs.testnet.udaya.net/api)

## 💬 Community Channels

Stay connected with the Udaya community:

- **Discord:** [https://discord.gg/udaya](https://discord.gg/udaya)
- **Telegram:** [https://t.me/udayaofficial](https://t.me/udayaofficial)
- **Twitter:** [https://twitter.com/udayaofficial](https://twitter.com/udayaofficial)
- **Reddit:** [https://reddit.com/r/udaya](https://reddit.com/r/udaya)

## 🎉 Getting Started

1. **Join our community channels**
2. **Set up your node or wallet**
3. **Start testing and exploring**
4. **Report bugs and provide feedback**
5. **Engage with other community members**

## 📢 Important Notes

- **Testnet coins have no real value** - they are for testing only
- **Network resets may occur** - testnet may be reset during development
- **No guarantees** - testnet is provided "as is" for development purposes
- **Be respectful** - follow our [Code of Conduct](CODE_OF_CONDUCT.md)

## 🙏 Thank You!

Your participation is invaluable to the success of Udaya. Together, we're building a more decentralized and equitable financial future.

**Happy Testing!** 🚀

---

**Udaya Team**
[https://udaya.net](https://udaya.net)
contact@udaya.net
<task_progress>
- [x] Analyze current project structure
- [x] Review existing deployment configurations
- [x] Identify components to deploy (seed nodes, RPC node, block explorer, faucet, mining pool)
- [x] Check documentation website status
- [x] Prepare deployment plan
- [x] Create testnet-specific Docker configuration
- [x] Create Prometheus configuration for testnet
- [x] Fix docker-compose YAML formatting
- [x] Clean up any old files
- [x] Create deployment script
- [x] Create community invitation materials
- [ ] Execute deployment
- [ ] Verify all components are running
</task_progress>
</write_to_file>