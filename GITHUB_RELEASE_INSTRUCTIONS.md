# GitHub Release Instructions for Udaya v1.0.0-alpha.1

📅 **Release Date**: July 27, 2026
🔖 **Version**: v1.0.0-alpha.1
🏷️ **Type**: Pre-release (Alpha)

---

## 📋 Pre-Release Checklist

### ✅ Completed Items
- [x] Repository validation (fmt, clippy, check, build, test)
- [x] Security audit and secrets scan
- [x] Documentation verification
- [x] Version updated to 1.0.0-alpha.1
- [x] Annotated tag created
- [x] Release notes generated
- [x] LICENSE file created
- [x] All required files present

### 📝 Remaining Manual Steps

---

## 🚀 GitHub Publication Steps

### 1. **Push Commits to Remote Repository**

```bash
# Push the initial commit to master branch
git push origin master
```

**Expected Output**:
```
Enumerating objects: 309, done.
Counting objects: 100% (309/309), done.
Delta compression using up to 8 threads
Compressing objects: 100% (271/271), done.
Writing objects: 100% (309/309), 1.24 MiB | 5.12 MiB/s, done.
Total 309 (delta 20), reused 0 (delta 0), pack-reused 0
To github.com:udayafoundation/udaya.git
 * [new branch]      master -> master
```

---

### 2. **Push Annotated Tag to Remote**

```bash
# Push the v1.0.0-alpha.1 tag
git push origin v1.0.0-alpha.1
```

**Expected Output**:
```
Enumerating objects: 1, done.
Counting objects: 100% (1/1), done.
Writing objects: 100% (1/1), 172 bytes, done.
Total 1 (delta 0), reused 0 (delta 0), pack-reused 0
To github.com:udayafoundation/udaya.git
 * [new tag]         v1.0.0-alpha.1 -> v1.0.0-alpha.1
```

---

### 3. **Create GitHub Release**

#### **Web UI Method**
1. **Navigate** to: `https://github.com/udayafoundation/udaya/releases/new`
2. **Tag version**: `v1.0.0-alpha.1` (select from dropdown)
3. **Release title**: `Udaya v1.0.0-alpha.1 - Community Alpha Release`
4. **Description**: Copy contents from `RELEASE_NOTES.md`
5. **Pre-release**: ✅ Check "This is a pre-release"
6. **Attach binaries** (optional):
   - `target/release/Udayad` (Linux binary)
   - `target/release/Udayad.exe` (Windows binary)
   - `target/release/udaya-wallet-cli`
   - `target/release/udaya-faucet`
7. **Publish release**: Click "Publish release"

#### **GitHub CLI Method**
```bash
# Install GitHub CLI if not available
# brew install gh (macOS) or choco install gh (Windows)

# Authenticate
gh auth login

# Create release
gh release create v1.0.0-alpha.1 \
  --title "Udaya v1.0.0-alpha.1 - Community Alpha Release" \
  --notes-file RELEASE_NOTES.md \
  --prerelease \
  target/release/Udayad \
  target/release/udaya-wallet-cli \
  target/release/udaya-faucet
```

---

## 📦 Release Artifacts

### **Built Binaries** (Location: `target/release/`)
| Artifact | Platform | Size | Description |
|----------|----------|------|-------------|
| `Udayad` | Linux/macOS | ~12MB | Main daemon binary |
| `Udayad.exe` | Windows | ~14MB | Main daemon binary |
| `udaya-wallet-cli` | Cross-platform | ~8MB | Wallet CLI tool |
| `udaya-faucet` | Cross-platform | ~6MB | Testnet faucet service |
| `udaya-miner` | Cross-platform | ~10MB | Mining software |

### **Docker Images**
```bash
# Build Docker image
docker build -t udayafoundation/udaya:v1.0.0-alpha.1 -f deployments/docker/Dockerfile .

# Push to Docker Hub (requires authentication)
docker push udayafoundation/udaya:v1.0.0-alpha.1
```

---

## 📋 Post-Release Verification

### **Manual Verification Steps**

1. **Release Page**
   - [ ] Navigate to `https://github.com/udayafoundation/udaya/releases`
   - [ ] Verify v1.0.0-alpha.1 appears in releases list
   - [ ] Confirm "Pre-release" badge is visible

2. **Tag Verification**
   - [ ] Check tags page: `https://github.com/udayafoundation/udaya/tags`
   - [ ] Verify v1.0.0-alpha.1 tag exists
   - [ ] Confirm annotated tag message

3. **Documentation Links**
   - [ ] Test README.md rendering
   - [ ] Verify CHANGELOG.md is accessible
   - [ ] Check LICENSE displays correctly
   - [ ] Confirm CONTRIBUTING.md loads properly

4. **Clone Instructions**
   ```bash
   git clone --branch v1.0.0-alpha.1 https://github.com/udayafoundation/udaya.git
   cd udaya
   git checkout v1.0.0-alpha.1
   ```

5. **Build Verification**
   ```bash
   cd udaya
   cargo build --release
   cargo test --release
   ```

6. **CI Status**
   - [ ] Verify GitHub Actions workflows pass
   - [ ] Check build status badge in README

---

