#!/bin/bash
set -e

# Create and activate virtual environment if it doesn't exist
if [ ! -d "venv" ]; then
    python -m venv venv
fi
source venv/bin/activate

# Install test dependencies
pip install pytest pytest-cov

# Build the Rust binary
cargo build --release

# Create directory structure
mkdir -p lmprep/binaries/darwin_universal2
mkdir -p lmprep/binaries/linux_x86_64
mkdir -p lmprep/binaries/win_amd64

# Copy the binary to the appropriate location based on platform
if [[ "$OSTYPE" == "darwin"* ]]; then
    cp target/release/lm lmprep/binaries/darwin_universal2/
    chmod +x lmprep/binaries/darwin_universal2/lm
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    cp target/release/lm lmprep/binaries/linux_x86_64/
    chmod +x lmprep/binaries/linux_x86_64/lm
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    cp target/release/lm.exe lmprep/binaries/win_amd64/
fi

# Install in development mode
pip install -e .

echo "Development setup complete. You can now use 'lm' command (make sure to activate the venv with 'source venv/bin/activate')"
echo "Run tests with: pytest tests/"
