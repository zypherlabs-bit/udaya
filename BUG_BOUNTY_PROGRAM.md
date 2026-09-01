# Udaya Blockchain Bug Bounty Program
## Phase 6 - External Security Review

### Effective Date: July 27, 2026
### Program Version: 1.0

---

## 🎯 Program Overview

The Udaya Blockchain Bug Bounty Program is designed to encourage security researchers to help identify and responsibly disclose vulnerabilities in our blockchain protocol, cryptographic implementations, and network infrastructure. This program is critical for ensuring the security and reliability of the Udaya mainnet before and after launch.

---

## 💰 Reward Structure

### Severity Levels and Rewards

| Severity Level | Reward Range (USD) | Description |
|---------------|-------------------|-------------|
| **Critical** | $5,000 - $20,000 | Vulnerabilities that directly impact consensus, allow remote code execution, or enable theft of funds |
| **High** | $1,000 - $5,000 | Vulnerabilities that could lead to denial of service, transaction malleability, or network partitioning |
| **Medium** | $500 - $1,000 | Vulnerabilities that could expose sensitive information or bypass access controls |
| **Low** | $100 - $500 | Minor security issues with limited impact |

### Reward Multipliers

- **Pre-Mainnet Bonus**: ×1.5 multiplier for vulnerabilities found before mainnet launch
- **Testnet Exclusive**: ×2.0 multiplier for vulnerabilities that only affect testnet
- **Creative/Novel**: ×1.2 multiplier for particularly innovative or complex findings
- **Documentation**: ×1.1 multiplier for well-documented reports with reproduction steps

---

## 🔍 Scope

### In Scope

The following components are eligible for bug bounty rewards:

#### Core Protocol
- Consensus algorithm implementation
- Proof-of-Work verification
- Block validation logic
- Chain reorganization handling
- Difficulty adjustment algorithm

#### Cryptographic Implementations
- Address generation (P2PKH, P2SH, Bech32)
- Transaction signing and verification
- Hash functions (SHA-256, RIPEMD-160)
- Merkle tree construction
- Key derivation (BIP32, BIP39, BIP44)

#### Network Layer
- P2P protocol implementation
- Message serialization/deserialization
- Peer discovery and management
- Block and transaction propagation
- DoS protection mechanisms

#### Wallet Security
- Private key management
- Transaction signing
- Address generation
- Seed phrase handling
- Encryption/decryption

#### RPC Interfaces
- JSON-RPC endpoints
- Authentication mechanisms
- Input validation
- Rate limiting

### Out of Scope

The following are **not** eligible for bug bounty rewards:

- Issues in third-party dependencies (unless you can demonstrate exploitation)
- Theoretical vulnerabilities without practical impact
- Vulnerabilities requiring physical access to a device
- Social engineering attacks
- Denial of service attacks that require excessive resources
- Issues already known to the team or publicly disclosed
- Vulnerabilities in example code or documentation

---

## 🎯 Vulnerability Examples

### Critical Severity ($5,000 - $20,000)
- Consensus bugs that could cause chain forks
- Remote code execution in node software
- Private key leakage or theft
- Double spend vulnerabilities
- Invalid block acceptance that violates consensus rules

### High Severity ($1,000 - $5,000)
- Denial of service vulnerabilities in P2P layer
- Transaction malleability issues
- Network partitioning attacks
- Memory corruption in critical components
- Improper access control in RPC interfaces

### Medium Severity ($500 - $1,000)
- Information disclosure (e.g., exposing node information)
- Improper input validation
- Missing rate limiting
- Insecure default configurations
- Side-channel attacks with practical impact

### Low Severity ($100 - $500)
- UI/UX security issues
- Minor implementation flaws
- Non-critical information disclosure
- Low-impact configuration issues

---

## 📋 Rules and Requirements

### Eligibility

1. **First Come, First Served**: Only the first valid report for a vulnerability is eligible
2. **Responsible Disclosure**: You must follow our disclosure guidelines
3. **No Harm**: Testing must not disrupt mainnet operations
4. **Legal Compliance**: All testing must comply with applicable laws

### Prohibited Activities

❌ **Do NOT**:
- Perform denial of service attacks against mainnet
- Attempt to steal funds or disrupt operations
- Engage in social engineering of team members
- Violate privacy of users or team members
- Test on systems you don't own or have permission to test
- Use automated scanning tools that generate excessive traffic

### Testing Guidelines

✅ **DO**:
- Test on our public testnet
- Use your own test nodes
- Coordinate large-scale tests with our team
- Report vulnerabilities promptly
- Provide clear reproduction steps
- Keep findings confidential until disclosure

---

## 📝 Reporting Process

### Step 1: Submit Your Report

Send an encrypted email to: **security@udaya.org**

**PGP Key**: Available at https://udaya.org/security-pgp-key.asc
**Fingerprint**: `A1B2 C3D4 E5F6 G7H8 I9J0 K1L2 M3N4 O5P6 Q7R8`

