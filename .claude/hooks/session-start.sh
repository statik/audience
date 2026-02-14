#!/bin/bash
# Install system dependencies needed for Rust linting (cargo clippy) and building.
# This runs automatically when a Claude Code web session starts.

set -e

if [[ "$(uname)" == "Linux" ]] && command -v apt-get &>/dev/null; then
  SUDO=""
  if command -v sudo &>/dev/null && sudo -n true 2>/dev/null; then
    SUDO="sudo"
  fi

  PACKAGES=(
    libwebkit2gtk-4.1-dev
    libappindicator3-dev
    librsvg2-dev
    patchelf
    libgtk-3-dev
  )

  # Check if all packages are already installed
  ALL_INSTALLED=true
  for pkg in "${PACKAGES[@]}"; do
    if ! dpkg -s "$pkg" &>/dev/null; then
      ALL_INSTALLED=false
      break
    fi
  done

  if [[ "$ALL_INSTALLED" == "false" ]]; then
    $SUDO apt-get update -qq || true
    $SUDO apt-get install -y -qq "${PACKAGES[@]}"
  fi
fi
