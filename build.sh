#!/bin/bash
# Medusa Build Script - Cross-Platform
# Usage: ./build.sh [release|debug]

set -e

MODE=${1:-release}
echo "Building Medusa in $MODE mode..."

if command -v rustc &> /dev/null; then
    echo "[1/2] Rust found: $(rustc --version)"
else
    echo "[1/2] Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo "[2/2] Building..."
if [ "$MODE" = "debug" ]; then
    cargo build
    echo "✅ Built: target/debug/medusa"
else
    cargo build --release
    echo "✅ Built: target/release/medusa"
fi
