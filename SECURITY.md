# Security Policy

## Reporting Security Vulnerabilities

The security of the Udaya blockchain is our top priority. We appreciate responsible disclosure of any security vulnerabilities.

**Please DO NOT** open public GitHub issues for security vulnerabilities.

### Reporting Process

1. **Email** security@udaya.org with:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact assessment
   - Suggested fix (if available)

2. **Encryption** (optional but recommended):
   - Use our PGP key for sensitive reports
   - Key fingerprint: `[GENERATE_real_PGP_key_and_update_this_fingerprint]`

3. **Response Time**:
   - Acknowledgment within 48 hours
   - Initial assessment within 7 days
   - Regular updates on resolution progress

### What to Report

Report security issues including but not limited to:
- Consensus vulnerabilities
- Transaction validation bypass
- Network protocol exploitation
- Cryptographic weaknesses
- RPC authentication/authorization bypass
- Memory corruption or panic conditions
- Information disclosure
- Denial of service vulnerabilities
- Smart contract vulnerabilities (if applicable)

### Bug Bounty Program

We run an active bug bounty program for qualifying vulnerabilities:

| Severity | Bounty Range |
|----------|--------------|
| Critical | $5,000 - $50,000 |
| High | $1,000 - $10,000 |
| Medium | $200 - $2,000 |
| Low | $50 - $500 |

**Criteria**:
- Vulnerability must be novel and not previously reported
- Must affect production/mainnet code
- Must include working proof-of-concept (not just theoretical)
- Exploit must be reliable and reproducible

### Safe Harbor

We commit to:
- Not pursue legal action against researchers who:
  - Follow responsible disclosure practices
  - Do not exploit vulnerabilities beyond proof-of-concept
  - Do not access or modify user data
  - Act in good faith to protect user interests
- Work with researchers to resolve issues promptly
- Credit researchers in security advisories (with permission)

### Security Best Practices for Users

#### Running a Node

1. **Change Default Credentials**:
   ```bash
   # Set via environment variables
   export RPC_USER=<generate_strong_username>
   export RPC_PASSWORD=<generate_strong_password>
   ```

2. **Firewall Configuration**:
   ```bash
   # Only expose necessary ports
   # P2P port (default: 9333)
   # RPC port (default: 9332) - bind to localhost only if possible
   
   # Example ufw rules
   sudo ufw allow 9333/tcp  # P2P
   sudo ufw deny 9332/tcp    # Block external RPC access
   ```

3. **Enable TLS for RPC** (recommended for production):
   ```toml
   [rpc]
   enable_tls = true
   tls_cert_path = "/path/to/cert.pem"
   tls_key_path = "/path/to/key.pem"
   ```

4. **Use Non-Root User**:
   ```bash
   sudo useradd -m -s /bin/bash udaya
   sudo -u udaya ./target/release/udayad
   ```

5. **Regular Updates**:
   - Keep software updated to latest release
   - Subscribe to security announcements
   - Review changelog for security fixes

#### Wallet Security

1. **Backup Seeds**:
   - Store mnemonic phrases securely offline
   - Use hardware wallets for large amounts
   - Never share seed phrases

2. **Encryption**:
   - Enable wallet encryption
   - Use strong passwords (16+ characters)
   - Enable 2FA where available

3. **Testing**:
   - Test recovery process before storing significant funds
   - Verify addresses before sending transactions
   - Use small test transactions first

### Security Audit History

| Date | Auditor | Scope | Report |
|------|---------|-------|--------|
| 2026-07 | Internal | Core consensus, cryptography | [generate_security_audit_report](generate_security_audit_report) |

### Known Security Considerations

1. **P2P Protocol**:
   - Current implementation does not use TLS for peer connections
   - Risk: Man-in-the-middle attacks possible on untrusted networks
   - Mitigation: Use trusted networks, firewall P2P port

2. **Default Configuration**:
   - RPC credentials are now configured via environment variables (no defaults)
   - CORS is open by default for ease of development
   - Mitigation: Follow firewall and configuration guidelines

3. **Mining**:
   - Solo mining is single-threaded (not affected)
   - No ASIC optimization profiles tested
   - Mitigation: Use established mining pools for production

4. **Network Attacks**:
   - 51% attacks possible on small networks
   - Eclipse attack protection is basic
   - Mitigation: Wait for 6+ confirmations for high-value transactions

### Security Updates

Subscribe to security notifications:
- GitHub Watch → Releases only
- Follow @UdayaFoundation on Twitter
- Join Discord #security-announcements channel

### Incident Response

If you discover a security incident:

1. **Document** the issue with screenshots/logs
2. **Assess** the potential impact
3. **Report** to security@udaya.org immediately
4. **Preserve** evidence without compromising systems
5. **Cooperate** with the security team during investigation

## Contact

- **Security Team**: security@udaya.org
- **General Contact**: info@udaya.org
- **PGP Key**: Available at https://udaya.org/pgp-key.asc (generate and publish before release)

## References

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://ansible.github.io/community/security/)
- [Blockchain Security Best Practices](https://github.com/ConsenSys/blockchain-security)

---

*Last Updated: 2026-07-26*
*Version: 1.0.0*