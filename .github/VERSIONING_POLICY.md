# Udaya Versioning Policy

This document outlines Udaya's versioning strategy, release cycle, and support policy.

## 📌 Versioning Strategy

Udaya follows [Semantic Versioning 2.0.0](https://semver.org/) (SemVer) with the format `MAJOR.MINOR.PATCH`:

- **MAJOR**: Incremented for breaking changes or significant new features
- **MINOR**: Incremented for backwards-compatible new functionality
- **PATCH**: Incremented for backwards-compatible bug fixes

### Pre-release Versions

Pre-release versions are indicated by appending a hyphen and identifier:
- **Alpha**: `MAJOR.MINOR.PATCH-alpha.X` - Early testing, unstable API
- **Beta**: `MAJOR.MINOR.PATCH-beta.X` - Feature complete, testing phase
- **RC**: `MAJOR.MINOR.PATCH-rc.X` - Release candidate, final testing

### Build Metadata

Build metadata may be appended with a plus sign:
- `MAJOR.MINOR.PATCH+build.XXXXXX` - Git commit hash or build identifier

## 🎯 Release Types

### 1. Major Releases (X.0.0)

**Frequency**: 12-18 months
**Purpose**: Significant architectural changes, breaking API changes, or major new features

**Criteria**:
- Breaking changes to core protocol
- Major consensus rule changes
- Significant performance improvements
- New major subsystems

**Examples**:
- `1.0.0` → `2.0.0` (Consensus algorithm change)
- `1.0.0` → `2.0.0` (Major wallet architecture rewrite)

### 2. Minor Releases (X.Y.0)

**Frequency**: 3-6 months
**Purpose**: New backwards-compatible features and improvements

**Criteria**:
- New API endpoints
- New wallet features
- Performance optimizations
- New mining algorithms (optional)
- Enhanced security features

**Examples**:
- `1.0.0` → `1.1.0` (New privacy features)
- `1.1.0` → `1.2.0` (Enhanced smart contract support)

### 3. Patch Releases (X.Y.Z)

**Frequency**: As needed (typically 2-4 weeks)
**Purpose**: Bug fixes, security patches, and minor improvements

**Criteria**:
- Critical bug fixes
- Security vulnerability patches
- Dependency updates
- Minor performance tweaks

**Examples**:
- `1.0.0` → `1.0.1` (Security patch)
- `1.1.2` → `1.1.3` (Memory leak fix)

### 4. Pre-release Versions

#### Alpha Releases (X.Y.Z-alpha.N)
- **Frequency**: Monthly during active development
- **Purpose**: Early testing, feature preview
- **Stability**: Unstable, not for production
- **Support**: Best effort, no guarantees

#### Beta Releases (X.Y.Z-beta.N)
- **Frequency**: Quarterly before major/minor releases
- **Purpose**: Feature complete, stabilization
- **Stability**: Mostly stable, some bugs expected
- **Support**: Limited support, security patches only

#### Release Candidates (X.Y.Z-rc.N)
- **Frequency**: As needed before final release
- **Purpose**: Final testing before production
- **Stability**: Production-ready candidate
- **Support**: Full support, final API

## 📅 Release Cycle

### Standard Release Cycle

```
[Alpha Phase] → [Beta Phase] → [RC Phase] → [Stable Release] → [Patch Releases]
    3-6 months      2-3 months     1 month        Production        As needed
```

### Version Progression Example

```
v1.0.0-alpha.1 → v1.0.0-alpha.2 → ... → v1.0.0-beta.1 → v1.0.0-rc.1 → v1.0.0
                      ↑                                    ↑
                  Feature development                  Stable release
```

### Timelines

| Phase | Duration | Focus | Support Level |
|-------|----------|-------|---------------|
| Alpha | 3-6 months | Feature development | Best effort |
| Beta | 2-3 months | Stabilization | Limited |
| RC | 1 month | Final testing | Full |
| Stable | 12+ months | Production | Full |
| LTS | 24 months | Long-term | Extended |

## 🔧 Support Policy

### Support Duration

| Version Type | Active Support | Security Updates | Total Support |
|--------------|----------------|-------------------|---------------|
| Alpha | 3 months | ❌ No | 3 months |
| Beta | 6 months | ✅ Yes | 6 months |
| Stable | 12 months | ✅ Yes | 12 months |
| LTS | 24 months | ✅ Yes | 24 months |

### Support Levels

#### Active Support
- Bug fixes
- Documentation updates
- Community support
- Regular patch releases

#### Security Updates Only
- Critical security patches
- No new features
- Limited community support
- Infrequent releases

#### End of Life (EOL)
- No updates
- No support
- Documentation archived
- Users encouraged to upgrade

### Current Support Status

| Version | Release Date | Support End | Status |
|---------|---------------|-------------|--------|
| v1.0.0-alpha.1 | 2026-07-26 | 2026-10-26 | Active |
| v0.9.0 | 2026-06-15 | 2026-09-15 | EOL |

## 🔄 Upgrade Paths

### Recommended Upgrade Paths

```
v0.9.0 → v1.0.0-alpha.1 → v1.0.0-beta.1 → v1.0.0
```

### Direct Upgrade Compatibility

| From \ To | v1.0.0-alpha.1 | v1.0.0-beta.1 | v1.0.0 |
|-----------|----------------|---------------|--------|
| v0.9.0 | ✅ Supported | ⚠️ Manual steps | ⚠️ Manual steps |
| v1.0.0-alpha.1 | - | ✅ Supported | ✅ Supported |
| v1.0.0-beta.1 | ❌ Not supported | - | ✅ Supported |

### Upgrade Procedures

#### Minor Version Upgrades (X.Y.0 → X.Y+1.0)
1. Backup data directory
2. Stop services
3. Install new version
4. Start services
5. Verify functionality

#### Major Version Upgrades (X.0.0 → X+1.0.0)
1. Review breaking changes
2. Backup data directory
3. Run migration tool if required
4. Update configuration files
5. Test in staging environment
6. Deploy to production
7. Monitor for issues

## 🔒 Backwards Compatibility

### API Compatibility

- **Minor versions**: Fully backwards compatible
- **Major versions**: May include breaking changes
- **Deprecation policy**: 1 major version deprecation period

### Data Format Compatibility

- **Database**: Automatic migration for minor versions
- **Configuration**: Manual updates may be required for major versions
- **Wallet files**: Backwards compatible within major versions

### Network Protocol Compatibility

- **Soft forks**: Backwards compatible
- **Hard forks**: Require network-wide upgrade
- **P2P protocol**: Version negotiation supported

## 📦 Release Artifacts

### Standard Release Artifacts

- **Binaries**: Linux (x86_64, aarch64), Windows (x86_64), macOS (x86_64, aarch64)
- **Containers**: Docker images for all supported architectures
- **Source**: Complete source code archive
- **Checksums**: SHA256 checksums for all artifacts
- **Signatures**: GPG signatures for verification

### Naming Convention

```
udaya-[version]-[platform]-[architecture].[ext]
udaya-v1.0.0-alpha.1-x86_64-unknown-linux-gnu.tar.gz
udaya-v1.0.0-alpha.1-x86_64-pc-windows-msvc.zip
udaya-v1.0.0-alpha.1-universal-apple-darwin.tar.gz
```

### Verification

```bash
# Verify checksums
sha256sum -c udaya-v1.0.0-alpha.1-SHA256SUMS

# Verify GPG signature
gpg --verify udaya-v1.0.0-alpha.1-SHA256SUMS.sig
```

## 🎯 Versioning Workflow

### Version Bumping Process

1. **Identify change type**:
   - Breaking change → Major version bump
   - New feature → Minor version bump
   - Bug fix → Patch version bump

2. **Update version files**:
   - `Cargo.toml` (workspace and crates)
   - `CHANGELOG.md`
   - Documentation

3. **Create release branch**:
   ```bash
   git checkout -b release/v1.0.0-alpha.1
   ```

4. **Final testing**:
   - Run full test suite
   - Perform manual testing
   - Verify documentation

5. **Tag release**:
   ```bash
   git tag -a v1.0.0-alpha.1 -m "Release v1.0.0-alpha.1"
   git push origin v1.0.0-alpha.1
   ```

6. **Build artifacts**:
   - Build binaries for all platforms
   - Create Docker images
   - Generate checksums and signatures

7. **Publish release**:
   - Create GitHub release
   - Update documentation
   - Announce to community

## 🚨 Emergency Releases

### Criteria for Emergency Release
- Critical security vulnerabilities (CVSS ≥ 9.0)
- Major functionality breakdowns
- Data corruption bugs
- Network consensus issues

### Emergency Release Process

1. **Assessment**: Core team evaluates severity
2. **Triage**: Identify root cause and fix
3. **Testing**: Minimal but thorough testing
4. **Release**: Immediate patch release
5. **Communication**: Urgent notification to users
6. **Follow-up**: Full patch in next regular release

### Emergency Versioning
- Increment patch version: `X.Y.Z` → `X.Y.Z+1`
- Append emergency identifier if needed: `X.Y.Z-emergency.1`

## 📊 Version Adoption Metrics

### Monitoring
- Node version distribution
- Wallet version adoption
- Exchange support status
- API client versions

### Deprecation Timeline
1. **Announcement**: 3 months before EOL
2. **Warnings**: 1 month before EOL
3. **EOL**: End of support
4. **Archive**: 6 months after EOL

## 🔄 Governance

### Versioning Authority
- Core team has final authority on version numbers
- Follows SemVer principles
- Community input welcomed

### Dispute Resolution
1. Technical discussion
2. Core team vote
3. Community feedback period
4. Final decision

### Policy Changes
- Requires RFC process
- 2/3 Core team approval
- 30-day community review period

## 📚 References

- [Semantic Versioning 2.0.0](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Udaya Governance Model](GOVERNANCE.md)
- [Udaya Release Checklist](RELEASE_CHECKLIST.md)

This versioning policy ensures predictable, consistent releases that balance innovation with stability for the Udaya ecosystem.