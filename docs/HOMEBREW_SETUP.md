# Homebrew Setup Guide

This guide explains how to publish Scarab Terminal to Homebrew for easy installation on macOS.

## Overview

Scarab can be distributed via Homebrew in two ways:
1. **Homebrew Tap** (Custom repository) - Recommended for alpha/beta releases
2. **Homebrew Core** (Official repository) - For stable 1.0+ releases

## Current Status

- ✅ Homebrew formula created: `packaging/homebrew/scarab.rb`
- ✅ Formula version: v0.1.0-alpha.7
- 🔄 Tap repository: Not yet created
- ⏳ Homebrew Core: Not eligible (requires stable 1.0+)

## Quick Start for Users

Once the tap is published, users will install Scarab with:

```bash
brew tap raibid-labs/scarab
brew install scarab
```

Or in one command:

```bash
brew install raibid-labs/scarab/scarab
```

---

## Publishing to Homebrew Tap

### Prerequisites

1. GitHub repository for the tap (e.g., `homebrew-scarab`)
2. Release with binary archives on GitHub
3. SHA256 checksums for each platform

### Step 1: Create Homebrew Tap Repository

```bash
# Create new repository on GitHub
gh repo create raibid-labs/homebrew-scarab --public --description "Homebrew tap for Scarab Terminal"

# Clone locally
git clone https://github.com/raibid-labs/homebrew-scarab
cd homebrew-scarab
```

### Step 2: Calculate SHA256 Checksums

After each release, calculate checksums for all macOS binaries:

```bash
# For Apple Silicon (ARM64)
curl -sL https://github.com/raibid-labs/scarab/releases/download/v0.1.0-alpha.7/scarab-v0.1.0-alpha.7-aarch64-apple-darwin.tar.gz \
  | shasum -a 256

# For Intel (x86_64)
curl -sL https://github.com/raibid-labs/scarab/releases/download/v0.1.0-alpha.7/scarab-v0.1.0-alpha.7-x86_64-apple-darwin.tar.gz \
  | shasum -a 256
```

### Step 3: Update Formula with Checksums

Edit `packaging/homebrew/scarab.rb`:

```ruby
if OS.mac? && Hardware::CPU.arm?
  url "https://github.com/raibid-labs/scarab/releases/download/v0.1.0-alpha.7/scarab-v0.1.0-alpha.7-aarch64-apple-darwin.tar.gz"
  sha256 "abc123..." # Replace with actual checksum
elsif OS.mac? && Hardware::CPU.intel?
  url "https://github.com/raibid-labs/scarab/releases/download/v0.1.0-alpha.7/scarab-v0.1.0-alpha.7-x86_64-apple-darwin.tar.gz"
  sha256 "def456..." # Replace with actual checksum
end
```

### Step 4: Copy Formula to Tap

```bash
# Copy the formula
cp packaging/homebrew/scarab.rb homebrew-scarab/Formula/scarab.rb

# Commit and push
cd homebrew-scarab
git add Formula/scarab.rb
git commit -m "Add Scarab Terminal formula v0.1.0-alpha.7"
git push origin main
```

### Step 5: Test the Formula

```bash
# Test installation from tap
brew tap raibid-labs/scarab
brew install scarab --verbose

# Test the installation
scarab --version

# Test daemon
scarab-daemon --version

# Cleanup
brew uninstall scarab
brew untap raibid-labs/scarab
```

---

## Automation Script

Create `scripts/update-homebrew-formula.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

VERSION="$1"
REPO="raibid-labs/scarab"

echo "Updating Homebrew formula for $VERSION..."

# Download checksums
ARM64_URL="https://github.com/$REPO/releases/download/$VERSION/scarab-$VERSION-aarch64-apple-darwin.tar.gz"
X64_URL="https://github.com/$REPO/releases/download/$VERSION/scarab-$VERSION-x86_64-apple-darwin.tar.gz"

echo "Calculating SHA256 for ARM64..."
ARM64_SHA=$(curl -sL "$ARM64_URL" | shasum -a 256 | cut -d' ' -f1)

echo "Calculating SHA256 for x86_64..."
X64_SHA=$(curl -sL "$X64_URL" | shasum -a 256 | cut -d' ' -f1)

echo ""
echo "ARM64 SHA256: $ARM64_SHA"
echo "x86_64 SHA256: $X64_SHA"
echo ""

# Update formula
sed -i.bak "s/version \".*\"/version \"$VERSION\"/" packaging/homebrew/scarab.rb
sed -i.bak "s/sha256 \"PLACEHOLDER_SHA256_ARM64\"/sha256 \"$ARM64_SHA\"/" packaging/homebrew/scarab.rb
sed -i.bak "s/sha256 \"PLACEHOLDER_SHA256_X64\"/sha256 \"$X64_SHA\"/" packaging/homebrew/scarab.rb

rm -f packaging/homebrew/scarab.rb.bak

echo "Formula updated! Review changes:"
git diff packaging/homebrew/scarab.rb
```

