# Contributing to Udaya

Thank you for your interest in contributing to Udaya! We welcome contributions from everyone.

## Ways to Contribute

- 🐛 **Bug Reports**: File bug reports using our [bug report template](.github/ISSUE_TEMPLATE/bug_report.md)
- 🚀 **Feature Requests**: Suggest new features using our [feature request template](.github/ISSUE_TEMPLATE/feature_request.md)
- 💻 **Code Contributions**: Submit pull requests with bug fixes or new features
- 📖 **Documentation**: Improve our documentation, tutorials, and examples
- 🎨 **Design**: Help with UI/UX design for our wallet and explorer interfaces
- 🌍 **Community**: Help grow our community through outreach and education

## Getting Started

1. **Fork the repository** and clone your fork
2. **Create a branch** for your changes: `git checkout -b feature/your-feature-name`
3. **Make your changes** following our coding standards
4. **Test thoroughly** and ensure all tests pass
5. **Submit a pull request** with a clear description of your changes

## Development Setup

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/udayafoundation/udaya.git
cd udaya

# Build the project
cargo build

# Run tests
cargo test
```

## Coding Standards

- Follow Rust best practices and idioms
- Use `cargo fmt` to format your code
- Ensure `cargo clippy` produces no warnings
- Write comprehensive tests for new functionality
- Document public APIs with rustdoc comments
- Keep commits focused and meaningful

## Pull Request Process

1. Ensure your code follows our coding standards
2. Update the CHANGELOG.md with your changes
3. Make sure all tests pass (`cargo test`)
4. Submit your pull request to the `main` branch
5. Wait for code review and address any feedback

## Community Guidelines

- Be respectful and inclusive
- Follow our [Code of Conduct](CODE_OF_CONDUCT.md)
- Ask questions if you're unsure about anything
- Have fun and enjoy contributing!

## Good First Issues

Looking for a good place to start? Check out issues labeled `good first issue` in our issue tracker!

## Maintainers

- **Review Process**: All pull requests require at least 2 approvals from maintainers
- **Merge Policy**: Pull requests are merged using squash merge
- **Release Process**: Follows semantic versioning (SemVer)

Thank you for contributing to Udaya! 🚀