#!/usr/bin/env bash
set -euo pipefail

# Update Scoop and Homebrew manifests after a GitHub Release.
# Required env: VERSION (semver, no "v" prefix), GH_TOKEN

: "${VERSION:?VERSION env var is required (e.g. 1.0.4)}"
: "${GH_TOKEN:?GH_TOKEN env var is required}"

REPO="statik/audience"
SCOOP_REPO="kindlyops/kindlyops-scoop"
HOMEBREW_REPO="kindlyops/homebrew-tap"
TAG="v${VERSION}"
WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

download_asset() {
  local pattern="$1"
  gh release download "$TAG" \
    --repo "$REPO" \
    --pattern "$pattern" \
    --dir "$WORKDIR"
}

sha256_of() {
  sha256sum "$1" | cut -d' ' -f1
}

push_to_repo() {
  local repo="$1" path="$2" content_b64="$3" message="$4"

  # Fetch existing file SHA (needed for updates, absent for creates)
  local existing_sha
  existing_sha=$(
    gh api "repos/${repo}/contents/${path}" \
      --jq '.sha' 2>/dev/null
  ) || existing_sha=""

  local json
  json=$(jq -n \
    --arg message "$message" \
    --arg content "$content_b64" \
    --arg sha "$existing_sha" \
    '{message: $message, content: $content}
     + if $sha != "" then {sha: $sha} else {} end'
  )

  gh api --method PUT "repos/${repo}/contents/${path}" \
    --input - <<< "$json" > /dev/null
}

# --- Scoop (Windows .msi) ---

echo "Downloading MSI..."
download_asset "*.msi"
MSI_FILE=$(find "$WORKDIR" -name '*.msi' | head -1)
MSI_HASH=$(sha256_of "$MSI_FILE")

echo "Generating Scoop manifest..."
SCOOP_JSON=$(jq -n \
  --arg version "$VERSION" \
  --arg hash "$MSI_HASH" \
  --arg repo "$REPO" \
  '{
    version: $version,
    description: "PTZ camera controller with live video feed and preset management",
    homepage: ("https://github.com/" + $repo),
    license: "MIT",
    architecture: {
      "64bit": {
        url: ("https://github.com/" + $repo + "/releases/download/v" + $version + "/Audience_" + $version + "_x64_en-US.msi"),
        hash: $hash
      }
    },
    checkver: { github: ("https://github.com/" + $repo) },
    autoupdate: {
      architecture: {
        "64bit": {
          url: ("https://github.com/" + $repo + "/releases/download/v$version/Audience_$version_x64_en-US.msi")
        }
      }
    }
  }'
)

SCOOP_B64=$(echo "$SCOOP_JSON" | base64 -w0)
echo "Pushing Scoop manifest to ${SCOOP_REPO}..."
push_to_repo "$SCOOP_REPO" "audience.json" "$SCOOP_B64" \
  "chore: update audience to ${VERSION}"

# --- Homebrew (macOS .dmg) ---

echo "Downloading DMG..."
download_asset "*.dmg"
DMG_FILE=$(find "$WORKDIR" -name '*.dmg' | head -1)
DMG_HASH=$(sha256_of "$DMG_FILE")

echo "Generating Homebrew cask..."
CASK_CONTENT=$(cat <<RUBY
cask "audience" do
  version "${VERSION}"
  sha256 "${DMG_HASH}"

  url "https://github.com/${REPO}/releases/download/v#{version}/Audience_#{version}_universal.dmg"
  name "Audience"
  desc "PTZ camera controller with live video feed and preset management"
  homepage "https://github.com/${REPO}"

  livecheck do
    url :url
    strategy :github_latest
  end

  app "Audience.app"
end
RUBY
)

CASK_B64=$(echo "$CASK_CONTENT" | base64 -w0)
echo "Pushing Homebrew cask to ${HOMEBREW_REPO}..."
push_to_repo "$HOMEBREW_REPO" "Casks/audience.rb" "$CASK_B64" \
  "chore: update audience to ${VERSION}"

echo "Done. Scoop and Homebrew manifests updated for ${VERSION}."
