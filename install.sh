#!/bin/bash
# Medusa Skill Framework - One-Line Installer
# Usage: curl -sSL https://raw.githubusercontent.com/your-repo/medusa/main/install.sh | bash

set -e

echo "Medusa Skill Framework (MSF) - Installer"
echo "========================================"

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "[1/3] Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "[1/3] Rust already installed ($(rustc --version))"
fi

# Check if project exists
if [ -d "medusa" ]; then
    echo "[2/3] Project directory exists, building..."
    cd medusa
else
    echo "[2/3] Cloning repository..."
    git clone https://github.com/your-repo/medusa.git
    cd medusa
fi

# Build
echo "[3/3] Building Medusa..."
source "$HOME/.cargo/env" 2>/dev/null || true
cargo build --release

echo ""
echo "✅ Medusa installed successfully!"
echo ""
echo "Binary location: $(pwd)/target/release/medusa"
echo ""
echo "Add to PATH: export PATH=\"\$(pwd)/target/release:\$PATH\""
echo "Or run: ./target/release/medusa --help"
