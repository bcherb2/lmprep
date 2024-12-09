#!/usr/bin/env python3
import os
import shutil
import subprocess
import sys
from pathlib import Path
from wheel.wheelfile import WheelFile

def clean():
    """Clean build artifacts"""
    print("=== Cleaning build artifacts ===")
    paths = ["dist", "build", "*.egg-info"]
    for path in paths:
        for p in Path(".").glob(path):
            if p.is_dir():
                shutil.rmtree(p)
            else:
                p.unlink()
    print("Clean complete\n")

def build():
    """Build the wheel"""
    print("=== Building wheel ===")
    result = subprocess.run(
        [sys.executable, "-m", "build", "--wheel", "--no-isolation"],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    if result.stderr:
        print("STDERR:", result.stderr)
    if result.returncode != 0:
        raise RuntimeError("Build failed")
    print("Build complete\n")

def check_wheel():
    """Check the wheel file for platform tags"""
    print("=== Checking wheel ===")
    wheels = list(Path("dist").glob("*.whl"))
    if not wheels:
        raise RuntimeError("No wheel found in dist/")
    
    for wheel in wheels:
        print(f"\nAnalyzing wheel: {wheel.name}")
        with WheelFile(wheel) as wf:
            print(f"Tags: {wf.parsed_filename.groupdict()}")
            print(f"Contents:")
            for name in wf.namelist():
                print(f"  {name}")
    print("\nWheel check complete")

def main():
    """Main test function"""
    try:
        clean()
        build()
        check_wheel()
    except Exception as e:
        print(f"ERROR: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
