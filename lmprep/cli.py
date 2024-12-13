#!/usr/bin/env python3
import os
import platform
import subprocess
import sys
import argparse
from pathlib import Path
from importlib.resources import files, as_file

def get_binary_name():
    """Get the appropriate binary name for the current platform."""
    system = platform.system().lower()
    
    if system == "windows":
        return "lm.exe"
    return "lm"

def get_binary_path():
    """Get the appropriate binary path for the current platform."""
    system = platform.system().lower()
    machine = platform.machine().lower()
    
    if system == "windows":
        return "binaries/win_amd64/lm.exe"
    elif system == "linux":
        return "binaries/linux_x86_64/lm"
    elif system == "darwin":
        # macOS uses universal binary now
        return "binaries/darwin_universal2/lm"
    raise RuntimeError(f"Unsupported platform: {system} {machine}")

def get_binary():
    """Get the binary path from package resources."""
    try:
        binary_path = get_binary_path()
        resource_path = files("lmprep").joinpath(binary_path)
        if not resource_path.is_file():
            raise RuntimeError(f"Binary not found at {resource_path}")
        return str(resource_path)
    except Exception as e:
        raise RuntimeError(f"Binary not found in package: {e}")

def find_config_location():
    """Find the appropriate location for the config file."""
    # Always use the current working directory
    return Path.cwd()

def get_default_config():
    """Get the default configuration from various possible locations."""
    # First check the current directory for .lmprep.yml
    cwd = Path.cwd()
    config_file = cwd / ".lmprep.yml"
    if config_file.is_file():
        return config_file.read_text()
    
    # Check package default as last resort
    try:
        config_path = files("lmprep").joinpath("default_config.yml")
        with as_file(config_path) as default_config:
            if default_config.exists():
                return default_config.read_text()
    except Exception as e:
        print(f"Warning: Could not read package default config: {e}")
    
    raise RuntimeError("No config file found. Run any command to create one.")

def create_config():
    """Create default config files if they don't exist."""
    # Get default config content from package
    try:
        config_path = files("lmprep").joinpath("default_config.yml")
        with as_file(config_path) as default_config:
            if not default_config.exists():
                raise RuntimeError("Default config template not found in package. Please reinstall lmprep.")
            config_content = default_config.read_text()
    except Exception as e:
        raise RuntimeError(f"Could not read default config template: {e}")
    
    # Find the appropriate location for the config
    config_dir = find_config_location()
    config_file = config_dir / ".lmprep.yml"
    
    # Don't overwrite existing config
    if config_file.exists():
        return
    
    try:
        config_file.write_text(config_content)
        print(f"Created configuration in: {config_file}")
    except (PermissionError, OSError) as e:
        raise RuntimeError(f"Could not create config file at {config_file}: {e}")

def main():
    """Main entry point for the lmprep CLI."""
    try:
        binary_path = get_binary()
        
        # Always try to create config unless showing help/version
        if len(sys.argv) == 1 or sys.argv[1] not in ['-h', '--help', '-V', '--version']:
            create_config()
        
        # Run the binary with all arguments
        try:
            result = subprocess.run(
                [binary_path] + sys.argv[1:],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                timeout=30
            )
            
            if result.stdout:
                print(result.stdout)
            if result.stderr:
                print(result.stderr, file=sys.stderr)
                
            sys.exit(result.returncode)
        except subprocess.TimeoutExpired:
            print("\nError: Command timed out after 30 seconds")
            sys.exit(1)
            
    except KeyboardInterrupt:
        print("\nOperation cancelled by user")
        sys.exit(1)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