## 🎯 Community Setup

### **GitHub Repository Settings**

1. **Issue Templates**
   - [ ] Verify templates exist in `.github/ISSUE_TEMPLATE/`
   - [ ] Test bug report template
   - [ ] Test feature request template

2. **Pull Request Template**
   - [ ] Confirm `.github/pull_request_template.md` exists
   - [ ] Test PR creation flow

3. **Community Features**
   - [ ] Enable GitHub Discussions
   - [ ] Set up community health files
   - [ ] Configure code owners

4. **Security Settings**
   - [ ] Enable security advisories
   - [ ] Configure dependabot alerts
   - [ ] Set up code scanning (CodeQL)

5. **Branch Protection**
   ```bash
   # Protect master branch
   gh api \
     --method PUT \
     -H "Accept: application/vnd.github+json" \
     -H "X-GitHub-Api-Version: 2022-11-28" \
     /repos/udayafoundation/udaya/branches/master/protection \
     -f '{
       "required_status_checks": {
         "strict": true,
         "contexts": ["ci"]
       },
       "enforce_admins": true,
       "required_pull_request_reviews": {
         "require_code_owner_reviews": true,
         "required_approving_review_count": 2
       },
       "restrictions": null,
       "required_linear_history": true,
       "allow_force_pushes": false,
       "allow_deletions": false
     }'
   ```

---

## 📊 Monitoring and Metrics

### **Release Success Metrics**
- **GitHub Stars**: Track repository stars growth
- **Forks**: Monitor community forks
- **Issues**: Track bug reports and feature requests
- **Downloads**: Monitor release asset downloads
- **Contributors**: Track new contributors

### **Community Engagement**
- **Discussions**: Monitor GitHub Discussions activity
- **Pull Requests**: Track community contributions
- **Issue Resolution**: Measure response time and closure rate
- **Documentation Feedback**: Gather docs improvement suggestions

---

## 🚨 Troubleshooting

### **Common Issues**

1. **Tag Already Exists**
   ```bash
   git tag -d v1.0.0-alpha.1
   git push origin :refs/tags/v1.0.0-alpha.1
   git tag -a v1.0.0-alpha.1 -m "Udaya v1.0.0-alpha.1 - Community Alpha Release"
   git push origin v1.0.0-alpha.1
   ```

2. **Permission Denied**
   ```bash
   # Ensure you have write access to the repository
   gh auth status
   gh auth login
   ```

3. **Release Already Exists**
   ```bash
   # Delete existing release via GitHub UI or:
   gh release delete v1.0.0-alpha.1 --yes
   ```

---

## 📝 Release Checklist Completion

- [x] Repository validation completed
- [x] Security audit passed
- [x] Documentation verified
- [x] Version tag created
- [x] Release notes generated
- [x] Git commands prepared
- [x] Verification steps documented
- [ ] Push commits to GitHub
- [ ] Push tag to GitHub
- [ ] Create GitHub release
- [ ] Announce release to community

---

## 📢 Announcement Template

**Subject**: 🚀 Udaya v1.0.0-alpha.1 - Community Alpha Release is Live!

**Body**:
```markdown
🎉 **Exciting News!** The first public release of Udaya Blockchain is now available!

🔗 **Release Page**: https://github.com/udayafoundation/udaya/releases/tag/v1.0.0-alpha.1
📋 **Documentation**: https://github.com/udayafoundation/udaya#readme
💬 **Community**: https://github.com/udayafoundation/udaya/discussions

**What's Included**:
✅ Production-ready CI/CD pipeline
✅ 22 JSON-RPC methods for full node interaction
✅ HD Wallet with hardware wallet support
✅ Stratum V2 mining pool implementation
✅ Comprehensive security audits
✅ Docker & Kubernetes deployments

**How to Help**:
🐛 Report bugs via GitHub Issues
📝 Improve documentation
🧪 Add test coverage
💬 Join the discussion

**Next Steps**:
- v1.0.0-beta.1 in 6-8 weeks
- Public testnet launch
- Mainnet target: Q1 2027

**Get Started Today**:
```bash
git clone https://github.com/udayafoundation/udaya.git
cd udaya
cargo build --release
./target/release/Udayad --help
```

We're thrilled to have you join us on this journey! 🚀

#Udaya #Blockchain #Crypto #OpenSource #Rust
```

---

## 🎯 Final Verification

### **Repository Status**
```bash
# Check current status
git status
git log --oneline -5
git tag -n
```

### **Build Verification**
```bash
# Quick build test
cargo check --workspace
cargo build --release --bin Udayad
```

### **Test Verification**
```bash
# Run core tests
cargo test --release -p udaya-core
cargo test --release -p udaya-wallet
```

---

## ✅ Completion Confirmation

Once all manual steps are completed:

1. **Repository Published**: ✅ All code pushed to GitHub
2. **Tag Published**: ✅ v1.0.0-alpha.1 tag available
3. **Release Created**: ✅ GitHub release with notes
4. **Documentation Live**: ✅ All docs accessible
5. **Community Ready**: ✅ Issues and discussions enabled

**Udaya v1.0.0-alpha.1 is officially published and ready for community testing!** 🎉