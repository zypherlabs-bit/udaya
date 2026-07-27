# Udaya Code Review Standards

## Overview
Mandatory code review standards for all contributions to the Udaya codebase.

## Review Requirements
| Severity | Required Reviewers | Review Window | Merge Requirement |
|----------|-------------------|---------------|-------------------|
| Critical (consensus, crypto) | 3 core maintainers | 72 hours | All approvals |
| High (networking, storage) | 2 maintainers | 48 hours | All approvals |
| Medium (wallet, mining) | 1 maintainer | 24 hours | One approval |
| Low (documentation, tests) | 1 contributor | 12 hours | One approval |

## Review Checklist
### Security
- [ ] No unsafe code without justification
- [ ] No memory safety violations
- [ ] Proper error handling (no panics in production paths)
- [ ] Input validation for all external data
- [ ] Cryptographic operations use constant-time implementations

### Correctness
- [ ] Unit tests cover the change
- [ ] Integration tests where applicable
- [ ] Edge cases documented and handled
- [ ] No regressions in existing tests
- [ ] Fuzz tests pass

### Performance
- [ ] No obvious performance regressions
- [ ] Critical paths are optimized
- [ ] No unnecessary allocations in hot paths
- [ ] Benchmarks included for performance-critical changes

### Style
- [ ] Follows Rust standard formatting (rustfmt)
- [ ] Clippy warnings resolved
- [ ] Meaningful variable/function names
- [ ] Comments explain non-obvious logic
- [ ] No dead code

## CI Requirements
- All tests pass
- Security audit passes (10K fuzz iterations)
- Clippy with no warnings
- Formatting check passes
- Documentation builds successfully