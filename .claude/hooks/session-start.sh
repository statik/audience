#!/bin/bash
# Install system dependencies needed for Rust linting (cargo clippy) and building.
# This runs automatically when a Claude Code web session starts.

set -e

if [[ "$(uname)" == "Linux" ]]; then
  sudo apt-get update -qq
  sudo apt-get install -y -qq \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libgtk-3-dev
fi
