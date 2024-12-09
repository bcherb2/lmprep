# Installation Instructions

## Prerequisites
The installation scripts will check for these prerequisites and provide instructions if anything is missing:

### All Platforms
- Rust and Cargo (https://rustup.rs)
- Git (for cloning the repository)

### Additional Windows Requirements
- Microsoft Visual Studio Build Tools with C++ build tools
- PowerShell (comes with Windows)
- if you run into issues with Windows, try just `cargo build --release` manually, or using the pypi package

### Additional Linux Requirements
- build-essential package (for Ubuntu/Debian)

## Installation

### macOS
1. Open Terminal
2. Navigate to the project directory
3. Run:
```bash
chmod +x install/install_macos.sh
./install/install_macos.sh
```

### Ubuntu/Linux
1. Open Terminal
2. Navigate to the project directory
3. Run:
```bash
chmod +x install/install_ubuntu.sh
./install/install_ubuntu.sh
```

### Windows
1. Open PowerShell as Administrator
2. Navigate to the project directory
3. If not already enabled, you may need to allow script execution:
```powershell
Set-ExecutionPolicy RemoteSigned
```
4. Run:
```powershell
.\install\install_windows.ps1
```

## Post-Installation
- The binary will be installed to (not mandatory, put it anywhere on PATH):
  - macOS/Linux: `/usr/local/bin/lm`
  - Windows: `C:\Program Files\lmprep\lm.exe`
- A default configuration file will be created at `~/.lmprep.yml`
- The installation directory will be added to your system's PATH

## Troubleshooting

### Build Fails
- Make sure you have the latest stable Rust toolchain:
  ```bash
  rustup update stable
  ```
- On Windows, ensure you have Visual Studio Build Tools with C++ support installed
- On Ubuntu/Linux, ensure you have build-essential installed:
  ```bash
  sudo apt update && sudo apt install build-essential
  ```

### Command Not Found After Installation
- Try restarting your terminal to refresh the PATH
- On macOS/Linux, run: `source ~/.bashrc` (or `~/.zshrc` for Zsh)
- On Windows, restart PowerShell

## Verification
After installation, open a new terminal and run:
```bash
lm --help
```

This should display the help message for the lmprep tool.


## Development Commands

### Building from Source
```bash
# Build the Rust binary
cargo build --release

# Build the Python package
python -m build

# Install the package locally in editable mode
pip install -e .

# Create a wheel distribution
python -m build --wheel
```

### Publishing
```bash
# Build and publish to PyPI
python -m build
twine upload dist/*
```

### Dependencies
The Python package dependencies are managed through `pyproject.toml`. The core build dependencies are:
- setuptools >= 75.6.0
- wheel >= 0.45.1

We use specific versions to ensure reproducible builds. If you need to freeze all dependencies for development:
```bash
# Create requirements.txt with all dependencies
pip freeze > requirements.txt

# Install from requirements.txt
pip install -r requirements.txt
```

Note: For development, you'll need both Rust's Cargo and Python's pip package managers installed.