# Creating a Test Release

This document describes how the GitHub Actions release pipeline works and how to create a test (pre-release) build.

## How the Release Pipeline Works

The release workflow (`.github/workflows/release.yml`) is triggered by pushing a git tag matching the pattern `v*.*.*`. It runs three jobs in sequence:

1. **validate-version** -- Ensures the git tag version matches `package.json`
2. **build** -- Builds signed binaries for Windows (MSI, NSIS exe) and macOS (universal DMG)
3. **publish-release** -- Collects artifacts, generates a changelog, and creates a GitHub Release

Tags containing a hyphen (e.g. `v0.2.0-rc.1`) are automatically marked as **pre-release** on GitHub.

## Steps to Create a Test Release

### 1. Bump the version

Use the version-bump script to sync the version across `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`:

```bash
# Explicit version with pre-release suffix
npm run version-bump 0.2.0-rc.1

# Or bump by semver level (major, minor, patch) -- no pre-release suffix
npm run version-bump patch
```

Verify the version was updated in all three files:

```bash
grep '"version"' package.json src-tauri/tauri.conf.json
grep '^version' src-tauri/Cargo.toml
```

### 2. Commit the version bump

```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: bump version to 0.2.0-rc.1"
```

### 3. Run checks before tagging

```bash
just check
just test
```

### 4. Tag the commit

The tag **must** start with `v` and match the version in `package.json` exactly, otherwise the `validate-version` job will fail.

```bash
git tag v0.2.0-rc.1
```

### 5. Push the commit and tag

```bash
git push origin main
git push origin v0.2.0-rc.1
```

This triggers the release workflow.

### 6. Monitor the workflow

Go to the repository's **Actions** tab on GitHub to watch the three jobs run:

- **validate-version** -- Should pass in seconds
- **build** -- Runs on `windows-latest` and `macos-latest` in parallel; builds the Tauri app and uploads artifacts
- **publish-release** -- Creates the GitHub Release with all artifacts attached

### 7. Verify the release

Once the workflow completes, check the **Releases** page on GitHub. A pre-release tag (containing a hyphen like `0.2.0-rc.1`) will be marked as a pre-release automatically. The release will include:

- macOS universal DMG (`.dmg`)
- Windows MSI installer (`.msi`)
- Windows NSIS installer (`.exe`)
- Auto-generated changelog from commits since the previous tag

## Pre-release vs Production Release

| Aspect | Pre-release | Production |
|---|---|---|
| Tag format | `v1.0.0-rc.1`, `v1.0.0-beta.2` | `v1.0.0` |
| GitHub label | Pre-release | Latest |
| Detection | Tag contains a `-` | Tag has no `-` |

## Code Signing (Optional)

The release workflow supports code signing via repository secrets. Without these secrets the builds still succeed, but the binaries will be unsigned.

| Secret | Purpose |
|---|---|
| `WINDOWS_CERTIFICATE_BASE64` | Base64-encoded PFX certificate for Windows |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password for the PFX certificate |
| `APPLE_CERTIFICATE_BASE64` | Base64-encoded P12 certificate for macOS |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the P12 certificate |
| `APPLE_ID` | Apple ID for notarization |
| `APPLE_TEAM_ID` | Apple Developer Team ID |
| `APPLE_APP_SPECIFIC_PASSWORD` | App-specific password for notarization |

## Notes

- The release workflow does **not** build for Linux. Only Windows and macOS binaries are produced.
- macOS builds target `universal-apple-darwin` (both Intel and Apple Silicon).
- The CI workflow (`.github/workflows/ci.yml`) runs on pushes to `main` and all PRs, covering Ubuntu, Windows, and macOS. It does **not** produce release artifacts.
