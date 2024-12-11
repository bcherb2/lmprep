#!/usr/bin/env python3
import os
import platform
import subprocess
import sys
from pathlib import Path
from importlib.resources import files

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

def get_default_config():
    """Get the default configuration from the default_config.yml file."""
    default_config_path = Path(__file__).parent.parent / "default_config.yml"
    if not default_config_path.exists():
        raise RuntimeError("Default config file not found. Please reinstall lmprep.")
    return default_config_path.read_text()

def create_config():
    """Create default config files if they don't exist."""
    # Get default config content
    config_content = get_default_config()
    
    # Try to create config in home directory
    home = Path.home()
    home_config = home / ".lmprep.yml"
    
    if not home_config.exists():
        try:
            home_config.write_text(config_content)
            print(f"Created default configuration at {home_config}")
        except (PermissionError, OSError) as e:
            print(f"Warning: Could not create config in home directory ({e})")
            print("Will fall back to local config files in your project directories")
            print("To enable a global config, create .lmprep.yml in your home directory")
    
    # Create config in current directory
    local_config = Path(".lmprep.yml")
    if not local_config.exists():
        try:
            local_config.write_text(config_content)
            print(f"Created local configuration at {local_config}")
        except (PermissionError, OSError) as e:
            print(f"Warning: Could not create local config file ({e})")

def main():
    """Main entry point for the CLI."""
    try:
        binary_path = get_binary()
        create_config()
          
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
