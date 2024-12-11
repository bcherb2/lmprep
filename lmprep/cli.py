#!/usr/bin/env python3
import os
import platform
import subprocess
import sys
import shutil
from pathlib import Path
from importlib import resources
from importlib.resources import files

def get_binary_name():
    """Get the appropriate binary name for the current platform."""
    system = platform.system().lower()
    machine = platform.machine().lower()
    
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
        if machine == "arm64":
            return "binaries/darwin_arm64/lm"
        else:
            return "binaries/darwin_x86_64/lm"
    raise RuntimeError(f"Unsupported platform: {system} {machine}")

def install_binary():
    """Install the binary from package resources."""
    try:
        binary_name = get_binary_name()
        binary_path = get_binary_path()
        
        try:
            resource_path = files("lmprep").joinpath(binary_path)
            if not resource_path.is_file():
                raise RuntimeError(f"Binary not found at {resource_path}")
            resource_path = str(resource_path)
        except Exception as e:
            raise RuntimeError(f"Binary not found in package: {e}")
            
        if platform.system().lower() == "windows":
            install_dir = os.path.join(os.environ.get("ProgramFiles", "C:\\Program Files"), "lmprep")
        else:
            if os.access("/usr/local/bin", os.W_OK):
                install_dir = "/usr/local/bin"
            else:
                install_dir = os.path.expanduser("~/.local/bin")
                os.makedirs(install_dir, exist_ok=True)
                
        target_path = os.path.join(install_dir, binary_name)
        os.makedirs(os.path.dirname(target_path), exist_ok=True)
        
        shutil.copy2(resource_path, target_path)
        if platform.system().lower() != "windows":
            os.chmod(target_path, 0o755)
            
        print(f"Successfully installed lm binary to {target_path}")
        return target_path
            
    except Exception as e:
        raise RuntimeError(f"Failed to install binary: {e}")

def get_default_config():
    """Get the default configuration from the default_config.yml file."""
    default_config_path = Path(__file__).parent.parent / "default_config.yml"
    if not default_config_path.exists():
        raise RuntimeError("Default config file not found. Please reinstall lmprep.")
    return default_config_path.read_text()

def create_config():
    """Create default config files if they don't exist."""
    # Create config in home directory
    home = Path.home()
    home_config = home / ".lmprep.yml"
    
    # Create config in current directory
    local_config = Path(".lmprep.yml")
    
    # Get default config content
    config_content = get_default_config()
    
    # Create in home directory if it doesn't exist
    if not home_config.exists():
        home_config.write_text(config_content)
        print(f"Created default configuration at {home_config}")
        
    # Create in current directory if it doesn't exist
    if not local_config.exists():
        local_config.write_text(config_content)
        print(f"Created local configuration at {local_config}")

def main():
    """Main entry point for the CLI."""
    try:
        binary_path = install_binary()
        create_config()
          
        try:
            result = subprocess.run(
                [binary_path] + sys.argv[1:],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                timeout=30
            )
            
            if result.returncode == 0:
                print("lm completed successfully")
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
