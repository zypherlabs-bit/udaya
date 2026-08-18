# Udaya Developer Guide

## Welcome Developers!

This guide provides comprehensive information for developers who want to contribute to Udaya, build applications on the platform, or understand the codebase architecture.

---

## Table of Contents

1. [Development Setup](#development-setup)
2. [Codebase Structure](#codebase-structure)
3. [Coding Standards](#coding-standards)
4. [Building and Testing](#building-and-testing)
5. [Contributing Guidelines](#contributing-guidelines)
6. [API Development](#api-development)
7. [Smart Contract Development](#smart-contract-development)
8. [Debugging and Profiling](#debugging-and-profiling)
9. [Release Process](#release-process)
10. [Community Resources](#community-resources)

---

## Development Setup

### Prerequisites

- **Rust**: 1.75+ (recommended: latest stable)
- **Git**: 2.30+
- **OS**: Windows 10+, Linux (Ubuntu 22.04+, Debian 12+, Fedora 38+), macOS 13+
- **Tools**: `cargo`, `rustfmt`, `clippy`, `rust-analyzer`

### Recommended IDE Setup

#### Visual Studio Code

1. Install extensions:
   - Rust Analyzer
   - Better TOML
   - CodeLLDB (for debugging)
   - crates

2. Recommended settings (`settings.json`):
   ```json
   {
     "rust-analyzer.checkOnSave.command": "clippy",
     "rust-analyzer.cargo.runBuildScripts": true,
     "rust-analyzer.procMacro.enable": true,
     "rust-analyzer.inlayHints.enable": true,
     "rust-analyzer.assist.importEnforceGranularity": true,
     "rust-analyzer.assist.importPrefix": "crate"
   }
   ```

#### IntelliJ IDEA with Rust Plugin

1. Install Rust plugin
2. Configure Rust toolchain
3. Enable Clippy and Rustfmt integration

### Development Environment Setup

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Install development tools
rustup component add rustfmt clippy rust-analyzer

# Install additional tools
cargo install cargo-audit cargo-deny cargo-edit

# Clone Udaya repository
git clone https://github.com/UdayaFoundation/Udaya.git
cd Udaya

# Initialize git hooks (if available)
# git config core.hooksPath .githooks
```

---

## Codebase Structure

```
udaya/
├── Cargo.toml                # Workspace configuration
├── Cargo.lock                # Dependency versions
├── src/                      # Main workspace
│   ├── Cargo.toml            # Main binary configuration
│   ├── main.rs              # Main daemon entry point
│   ├── api/                 # JSON-RPC and REST API
│   ├── core/                # Core blockchain logic
│   ├── explorer/            # Block explorer backend
│   ├── faucet/              # Testnet faucet
│   ├── mempool/             # Transaction pool
│   ├── miner/               # Mining client
│   ├── mining/              # Mining pool
│   ├── p2p/                 # Peer-to-peer networking
│   ├── pool-server/         # Mining pool server
│   ├── storage/             # Database layer
│   ├── wallet/              # Wallet implementation
│   └── wallet-cli/          # Wallet CLI
├── e2e-validation/           # End-to-end tests
├── benches/                 # Performance benchmarks
├── config/                  # Configuration files
├── deployments/             # Deployment configurations
├── docs/                    # Documentation
├── scripts/                 # Utility scripts
└── tests/                   # Unit and integration tests
```

### Key Crates Overview

| Crate | Purpose | Key Files |
|-------|---------|-----------|
| `udaya-core` | Core blockchain logic | `consensus.rs`, `genesis.rs`, `validation.rs` |
| `udaya-wallet` | Wallet functionality | `lib.rs`, `crypto.rs`, `psbt.rs` |
| `udaya-p2p` | Networking | `network.rs`, `discovery.rs` |
| `udaya-api` | API interfaces | `lib.rs`, `rpc.rs` |
| `udaya-mining` | Mining pool | `lib.rs`, `stratum.rs` |
| `udaya-storage` | Database | `blockchain_db.rs` |

---

## Coding Standards

### Rust Style Guide

1. **Formatting**: Follow `rustfmt` defaults
2. **Linting**: Pass `clippy` with no warnings
3. **Naming**: Use `snake_case` for functions/variables, `PascalCase` for types
4. **Error Handling**: Use `thiserror` or `anyhow` for error types
5. **Documentation**: All public items must have doc comments

### Code Quality Requirements

```bash
# Format code
cargo fmt --all

# Check linting
cargo clippy --all-targets --all-features -- -D warnings

# Audit dependencies
cargo audit

# Check licenses
cargo deny check
```

### Documentation Standards

1. **Public API Documentation**: All public functions, structs, and enums must have `///` documentation
2. **Examples**: Include usage examples where applicable
3. **Module Documentation**: Each module should have a module-level doc comment
4. **README Files**: Each crate should have a README.md explaining its purpose

### Commit Message Guidelines

Follow [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <description>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, missing semicolons)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or modifying tests
- `chore`: Maintenance tasks

**Example**:
```
feat(wallet): add BIP-84 derivation path support

Implement native SegWit address generation using BIP-84
derivation paths. This allows users to generate bech32
addresses directly from the wallet.

Closes #123
```

---

## Building and Testing

### Build Commands

| Command | Description |
|---------|-------------|
| `cargo build` | Debug build |
| `cargo build --release` | Optimized release build |
| `cargo build --release -j4` | Parallel build (4 threads) |
| `cargo build --all-features` | Build with all features |
| `cargo build -p udaya-core` | Build specific crate |

### Test Commands

| Command | Description |
|---------|-------------|
| `cargo test` | Run all tests |
| `cargo test --lib` | Run unit tests |
| `cargo test --test '*'` | Run integration tests |
| `cargo test --release` | Run tests in release mode |
| `cargo test --all-features` | Run tests with all features |
| `cargo test --doc` | Run doc tests |
| `cargo test <test_name>` | Run specific test |

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench performance_benchmarks

# Profile with flamegraph
cargo flamegraph --bench performance_benchmarks
```

### Fuzzing (Nightly Only)

```bash
# Install fuzzing tools
rustup install nightly
cargo install cargo-fuzz

# Run fuzz tests
cargo +nightly fuzz run fuzz_target
```

---

## Contributing Guidelines

### Getting Started with Contributions

1. **Fork the repository**
2. **Create a feature branch**:
   ```bash
   git checkout -b feat/your-feature-name
   ```
3. **Make your changes**
4. **Commit your changes** (following commit guidelines)
5. **Push to your fork**:
   ```bash
   git push origin feat/your-feature-name
   ```
6. **Open a Pull Request**

### Pull Request Process

1. Ensure your code passes all checks:
   ```bash
   cargo test --all-features
   cargo clippy --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   ```

2. Update documentation if needed
3. Add tests for new functionality
4. Update CHANGELOG.md if applicable
5. Request review from maintainers

### Review Process

- **Code Review**: At least one approval required
- **CI Checks**: All checks must pass
- **Documentation**: Must be complete and accurate
- **Tests**: Must have adequate test coverage
- **Performance**: No significant performance regressions

---

## API Development

### JSON-RPC API Extension

To add a new RPC method:

1. **Define the method** in `src/api/src/rpc.rs`:
   ```rust
   #[rpc_method(name = "newmethod")]
   pub fn new_method(&self, params: Params) -> Result<Value> {
       // Implementation
       Ok(json!({ "result": "success" }))
   }
   ```

2. **Register the method** in the RPC handler:
   ```rust
   io.extend_with(new_method.to_delegate());
   ```

3. **Add documentation** to the API docs

### REST API Extension

To add a new REST endpoint:

1. **Define the route** in `src/api/src/rest.rs`:
   ```rust
   async fn new_endpoint(
       Extension(state): Extension<Arc<AppState>>,
   ) -> impl IntoResponse {
       Json(json!({ "status": "ok" }))
   }
   ```

2. **Add the route** to the router:
   ```rust
   router.route("/new-endpoint", get(new_endpoint));
   ```

---

## Smart Contract Development

### Current Scripting Capabilities

Udaya currently supports Bitcoin-style scripting with the following opcodes:

- **Stack operations**: `OP_DUP`, `OP_DROP`, `OP_SWAP`, etc.
- **Bitwise logic**: `OP_AND`, `OP_OR`, `OP_XOR`, etc.
- **Arithmetic**: `OP_ADD`, `OP_SUB`, `OP_MUL`, etc.
- **Cryptographic**: `OP_CHECKSIG`, `OP_CHECKMULTISIG`, etc.
- **Flow control**: `OP_IF`, `OP_NOTIF`, `OP_ELSE`, `OP_ENDIF`

### Future Smart Contract Roadmap

| Phase | Features | Status |
|-------|----------|--------|
| Phase 1 | Basic scripting (current) | ✅ Complete |
| Phase 2 | Taproot/Tapscript | 🚧 In Development |
| Phase 3 | Simple covenants | 📝 Planned |
| Phase 4 | Full smart contracts | 🔮 Future |

---

## Debugging and Profiling

### Debugging Techniques

```bash
# Debug build with symbols
cargo build

# Run with logging
RUST_LOG=debug cargo run

# Attach debugger (VS Code)
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Udaya",
            "program": "${workspaceFolder}/target/debug/udayad",
            "args": ["--config", "config/bitfury.conf"],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

### Profiling Tools

```bash
# CPU profiling with perf (Linux)
perf record --call-graph dwarf ./target/release/udayad
perf report

# Memory profiling with heaptrack
heaptrack ./target/release/udayad

# Flamegraph generation
cargo flamegraph --bench performance_benchmarks
```

---

## Release Process

### Versioning Scheme

Udaya follows [Semantic Versioning](https://semver.org/):

- `MAJOR`: Breaking changes
- `MINOR`: Backwards-compatible features
- `PATCH`: Backwards-compatible bug fixes
- Pre-release: `alpha`, `beta`, `rc`

Current version: `1.0.0-alpha.1`

### Release Checklist

1. **Code Freeze**: No new features, only bug fixes
2. **Testing**: Comprehensive test suite execution
3. **Documentation**: Update all documentation
4. **Changelog**: Update CHANGELOG.md
5. **Version Bump**: Update Cargo.toml versions
6. **Tag Release**: Create Git tag
7. **Build Artifacts**: Generate binaries for all platforms
8. **GitHub Release**: Create release with assets
9. **Announcement**: Notify community

### Release Commands

```bash
# Bump version
cargo set-version 1.0.0-alpha.2

# Build release artifacts
cargo build --release

# Generate checksums
shasum -a 256 target/release/udaya*

# Create Git tag
git tag -a v1.0.0-alpha.2 -m "Release 1.0.0-alpha.2"
git push origin v1.0.0-alpha.2
```

---

## Community Resources

### Official Channels

- **GitHub**: https://github.com/UdayaFoundation/Udaya
- **Discussions**: https://github.com/UdayaFoundation/Udaya/discussions
- **Issues**: https://github.com/UdayaFoundation/Udaya/issues
- **Discord**: https://discord.gg/udaya
- **Twitter**: https://twitter.com/UdayaFoundation

### Learning Resources

- **Rust Documentation**: https://doc.rust-lang.org/
- **Bitcoin Developer Guide**: https://developer.bitcoin.org/
- **Blockchain Basics**: https://github.com/bitcoinbook/bitcoinbook
- **Cryptography**: https://cryptobook.nakov.com/

### Development Tools

- **Rust Analyzer**: https://rust-analyzer.github.io/
- **Clippy**: https://github.com/rust-lang/rust-clippy
- **Rustfmt**: https://github.com/rust-lang/rustfmt
- **Cargo Edit**: https://github.com/killercup/cargo-edit

---

## Development Workflow Examples

### Adding a New Feature

1. **Create feature branch**:
   ```bash
   git checkout -b feat/new-feature
   ```

2. **Implement feature** with tests:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_new_feature() {
           // Test implementation
       }
   }
   ```

3. **Run tests**:
   ```bash
   cargo test --all-features
   ```

4. **Check formatting and linting**:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   ```

5. **Commit and push**:
   ```bash
   git add .
   git commit -m "feat: implement new feature"
   git push origin feat/new-feature
   ```

### Fixing a Bug

1. **Create bugfix branch**:
   ```bash
   git checkout -b fix/bug-description
   ```

2. **Add regression test**:
   ```rust
   #[test]
   fn test_bug_regression() {
       // Test that reproduces the bug
   }
   ```

3. **Implement fix** and verify test passes:
   ```bash
   cargo test --test 'test_bug_regression'
   ```

4. **Commit and push**:
   ```bash
   git add .
   git commit -m "fix: resolve bug description"
   git push origin fix/bug-description
   ```

---

## Best Practices

### Code Organization

1. **Keep functions small** and focused on single responsibility
2. **Use meaningful names** for variables and functions
3. **Prefer immutability** where possible
4. **Handle errors explicitly** rather than using `unwrap()`
5. **Write comprehensive tests** for all functionality

### Performance

1. **Avoid unnecessary allocations** in hot paths
2. **Use appropriate data structures** for the task
3. **Batch database operations** to reduce I/O
4. **Profile before optimizing** to identify bottlenecks
5. **Consider async/await** for I/O-bound operations

### Security

1. **Validate all inputs** from external sources
2. **Use secure cryptographic primitives**
3. **Zeroize sensitive data** after use
4. **Implement proper access control**
5. **Follow defense-in-depth** principles

---

## Conclusion

This developer guide provides the foundation for contributing to and building on the Udaya platform. Whether you're fixing bugs, adding features, or building applications, following these guidelines will help ensure a consistent, high-quality codebase.

**Happy coding!** 🚀

**Last Updated**: 2026-07-27
**Udaya Version**: 1.0.0-alpha.1
**Documentation Status**: Phase 3 Public Alpha Testing Preparation