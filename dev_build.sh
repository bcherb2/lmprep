#!/bin/bash
set -e

# Build the Rust binary
echo "Building Rust binary..."
cargo build --release

# Create platform-specific directory
PLATFORM=$(uname)
if [ "$PLATFORM" = "Darwin" ]; then
    # Check if we're on Apple Silicon
    if [ "$(uname -m)" = "arm64" ]; then
        BINARY_DIR="binaries/darwin_arm64"
    else
        BINARY_DIR="binaries/darwin_x86_64"
    fi
elif [ "$PLATFORM" = "Linux" ]; then
    BINARY_DIR="binaries/linux_x86_64"
else
    echo "Unsupported platform: $PLATFORM"
    exit 1
fi

# Create directory if it doesn't exist
mkdir -p "$BINARY_DIR"

# Copy binary to platform-specific directory
echo "Copying binary to $BINARY_DIR..."
cp "target/release/lm" "$BINARY_DIR/"

# Install package locally
if [ "$1" = "--install" ]; then
    echo "Installing package locally..."
    pip install -e .
fi

echo "Dev build complete!"
