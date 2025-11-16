## Signing Releases with Minisign

Starting from **version 2.0.3**, all Volta release artifacts must be cryptographically signed using [Minisign](https://jedisct1.github.io/minisign/) to ensure authenticity and integrity.

### Prerequisites

Maintainers must have Minisign installed:

```bash
# macOS
brew install minisign

# Ubuntu/Debian
sudo apt install minisign

# Fedora
sudo dnf install minisign

# Or download from: https://jedisct1.github.io/minisign/
```

---

### One-Time Setup: Generate Release Signing Keys

**This only needs to be done once** (or when rotating keys):

```bash
# Generate a key pair for signing Volta releases
minisign -G -p volta-release.pub -s volta-release.key

# You'll be prompted to create a password - use a strong one!
# This creates:
#   volta-release.pub  - Public key (will be embedded in install.sh)
#   volta-release.key  - Private key (keep this SECRET and secure)
```

**IMPORTANT:**

- **Never commit `volta-release.key` to the repository**
- Store the private key in a secure, encrypted location
- Share the password securely among authorized maintainers only
- Commit `volta-release.pub` to the repository for reference
- Update the `Volta_PUBLIC_KEY` constant in `dev/unix/volta-install.sh` with the public key

---

### Signing a Release

After building release artifacts for all platforms, sign each one:

#### 1. Build Release Artifacts

```bash
# Example for v2.0.3
# (Adjust based on your actual build process)
./build-release.sh 2.0.3
```

This should produce tarballs for all supported platforms:

- `volta-2.0.3-macos.tar.gz`
- `volta-2.0.3-linux.tar.gz`
- `volta-2.0.3-linux-arm.tar.gz`

#### 2. Sign Each Artifact

```bash
# Navigate to the directory with your release artifacts
cd target/release  # or wherever your builds are

# Sign each platform's tarball
minisign -Sm volta-2.0.3-macos.tar.gz
minisign -Sm volta-2.0.3-linux.tar.gz
minisign -Sm volta-2.0.3-linux-arm.tar.gz

# You'll be prompted for the private key password for each file
# This creates .minisig files:
#   volta-2.0.3-macos.tar.gz.minisig
#   volta-2.0.3-linux.tar.gz.minisig
#   volta-2.0.3-linux-arm.tar.gz.minisig
```

#### 3. Verify Signatures Locally

**Always verify signatures before uploading** to catch any issues:

```bash
# Verify each signature
for file in volta-2.0.3-*.tar.gz; do
  echo "Verifying $file..."
  if minisign -Vm "$file" -p volta-release.pub; then
    echo " $file verified successfully"
  else
    echo " FAILED: $file"
    exit 1
  fi
done
```

#### 4. Upload to GitHub Releases

1. Go to https://github.com/volta-cli/volta/releases
2. Click "Draft a new release"
3. Tag: `v2.0.3`
4. Upload **BOTH** the tarballs and their signatures:
   - `volta-2.0.3-macos.tar.gz`
   - `volta-2.0.3-macos.tar.gz.minisig`
   - `volta-2.0.3-linux.tar.gz`
   - `volta-2.0.3-linux.tar.gz.minisig`
   - `volta-2.0.3-linux-arm.tar.gz`
   - `volta-2.0.3-linux-arm.tar.gz.minisig`

#### 5. Verify Public Accessibility

Before publishing the release, verify signatures are downloadable:

```bash
# Test each signature URL (replace v2.0.3 with your version)
curl -I https://github.com/volta-cli/volta/releases/download/v2.0.3/volta-2.0.3-macos.tar.gz.minisig
curl -I https://github.com/volta-cli/volta/releases/download/v2.0.3/volta-2.0.3-linux.tar.gz.minisig
curl -I https://github.com/volta-cli/volta/releases/download/v2.0.3/volta-2.0.3-linux-arm.tar.gz.minisig

# All should return: HTTP/2 200
```

#### 6. Publish the Release

Once verified, publish the GitHub release. Users will now get automatic signature verification!

---

### Key Management

#### Storing the Private Key

**Option 1: Local Secure Storage (Recommended for individual maintainers)**

- Store in an encrypted drive/partition
- The key is password-protected by default (minisign feature)
- Keep offline backups in secure locations

**Option 2: CI/CD Secrets (Recommended for automated releases)**

- Store the private key in GitHub Secrets
- Automate signing in the release workflow
- Limit access to authorized maintainers only

#### Key Rotation

If the private key is compromised or as part of regular security hygiene:

1. Generate new keys (follow "One-Time Setup" above)
2. Update `Volta_PUBLIC_KEY` in `dev/unix/volta-install.sh`
3. Create a new release with the updated install script
4. Announce the key rotation to users
5. Consider re-signing recent releases with the new key

#### Public Key Location

- **Embedded**: The public key is hardcoded in `dev/unix/volta-install.sh` as `Volta_PUBLIC_KEY`
- **Repository**: Also stored as `volta-release.pub` in the repo for reference
- **Documentation**: Listed in README.md security section

---

### Troubleshooting

#### "Signature verification failed" during local testing

```bash
# Make sure you're using the correct public key
cat volta-release.pub

# Verify the key in install.sh matches
grep Volta_PUBLIC_KEY dev/unix/volta-install.sh
```

#### "Permission denied" when signing

```bash
# Make sure the private key has correct permissions
chmod 600 volta-release.key
```

#### Forgot the private key password

Unfortunately, there's no way to recover it. You'll need to:

1. Generate new keys
2. Update the public key in `install.sh`
3. Re-sign all future releases with the new key

---

### Quick Reference

```bash
# Generate keys (one-time)
minisign -G -p volta-release.pub -s volta-release.key

# Sign a release
minisign -Sm volta-VERSION-PLATFORM.tar.gz

# Verify a signature
minisign -Vm volta-VERSION-PLATFORM.tar.gz -p volta-release.pub

# Verify all signatures
for f in volta-*.tar.gz; do minisign -Vm "$f" -p volta-release.pub || echo "Failed: $f"; done
```

---

### Security Notes

- The private key password adds an extra layer of security
- Signatures prove authenticity (from Volta maintainers) and integrity (not modified)
- Users' install script automatically verifies signatures before installation
- If verification fails, installation is aborted for security
- Old releases (< v2.0.3) don't have signatures and skip verification

---

### Questions?

If you have questions about the signing process, please reach out to the core maintainers or open a discussion in the repository.