**Report Template**:
```markdown
# Vulnerability Report

## Title
[Concise description of vulnerability]

## Severity
[Critical/High/Medium/Low]

## Affected Components
[List specific modules/files]

## Description
[Detailed technical description]

## Impact
[What could an attacker achieve?]

## Reproduction Steps
1. [Step 1]
2. [Step 2]
3. [Step 3]

## Proof of Concept
[Code snippet, script, or detailed instructions]

## Suggested Fix
[Optional: Your recommendation for remediation]

## Additional Context
[Any other relevant information]
```

### Step 2: Initial Response

- You will receive an acknowledgment within **24 hours**
- Our security team will validate the report
- We may request additional information

### Step 3: Validation and Triage

- Our team will reproduce the vulnerability
- We will assess the severity and impact
- You may be asked to provide additional details

### Step 4: Resolution

- Development team will create a fix
- We will keep you updated on progress
- You may be asked to verify the fix

### Step 5: Disclosure

- **90-day embargo period** from report acknowledgment
- Coordinated public disclosure after fix is deployed
- You will be credited in our security advisories

---

## 🕒 Timeline

| Phase | Duration | Description |
|-------|----------|-------------|
| **Report Submission** | - | You submit the vulnerability report |
| **Initial Response** | ≤24 hours | We acknowledge receipt of your report |
| **Validation** | ≤7 days | We reproduce and validate the issue |
| **Severity Assessment** | ≤3 days | We determine the severity level |
| **Fix Development** | Varies | Development team creates a patch |
| **Testing & QA** | Varies | Fix is tested internally |
| **Deployment** | Varies | Fix is deployed to affected systems |
| **Disclosure** | 90 days | Coordinated public disclosure |

---

## 🏆 Hall of Fame

Researchers who successfully identify and report vulnerabilities will be recognized in our **Security Hall of Fame**:

### Recognition Levels

| Level | Requirements | Benefits |
|-------|-------------|----------|
| **Bronze** | 1-2 valid reports | Name in Hall of Fame, social media recognition |
| **Silver** | 3-5 valid reports | Bronze benefits + exclusive swag, early access |
| **Gold** | 6+ valid reports | Silver benefits + conference invitations, advisory role |

---

## 📋 Program Terms and Conditions

### Legal Agreement

By participating in this program, you agree to:

1. **Confidentiality**: Keep all vulnerability information confidential
2. **Non-Disclosure**: Not disclose findings to third parties without permission
3. **Non-Exploitation**: Not exploit vulnerabilities for personal gain
4. **Compliance**: Follow all applicable laws and regulations
5. **Indemnification**: Hold Udaya harmless for any actions taken in good faith

### Payment Terms

- Rewards are paid in **USD equivalent** via bank transfer or cryptocurrency
- Taxes are the responsibility of the recipient
- We reserve the right to adjust rewards based on severity assessment
- Multiple vulnerabilities in the same component may be grouped

### Program Changes

- Udaya reserves the right to modify or terminate this program at any time
- Changes will be communicated via the program website
- Existing valid reports will be honored under the terms in effect at submission

---

## 🔗 Resources

### Official Documentation
- [Udaya Whitepaper](https://udaya.org/whitepaper.pdf)
- [Developer Documentation](https://docs.udaya.org)
- [GitHub Repository](https://github.com/udaya/udaya)

### Test Environment
- **Testnet Explorer**: https://testnet-explorer.udaya.org
- **Testnet Faucet**: https://testnet-faucet.udaya.org
- **Testnet RPC**: rpc.testnet.udaya.org:8332

### Contact Information
- **Security Email**: security@udaya.org
- **General Inquiries**: info@udaya.org
- **Website**: https://udaya.org/security

---

## 🛡️ Safe Harbor

Udaya will not initiate legal action against researchers who:
- Follow this bug bounty program's rules
- Act in good faith and with reasonable care
- Do not intentionally cause damage or disruption
- Cease testing upon request
- Maintain confidentiality of findings

This safe harbor applies only to security research conducted in accordance with this program.

---

## 🎓 Getting Started

### For Researchers

1. **Set up a test environment**:
   ```bash
   git clone https://github.com/udaya/udaya.git
   cd udaya
   cargo build --release
   ```

2. **Connect to testnet**:
   ```bash
   ./target/release/udaya --testnet --rpc-user=user --rpc-password=pass
   ```

3. **Start testing** and report any vulnerabilities you find!

### For the Udaya Team

1. **Monitor security@udaya.org** for incoming reports
2. **Acknowledge reports** within 24 hours
3. **Validate and triage** findings promptly
4. **Coordinate fixes** with development team
5. **Maintain communication** with researchers
6. **Process rewards** after successful resolution

---

## 📊 Program Metrics

We will publish quarterly transparency reports including:

- Number of reports received
- Severity distribution
- Average time to resolution
- Total rewards paid
- Top contributors

---

## 🎯 Conclusion

The Udaya Bug Bounty Program is a critical component of our security strategy. By working together with the security research community, we can build a more robust and secure blockchain that benefits everyone.

**Together, we make Udaya stronger and more secure!**

---

**Last Updated**: July 27, 2026
**Program Version**: 1.0
**Contact**: security@udaya.org