### Usage:

```bash
chmod +x scripts/update-homebrew-formula.sh
./scripts/update-homebrew-formula.sh v0.1.0-alpha.7
```

---

## Publishing to Homebrew Core (Future)

Once Scarab reaches v1.0.0 stable, you can submit to Homebrew Core:

### Requirements

1. **Stable version** (1.0.0+)
2. **Notable project** (500+ stars on GitHub)
3. **Maintained** (active development for 30+ days)
4. **No GUI** requirement (or provide CLI alternative)
5. **Open source** (OSI-approved license) ✅
6. **macOS/Linux support** ✅

### Submission Process

1. **Fork homebrew-core**:
   ```bash
   gh repo fork Homebrew/homebrew-core
   cd homebrew-core
   ```

2. **Create formula**:
   ```bash
   # Copy your formula
   cp ../scarab/packaging/homebrew/scarab.rb Formula/scarab.rb

   # Audit the formula
   brew audit --strict --online scarab
   ```

3. **Test thoroughly**:
   ```bash
   brew install --build-from-source scarab
   brew test scarab
   brew audit --strict --online scarab
   ```

4. **Submit PR**:
   ```bash
   git checkout -b scarab
   git add Formula/scarab.rb
   git commit -m "scarab 1.0.0 (new formula)"
   gh pr create --title "scarab 1.0.0 (new formula)" \
                --body "GPU-accelerated terminal emulator with plugin system"
   ```

5. **Address review feedback** from Homebrew maintainers

---

## CI/CD Integration

Add to `.github/workflows/release.yml`:

```yaml
update-homebrew:
  name: Update Homebrew Formula
  needs: build-release
  runs-on: macos-latest
  steps:
    - name: Checkout
      uses: actions/checkout@v4

    - name: Update Formula
      run: |
        VERSION=${{ needs.create-release.outputs.version }}
        ./scripts/update-homebrew-formula.sh $VERSION

    - name: Create PR to homebrew-scarab
      env:
        GH_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
      run: |
        git clone https://github.com/raibid-labs/homebrew-scarab
        cd homebrew-scarab
        cp ../packaging/homebrew/scarab.rb Formula/scarab.rb
        git config user.name "github-actions[bot]"
        git config user.email "github-actions[bot]@users.noreply.github.com"
        git add Formula/scarab.rb
        git commit -m "Update scarab to $VERSION"
        git push origin main
```

---

## Testing Checklist

Before publishing each version:

- [ ] Release binaries exist on GitHub
- [ ] SHA256 checksums calculated
- [ ] Formula updated with correct version and checksums
- [ ] Formula syntax validated: `brew audit scarab`
- [ ] Installation tested: `brew install raibid-labs/scarab/scarab`
- [ ] Binary runs correctly: `scarab --version`
- [ ] Service works: `brew services start scarab`
- [ ] Uninstallation works: `brew uninstall scarab`

---

## Troubleshooting

### Formula audit fails

```bash
# Check syntax
brew audit --strict scarab

# Fix common issues
brew audit --fix scarab
```

### Download fails

Ensure GitHub release has the correct asset names:
- `scarab-v0.1.0-alpha.7-aarch64-apple-darwin.tar.gz`
- `scarab-v0.1.0-alpha.7-x86_64-apple-darwin.tar.gz`

### SHA mismatch

Recalculate and update:

```bash
curl -sL DOWNLOAD_URL | shasum -a 256
```

---

## Resources

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Homebrew Acceptable Formulae](https://docs.brew.sh/Acceptable-Formulae)
- [Creating Taps](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
- [Formula Guidelines](https://docs.brew.sh/Formula-Cookbook#guidelines)

---

## Next Steps

1. ✅ Formula created and updated to v0.1.0-alpha.7
2. 🔄 Create `raibid-labs/homebrew-scarab` repository
3. 🔄 Add automation script for checksum updates
4. 🔄 Test installation from tap
5. ⏳ Reach v1.0.0 for Homebrew Core submission
