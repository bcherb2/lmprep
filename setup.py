import os
import platform
import shutil
import subprocess
import sys
from setuptools import setup, Command
from setuptools.command.build_py import build_py
from wheel.bdist_wheel import bdist_wheel

def get_platform_tag():
    """Get the platform tag for the wheel"""
    platform_override = os.environ.get('PLATFORM')
    if platform_override:
        return platform_override
        
    system = platform.system().lower()
    
    if system == "windows":
        return "win_amd64"
    elif system == "linux":
        return "manylinux2014_x86_64"
    elif system == "darwin":
        return "macosx_10_9_universal2"
    else:
        raise RuntimeError(f"Unsupported platform: {system}")

def get_binary_paths(platform_tag):
    """Get the binary paths to include based on platform"""
    if platform_tag == "win_amd64":
        return ["binaries/win_amd64/lm.exe"]
    elif platform_tag == "manylinux2014_x86_64":
        return ["binaries/linux_x86_64/lm"]
    elif platform_tag.startswith("macosx") and "universal2" in platform_tag:
        return ["binaries/darwin_universal2/lm"]
    elif platform_tag.startswith("macosx") and "x86_64" in platform_tag:
        return ["binaries/darwin_x86_64/lm"]
    elif platform_tag.startswith("macosx") and "arm64" in platform_tag:
        return ["binaries/darwin_arm64/lm"]
    else:
        raise RuntimeError(f"Unsupported platform tag: {platform_tag}")

class CustomBdistWheel(bdist_wheel):
    def finalize_options(self):
        super().finalize_options()
        self.root_is_pure = False
        # Set plat_name to force the correct platform tag
        self.plat_name = get_platform_tag()
        
    def get_tag(self):
        # Override get_tag to ensure we use our custom platform tag
        python_tag = 'py3'
        abi_tag = 'none'
        platform_tag = self.plat_name
        return python_tag, abi_tag, platform_tag

class CustomBuildPy(build_py):
    """Custom build command that builds Rust binary before Python package"""
    
    def run(self):
        """Run the build process"""
        print("=== Starting custom build process ===")
        
        # Get platform-specific paths
        platform_tag = get_platform_tag()
        binary_paths = get_binary_paths(platform_tag)
        
        # Only include platform-specific binaries in package data
        self.distribution.package_data = {
            "lmprep": binary_paths + ["__init__.py", "cli.py"]
        }
        
        print(f"Building for platform: {platform_tag}")
        print(f"Including binaries: {binary_paths}")
        
        # Run the original build_py
        super().run()

if __name__ == "__main__":    
    setup(
        name="lmprep",
        version="0.3.0",
        packages=["lmprep"],
        include_package_data=True,
        cmdclass={
            'build_py': CustomBuildPy,
            'bdist_wheel': CustomBdistWheel,
        },
        entry_points={
            'console_scripts': [
                'lm=lmprep.cli:main',
            ],
        },
    )