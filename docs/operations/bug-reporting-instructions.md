# Udaya Bug Reporting Instructions

## Comprehensive Guide to Reporting Issues

---

## Table of Contents

1. [Before Reporting](#before-reporting)
2. [Bug Report Template](#bug-report-template)
3. [Severity Guidelines](#severity-guidelines)
4. [Reporting Process](#reporting-process)
5. [Common Issue Types](#common-issue-types)
6. [Security Vulnerabilities](#security-vulnerabilities)
7. [After Reporting](#after-reporting)
8. [Bug Triage Process](#bug-triage-process)
9. [Bug Bounty Program](#bug-bounty-program)

---

## Before Reporting

### ✅ Checklist Before Submitting

- [ ] **Search existing issues**: Make sure the bug hasn't been reported
- [ ] **Update to latest version**: Test with the most recent commit
- [ ] **Reproduce consistently**: Verify the issue happens reliably
- [ ] **Isolate the problem**: Identify minimal reproduction steps
- [ ] **Check documentation**: Review relevant guides and FAQs
- [ ] **Gather information**: Collect logs, screenshots, and environment details

### 🔍 How to Search for Existing Issues

1. Go to [GitHub Issues](https://github.com/UdayaFoundation/Udaya/issues)
2. Use the search bar with relevant keywords
3. Filter by labels (e.g., "bug", "Phase3-Testing")
4. Check closed issues - your problem may already be fixed

---

## Bug Report Template

### 📋 Standard Bug Report Format

```markdown
## Bug Report

**Title**: [Concise, descriptive title summarizing the issue]

**Severity**: [Critical/High/Medium/Low]

### Environment Information
- **Operating System**: [Windows 10/11, Ubuntu 22.04, macOS 13+, etc.]
- **Architecture**: [x86_64, aarch64, etc.]
- **Rust Version**: [`rustc --version` output]
- **Udaya Version**: [Commit hash or version number]
- **Build Type**: [release/debug]
- **Installation Method**: [source/build, Docker, package manager]

### Steps to Reproduce
1. [First step]
2. [Second step]
3. [Third step]
4. [...]

**Expected Behavior**:
[Clear description of what should happen]

**Actual Behavior**:
[Clear description of what actually happens]

### Additional Context
**Logs/Error Messages**:
```
[Paste relevant log output here]
[Use code blocks for readability]
[Redact sensitive information]
```

**Screenshots**:
[If applicable, add screenshots showing the issue]

**Configuration Files**:
```
[Relevant parts of config files]
[Redact sensitive information like passwords]
```

**Possible Workarounds**:
[If you found any temporary solutions]

**Additional Notes**:
- [ ] This is a regression (worked in version X)
- [ ] I can provide a minimal reproduction case
- [ ] I'm available to help debug further
- [ ] This affects production use

**Possible Solution**:
[If you have ideas for how to fix the issue]
```

---

## Severity Guidelines

### 🚨 Severity Classification

| Severity | Criteria | Examples | Response Time |
|----------|----------|----------|---------------|
| **Critical** | Causes data loss, security vulnerabilities, or complete system failure | Consensus bugs, fund loss, remote code execution | Immediate (within hours) |
| **High** | Major functionality broken, severe performance degradation, or significant data corruption risk | Node crashes, failed transactions, API timeouts, sync failures | Within 24 hours |
| **Medium** | Minor functionality issues, usability problems, or edge cases | Incorrect error messages, UI inconsistencies, non-critical race conditions | Within 3 days |
| **Low** | Cosmetic issues, minor inconveniences, or documentation problems | Typos, formatting issues, non-critical warnings | Within 1 week |

---

## Reporting Process

### 📥 How to Submit a Bug Report

1. **Go to GitHub Issues**:
   [https://github.com/UdayaFoundation/Udaya/issues](https://github.com/UdayaFoundation/Udaya/issues)

2. **Click "New Issue"**:
   - Select the "Bug Report" template
   - If no template appears, use our standard format

3. **Fill Out the Template**:
   - Provide as much detail as possible
   - Be clear and concise
   - Use proper formatting (code blocks, lists, etc.)

4. **Add Labels**:
   - `bug` - For all bug reports
   - `Phase3-Testing` - For public alpha testing
   - Platform-specific labels (e.g., `windows`, `linux`, `macos`)
   - Component-specific labels (e.g., `wallet`, `p2p`, `api`)

5. **Submit the Issue**:
   - Click "Submit new issue"
   - Monitor for follow-up questions

---

## Common Issue Types

### 🐛 Common Bug Categories

| Category | Description | Example Issues |
|----------|-------------|---------------|
| **Consensus** | Issues with block validation or chain rules | Invalid blocks accepted, chain forks, difficulty miscalculation |
| **Wallet** | Problems with address generation or transactions | Incorrect balances, failed transactions, address validation errors |
| **Network** | P2P networking issues | Connection failures, peer discovery problems, message propagation issues |
| **API** | JSON-RPC or REST API problems | Method failures, incorrect responses, authentication issues |
| **Performance** | Speed or resource usage issues | Slow sync, high CPU/memory usage, disk I/O bottlenecks |
| **Build** | Compilation or dependency issues | Build failures, missing dependencies, platform-specific build problems |
| **Documentation** | Incorrect or missing documentation | Outdated guides, missing API docs, incorrect examples |
| **Security** | Potential vulnerabilities | Missing input validation, insecure defaults, potential exploits |

---

## Security Vulnerabilities

### 🔒 Reporting Security Issues

**DO NOT** report security vulnerabilities through public GitHub issues!

### ✅ Secure Reporting Process

1. **Email our security team**:
   [security@udaya.org](mailto:security@udaya.org)

2. **Use our PGP key** (if available):
   [Download PGP key](https://udaya.org/security/pgp-key.asc)

3. **Include in your report**:
   - Detailed description of the vulnerability
   - Steps to reproduce
   - Impact assessment
   - Potential mitigations

4. **What to expect**:
   - Acknowledgement within 24 hours
   - Regular updates on progress
   - Coordination for responsible disclosure
   - Credit in security advisories

### 🏆 Security Bug Bounty Program

We offer rewards for qualifying security vulnerabilities:

| Severity | Reward Range | Examples |
|----------|--------------|----------|
| **Critical** | $5,000 - $50,000 | Remote code execution, consensus failures, fund theft |
| **High** | $1,000 - $10,000 | Privilege escalation, authentication bypass, significant DoS |
| **Medium** | $200 - $2,000 | Information disclosure, CSRF, logic flaws |
| **Low** | $50 - $500 | Minor security issues, best practice violations |

**Eligibility Requirements**:
- Must be a previously unreported vulnerability
- Must include clear reproduction steps
- Must not be caused by unsupported configurations
- Must follow responsible disclosure practices

---

## After Reporting

### 📬 What Happens Next

1. **Triage** (Within 24 hours for critical/high):
   - Issue is reviewed and prioritized
   - Labels are added for categorization
   - Maintainer is assigned

2. **Reproduction** (1-3 days):
   - Maintainer attempts to reproduce the issue
   - May request additional information
   - Issue is confirmed or closed as "cannot reproduce"

3. **Investigation** (Varies by complexity):
   - Root cause is identified
   - Potential solutions are discussed
   - Community may be asked for input

4. **Resolution** (Varies by priority):
   - Fix is implemented
   - Tests are added
   - Documentation is updated
   - PR is created and reviewed

5. **Verification** (Before closing):
   - Fix is tested by reporter (if possible)
   - Regression tests are added
   - Issue is closed with resolution summary

### 💬 Communicating with Maintainers

**Do**:
- ✅ Respond promptly to questions
- ✅ Provide requested information
- ✅ Test proposed fixes when possible
- ✅ Be patient and professional
- ✅ Update the issue if you find new information

**Don't**:
- ❌ Demand immediate attention
- ❌ Be rude or aggressive
- ❌ Spam the issue with unrelated comments
- ❌ Share the issue publicly before it's resolved (for security issues)

---

## Bug Triage Process

### 🔍 How Issues Are Prioritized

Our triage team evaluates each bug report based on:

1. **Impact**: How severely does this affect users?
2. **Likelihood**: How likely is this to occur in production?
3. **Workaround**: Is there an easy workaround?
4. **Complexity**: How difficult is this to fix?
5. **User Base**: How many users are affected?

### 📊 Triage Workflow

```
New Issue → Triage → Confirmed → Investigation → Fix → Verification → Closed
         ↘ Unable to Reproduce → Closed
         ↘ Duplicate → Closed
         ↘ Not a Bug → Closed
         ↘ Needs More Info → Awaiting Response
```

### 🏷️ Common Issue States

| State | Meaning | Action Required |
|-------|---------|------------------|
| **Open** | Issue is being worked on | None |
| **Awaiting Response** | More info needed from reporter | Provide requested information |
| **In Progress** | Actively being fixed | None (unless you can help test) |
| **Review Needed** | Fix is ready for review | Test the proposed fix if possible |
| **Closed** | Issue is resolved | None |
| **Wont Fix** | Decision not to fix | None (but you can discuss) |

---

## Bug Bounty Program

### 💰 Reward Program Details

**Eligible Issues**:
- Security vulnerabilities
- Critical bugs affecting consensus
- Major performance regressions
- Significant compatibility issues

**Ineligible Issues**:
- Issues already known internally
- Issues in third-party dependencies
- Theoretical vulnerabilities without practical impact
- Issues requiring physical access
- Social engineering attacks

### 🏆 Claiming Your Reward

1. **Report the issue** following our guidelines
2. **Work with our team** to verify and fix the issue
3. **Provide any additional information** requested
4. **Wait for resolution** and reward determination
5. **Receive your reward** via your preferred method

**Reward Payment Methods**:
- Bitcoin (BTC)
- Ethereum (ETH)
- Bank transfer
- PayPal
- Gift cards

---

## Tips for Effective Bug Reporting

### ✅ Do's and Don'ts

**Do**:
- ✅ Use clear, descriptive titles
- ✅ Provide step-by-step reproduction instructions
- ✅ Include relevant logs and error messages
- ✅ Specify your exact environment
- ✅ Test with the latest version
- ✅ Search for existing reports first
- ✅ Be responsive to follow-up questions
- ✅ Use proper formatting and code blocks
- ✅ Redact sensitive information
- ✅ Be patient and professional

**Don't**:
- ❌ Use vague titles like "It's broken" or "Not working"
- ❌ Omit important details
- ❌ Report the same issue multiple times
- ❌ Be rude or demanding
- ❌ Share security issues publicly
- ❌ Expect instant fixes for complex issues
- ❌ Include unnecessary personal information
- ❌ Spam the issue tracker
- ❌ Argue about priority assignments
- ❌ Reopen closed issues without new information

### 📝 Example of a Good Bug Report

```markdown
## Bug Report

**Title**: Node crashes during initial sync when processing block #123456

**Severity**: High

### Environment Information
- **Operating System**: Ubuntu 22.04.3 LTS
- **Architecture**: x86_64
- **Rust Version**: rustc 1.75.0 (82e1608df 2023-12-21)
- **Udaya Version**: f840d5fbc9df608dcb2c525f3d7ab2c8341338a9
- **Build Type**: release
- **Installation Method**: Built from source using `cargo build --release`

### Steps to Reproduce
1. Clean installation of Ubuntu 22.04 on a VM with 4GB RAM
2. Install Rust toolchain and dependencies as per getting started guide
3. Clone Udaya repository and checkout commit f840d5f
4. Build with `cargo build --release`
5. Start node with `./target/release/udayad --config config/bitfury.conf`
6. Node syncs normally until block #123456
7. At block #123456, node crashes with segmentation fault

**Expected Behavior**:
Node should continue syncing past block #123456 without crashing

**Actual Behavior**:
Node crashes with segmentation fault at block #123456. The crash is consistent and happens every time at the same block.

### Additional Context
**Logs/Error Messages**:
```
[2026-07-27T10:15:23Z INFO  udayad] Syncing block 123455/123456
[2026-07-27T10:15:24Z INFO  udayad] Processing block 123456 (hash: 00000000000000000003b2f1c0b8b3b4b5b6b7b8b9b0b1b2b3b4b5b6b7b8b9b0)
[2026-07-27T10:15:25Z ERROR udayad] Thread 'tokio-runtime-worker' panicked at 'index out of bounds: the len is 10 but the index is 10', src/core/src/consensus.rs:423:21
[2026-07-27T10:15:25Z ERROR udayad] stack backtrace:
   0: rust_begin_unwind
   1: core::panicking::panic_fmt
   2: core::panicking::panic_bounds_check
   3: udaya_core::consensus::validate_block_transactions
   4: udaya_core::consensus::validate_block
   5: udaya_core::blockchain::process_block
   6: udayad::sync::block_sync_loop
   7: tokio::runtime::task::core::CoreTask::poll
   8: tokio::runtime::task::harness::poll
   9: tokio::runtime::task::harness::run
  10: tokio::runtime::thread_pool::worker::run
  11: tokio::runtime::thread_pool::WorkerThread::run
  12: std::sys_common::backtrace::__rust_begin_short_backtrace
  13: std::rt::lang_start::{{closure}}
  14: std::rt::lang_start_internal
  15: main
  16: __libc_start_main
  17: _start
segmentation fault
```

**Configuration Files**:
```toml
[node]
network = "testnet"
rpc_port = 8332
p2p_port = 9798

[database]
data_dir = "~/.udaya/data"
cache_size = 1024
```

**Additional Notes**:
- [x] This is a regression (worked in commit abc123)
- [x] I can provide a minimal reproduction case
- [x] I'm available to help debug further
- [ ] This affects production use

**Possible Solution**:
The error suggests an index out of bounds error in `validate_block_transactions` at line 423 of `consensus.rs`. This might be related to transaction validation when processing blocks with a specific transaction structure. The issue might be in how transaction inputs are being validated against the UTXO set.
```

---

## Conclusion

Effective bug reporting is crucial for the success of Udaya's Phase 3 Public Alpha Testing. By following these guidelines and providing comprehensive, well-documented bug reports, you'll help us identify and resolve issues quickly, ensuring a stable and reliable platform for all users.

**Remember**:
- The more detail you provide, the faster we can fix the issue
- Security issues should be reported privately
- Your contributions make a real difference
- We appreciate all bug reports, big or small

**Thank you for helping make Udaya better!** 🚀

**Need help with bug reporting?** Ask in our [Discord server](https://discord.gg/udaya) or [GitHub Discussions](https://github.com/UdayaFoundation/Udaya/discussions).

**The Udaya Quality Assurance Team**