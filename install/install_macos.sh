#!/bin/bash
set -e

# Check for cargo
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed"
    echo "To install Rust and Cargo, run:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "After installation, restart your terminal and run this script again."
    exit 1
fi


# Build the binary
echo "Building lmprep..."
cargo build --release || {
    echo "Error: Build failed"
    echo "Please ensure you have the latest stable Rust toolchain:"
    echo "rustup update stable"
    exit 1
}

# Create the installation directory
INSTALL_DIR="/usr/local/bin"
sudo mkdir -p "$INSTALL_DIR"

# Copy the binary
echo "Installing binary to $INSTALL_DIR..."
sudo cp "target/release/lm" "$INSTALL_DIR/lm"

# Set permissions
sudo chmod +x "$INSTALL_DIR/lm"

# Create default config file
CONFIG_FILE="$HOME/.lmprep.yml"
if [ ! -f "$CONFIG_FILE" ]; then
    echo "Creating default config file at $CONFIG_FILE..."
    cat > "$CONFIG_FILE" << EOL
allowed_extensions:
  - py
  - rs
  - md
  - txt
  - js
  - ts
  - html
  - css
  - cs
  - json
  - yaml
  - go
  - java
  - cpp
  - c
delimiter: "^"
subfolder: context
zip: false
tree: true
respect_gitignore: true
EOL
fi

# Add to PATH if not already there
if [[ ":$PATH:" != *":/usr/local/bin:"* ]]; then
    echo "Adding /usr/local/bin to PATH..."
    echo 'export PATH="/usr/local/bin:$PATH"' >> "$HOME/.zshrc"
    echo 'export PATH="/usr/local/bin:$PATH"' >> "$HOME/.bashrc"
    echo "Please restart your terminal or run 'source ~/.zshrc' (or ~/.bashrc) to update PATH"
fi

echo "Installation complete! You can now use 'lm' command."
