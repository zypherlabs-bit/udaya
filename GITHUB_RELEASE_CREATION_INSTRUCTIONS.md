# GitHub Release Creation Instructions for Udaya v1.0.0-alpha.1

## Manual Release Creation via GitHub Web Interface

Since GitHub CLI is not available, follow these steps to create the release manually:

### Step 1: Navigate to GitHub Repository
1. Open your web browser and go to: [https://github.com/contactzypherlabs-cell/udaya](https://github.com/contactzypherlabs-cell/udaya)
2. Ensure you are logged in with an account that has write permissions to the repository

### Step 2: Access the Releases Page
1. Click on the "Releases" link in the right sidebar (under "About")
2. Alternatively, append `/releases` to the repository URL

### Step 3: Create New Release
1. Click the "Draft a new release" button
2. If you don't see this button, ensure you have proper permissions

### Step 4: Configure Release Details

**Tag version:**
```
v1.0.0-alpha.1
```
*Select the existing tag from the dropdown*

**Release title:**
```
Udaya v1.0.0-alpha.1 – Community Alpha Release
```

**Description:**
Copy and paste the entire content from `RELEASE_NOTES.md` file

### Step 5: Mark as Pre-release
1. Check the box labeled "Set as a pre-release"
2. This ensures users understand this is an alpha version

### Step 6: Attach Release Artifacts

Attach the following Windows executables from `target\release\`:
- `Udayad.exe` (Main daemon - 14.1 MB)
- `udaya-wallet-cli.exe` (Wallet CLI - 3.6 MB)
- `udaya-miner.exe` (Miner - 6.1 MB)
- `udaya-pool.exe` (Pool server - 3.2 MB)
- `udaya-faucet.exe` (Faucet service - 5.4 MB)

### Step 7: Publish Release
1. Click the "Publish release" button
2. Verify the release appears in the releases list

## Alternative: Using GitHub API

If you prefer to use the GitHub API, here are the exact commands:

```bash
# First, create a GitHub personal access token with 'repo' scope
# Then use the following curl command:

curl -X POST \
  -H "Authorization: token YOUR_GITHUB_TOKEN" \
  -H "Accept: application/vnd.github.v3+json" \
  https://api.github.com/repos/contactzypherlabs-cell/udaya/releases \
  -d '{
    "tag_name": "v1.0.0-alpha.1",
    "name": "Udaya v1.0.0-alpha.1 – Community Alpha Release",
    "body": "'"$(cat RELEASE_NOTES.md | sed 's/"/\\"/g' | tr '\n' ' ')"'",
    "prerelease": true,
    "draft": false
  }'
```

## Verification Steps

After creating the release:

1. **Verify Release Page**: Ensure all content displays correctly
2. **Check Download Links**: Test that all attached binaries can be downloaded
3. **Validate Tag**: Confirm the tag `v1.0.0-alpha.1` is visible in the repository
4. **Test Clone**: Verify the repository can be cloned with the tag

## Troubleshooting

**Permission Issues:**
- Ensure your GitHub account has write access to the repository
- Contact repository administrators if you need permissions

**Tag Not Found:**
- Verify the tag was pushed: `git push origin v1.0.0-alpha.1`
- Check tags exist: `git ls-remote --tags origin`

**Release Not Appearing:**
- Refresh the page and check your internet connection
- Verify the release wasn't accidentally created as a draft

## Post-Release Checklist

1. [ ] Verify release is publicly accessible
2. [ ] Test download links for all attached binaries
3. [ ] Confirm release notes render correctly
4. [ ] Check that the tag is visible in the repository
5. [ ] Validate the release appears in the repository's release list