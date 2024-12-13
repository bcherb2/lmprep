#!/bin/bash

# Deactivate virtual environment if it exists
if [[ -n "${VIRTUAL_ENV}" ]]; then
    deactivate
fi

# Remove the virtual environment
if [ -d "venv" ]; then
    rm -rf venv
    echo "Removed virtual environment"
fi

# Remove any compiled binaries
if [ -f "target/release/lm" ]; then
    rm target/release/lm
    echo "Removed Rust binary"
fi

# Clean cargo build artifacts
if [ -d "target" ]; then
    cargo clean
    echo "Cleaned Rust build artifacts"
fi

echo "Development environment cleaned up successfully"
