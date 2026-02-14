# Creating a Test Release

This document describes how the automated release pipeline works and how to trigger releases.

## How the Release Pipeline Works

Releases are fully automated via [semantic-release](https://semantic-release.gitbook.io/). When commits are pushed (or merged) to `main`, the release workflow analyzes commit messages and automatically:

1. Determines the next version based on conventional commit types
2. Updates version numbers in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`
3. Generates a `CHANGELOG.md` entry
4. Commits the version bump with `[skip ci]`
5. Creates a git tag (`v*.*.*`)
6. Creates a GitHub Release with structured release notes
7. Builds signed binaries for Windows and macOS
8. Uploads build artifacts to the GitHub Release

### Version Bump Rules

| Commit type | Example | Version bump |
|---|---|---|
| `fix:` | `fix: correct pan speed calculation` | Patch (0.0.x) |
| `feat:` | `feat: add preset recall buttons` | Minor (0.x.0) |
| `feat!:` or `BREAKING CHANGE:` | `feat!: drop VISCA serial support` | Major (x.0.0) |
| `docs:`, `chore:`, `ci:`, etc. | `docs: update README` | No release |

## Steps to Create a Test Release

### 1. Write your changes using conventional commits

All commits must follow [Conventional Commits](https://www.conventionalcommits.org/) format. This is enforced by commitlint locally (via husky git hook) and in CI (on pull requests).

```bash
git commit -m "feat: add camera group selection"
git commit -m "fix(visca): handle timeout on preset recall"
```

### 2. Open a pull request

Push your branch and open a PR. CI will:
- Lint commit messages (commitlint)
- Run formatting, type, and lint checks on all platforms
- Run all tests

### 3. Merge to main

Once CI passes and the PR is approved, merge it. The release workflow triggers automatically.

### 4. Monitor the workflow

Go to the repository's **Actions** tab to watch the release run:

- **release** -- semantic-release analyzes commits, bumps version, creates tag and GitHub Release
- **build** -- Runs only if a new release was published; builds Tauri app on `windows-latest` and `macos-latest` in parallel
- **upload-assets** -- Uploads build artifacts to the GitHub Release

If no release-triggering commits are found (e.g. only `docs:` or `chore:` commits), the workflow exits without creating a release.

### 5. Verify the release

Check the **Releases** page on GitHub. The release will include:

- Structured release notes grouped by commit type (Features, Bug Fixes, etc.)
- macOS universal DMG (`.dmg`)
- Windows MSI installer (`.msi`)
- Windows NSIS installer (`.exe`)

## Forcing a Test Release

To force a release for testing, create a commit with a release-triggering type:

```bash
# This triggers a patch release
git commit --allow-empty -m "fix: trigger test release"
git push origin main
```

For a pre-release on a different branch, add the branch to the `branches` array in `.releaserc.json`:

```json
{
  "branches": [
    "main",
    { "name": "beta", "prerelease": true }
  ]
}
```

Then push to that branch to get pre-release versions like `1.1.0-beta.1`.

## Manual Version Bump (Escape Hatch)

If you ever need to manually set a version outside of semantic-release:

```bash
npm run version-bump 2.0.0
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore(release): 2.0.0 [skip ci]"
git tag v2.0.0
git push origin main --tags
```

This bypasses semantic-release. Use sparingly.

## Configuration

### semantic-release (`.releaserc.json`)

Controls version analysis, changelog generation, version bumping, and GitHub release creation. Plugins used:

| Plugin | Purpose |
|---|---|
| `@semantic-release/commit-analyzer` | Determines version bump from commits |
| `@semantic-release/release-notes-generator` | Generates structured release notes |
| `@semantic-release/changelog` | Maintains `CHANGELOG.md` |
| `@semantic-release/exec` | Runs `version-bump.mjs` to sync all version files |
| `@semantic-release/git` | Commits version bump back to repo |
| `@semantic-release/github` | Creates GitHub Release |

### Code Signing (Optional)

The build supports code signing via repository secrets. Without these secrets the builds still succeed unsigned.

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

- Releases are triggered **only** by merging to `main`. No manual tagging needed.
- The release workflow does **not** build for Linux. Only Windows and macOS binaries are produced.
- macOS builds target `universal-apple-darwin` (both Intel and Apple Silicon).
- The CI workflow (`.github/workflows/ci.yml`) runs on pushes to `main` and all PRs. It enforces conventional commits on PRs but does **not** produce release artifacts.
- The `[skip ci]` in the version bump commit message prevents the release workflow from re-triggering on its own commit.
