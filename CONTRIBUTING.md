# Contributing to Udaya

Thank you for your interest in contributing to Udaya! This document provides guidelines and information for contributors.

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## How to Contribute

### Reporting Bugs

1. Check existing issues to avoid duplicates
2. Use the bug report template
3. Include:
   - Clear description of the bug
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, Rust version, etc.)
   - Logs or screenshots if applicable

### Suggesting Features

1. Check existing issues and discussions
2. Use the feature request template
3. Describe the problem you're solving
4. Explain your proposed solution
5. Consider alternatives and trade-offs

### Submitting Code Changes

#### Getting Started

```bash
# Fork the repository on GitHub
git clone https://github.com/YOUR_USERNAME/Udaya.git
cd Udaya

# Add upstream remote
git remote add upstream https://github.com/UdayaFoundation/Udaya.git

# Create a feature branch
git checkout -b feature/your-feature-name
```

#### Development Setup

```bash
# Install Rust toolchain
rustup update stable

# Install development dependencies
cargo install cargo-clippy cargo-fmt cargo-audit

# Build the project
cargo build

# Run tests
cargo test

# Run linting
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

#### Code Standards

- Follow Rust edition 2021 conventions
- Use `cargo fmt` for formatting
- Ensure `cargo clippy` passes with no warnings
- Write documentation for public APIs
- Add tests for new functionality
- No `unwrap()` in production code paths — use proper error handling
- No hardcoded secrets or credentials

#### Commit Messages

Use conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`

Examples:
- `feat(consensus): add fast difficulty adjustment`
- `fix(rpc): handle invalid txid gracefully`
- `docs(api): add getblocktemplate documentation`

#### Pull Request Process

1. Update your fork to latest upstream
2. Ensure all tests pass
3. Ensure code is properly formatted
4. Write a clear PR description
5. Reference any related issues
6. Wait for review from maintainers
7. Address review feedback

### Review Process

- All PRs require at least one maintainer review
- CI must pass before merging
- Security-sensitive changes require additional review
- Breaking changes require discussion and approval

## Development Architecture

See [docs/architecture/](docs/architecture/) for detailed architecture documentation.

### Key Modules

| Module | Description |
|--------|-------------|
| `src/core/` | Core blockchain engine (consensus, validation, types) |
| `src/storage/` | RocksDB persistence layer |
| `src/p2p/` | Peer-to-peer networking |
| `src/mempool/` | Transaction memory pool |
| `src/wallet/` | HD wallet implementation |
| `src/mining/` | Mining pool and ASIC tools |
| `src/api/` | JSON-RPC and REST endpoints |
| `src/explorer/` | Blockchain explorer |

## Security

For security vulnerabilities, please follow our [Security Policy](SECURITY.md). Do **not** open public issues for security vulnerabilities.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

- Open a GitHub Discussion for general questions
- Join our community channels (see README for links)
- Check existing documentation before asking

## Recognition

Contributors are recognized in:
- Release notes
- Contributor hall of fame
- Project documentation

Thank you for helping make Udaya better!