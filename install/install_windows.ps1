# Requires -RunAsAdministrator

# Check if running as administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "Error: This script requires administrator privileges" -ForegroundColor Red
    Write-Host "Please:"
    Write-Host "1. Close PowerShell"
    Write-Host "2. Right-click on PowerShell"
    Write-Host "3. Select 'Run as administrator'"
    Write-Host "4. Navigate to the project directory"
    Write-Host "5. Run this script again"
    exit 1
}

# Check for cargo
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: cargo is not installed" -ForegroundColor Red
    Write-Host "To install Rust and Cargo:"
    Write-Host "1. Visit https://rustup.rs"
    Write-Host "2. Download and run rustup-init.exe"
    Write-Host "3. After installation, restart PowerShell and run this script again"
    exit 1
}

# Build the binary
Write-Host "Building lmprep..." -ForegroundColor Green
try {
    cargo build --release
} catch {
    Write-Host "Error: Build failed" -ForegroundColor Red
    Write-Host "Please ensure you have:"
    Write-Host "1. The latest stable Rust toolchain (run: rustup update stable)"
    Write-Host "2. Microsoft Visual Studio Build Tools installed"
    exit 1
}

# Create the installation directory
$INSTALL_DIR = "$env:ProgramFiles\lmprep"
Write-Host "Creating installation directory at $INSTALL_DIR..."
try {
    New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null
} catch {
    Write-Host "Error: Failed to create installation directory" -ForegroundColor Red
    Write-Host "Make sure you're running as administrator and have permissions to write to $env:ProgramFiles"
    exit 1
}

# Copy the binary
Write-Host "Installing binary to $INSTALL_DIR..."
try {
    Copy-Item "target\release\lm.exe" -Destination "$INSTALL_DIR\lm.exe" -Force
} catch {
    Write-Host "Error: Failed to copy binary to installation directory" -ForegroundColor Red
    Write-Host "Make sure:"
    Write-Host "1. The build completed successfully"
    Write-Host "2. You have permissions to write to $INSTALL_DIR"
    exit 1
}

# Create default config file
$CONFIG_FILE = "$env:USERPROFILE\.lmprep.yml"
if (-not (Test-Path $CONFIG_FILE)) {
    Write-Host "Creating default config file at $CONFIG_FILE..."
    @"
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
"@ | Out-File -FilePath $CONFIG_FILE -Encoding UTF8
}

# Add to PATH if not already there
$currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
if (-not $currentPath.Contains($INSTALL_DIR)) {
    Write-Host "Adding installation directory to PATH..."
    try {
        $newPath = "$currentPath;$INSTALL_DIR"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "Machine")
        $env:Path = $newPath
    } catch {
        Write-Host "Error: Failed to update system PATH" -ForegroundColor Red
        Write-Host "Make sure you have administrator privileges to modify system environment variables"
        Write-Host "The program is installed but you'll need to manually add $INSTALL_DIR to your system PATH"
    }
}

Write-Host "Installation complete!" -ForegroundColor Green
Write-Host "Please restart your terminal to use the 'lm' command."
