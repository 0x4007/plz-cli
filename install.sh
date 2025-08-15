#!/bin/bash

# Install script for plz-cli

set -e

echo "=== Installing plz-cli ==="
echo

# Build release version
echo "Building release version..."
cargo build --release

# Check if build succeeded
if [ ! -f "target/release/plz" ]; then
    echo "Error: Build failed. Please check cargo output above."
    exit 1
fi

# Install location
INSTALL_PATH="/usr/local/bin/plz"

echo
echo "Installing to: $INSTALL_PATH"
echo "This requires sudo permissions."
echo

# Copy to /usr/local/bin
sudo cp target/release/plz "$INSTALL_PATH"
sudo chmod +x "$INSTALL_PATH"

# Verify installation
if command -v plz &> /dev/null; then
    echo "✅ Installation successful!"
    echo
    echo "plz is now available at: $(which plz)"
    echo "Version info:"
    plz --version || echo "(no version flag implemented)"
    echo
    echo "You can now use 'plz' from anywhere in your terminal!"
    echo
    echo "Example usage:"
    echo "  plz 'create a hello world script'"
    echo "  PLZ_DEBUG=1 plz 'list all json files'  # with debug logging"
else
    echo "❌ Installation failed. Please check permissions."
    exit 1
fi