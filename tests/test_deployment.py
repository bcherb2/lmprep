#!/usr/bin/env python3
import os
import subprocess
import sys
from pathlib import Path
import pytest
import venv

def run_command(cmd, cwd=None):
    """Run a command and return its output."""
    result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"Command failed: {cmd}\nOutput: {result.stdout}\nError: {result.stderr}")
    return result.stdout

class TestDeployment:
    @pytest.fixture(autouse=True)
    def setup(self, tmp_path):
        """Set up a fresh virtual environment for each test."""
        self.venv_path = tmp_path / "venv"
        venv.create(self.venv_path, with_pip=True)
        
        # Get the Python executable path in the virtual environment
        if sys.platform == "win32":
            self.venv_python = str(self.venv_path / "Scripts" / "python.exe")
        else:
            self.venv_python = str(self.venv_path / "bin" / "python")
            
        # Get the project root (parent of tests directory)
        self.project_root = Path(__file__).resolve().parent.parent
        
        # Ensure the binary is built
        if not (self.project_root / "target" / "release").exists():
            run_command(["cargo", "build", "--release"], cwd=str(self.project_root))
            
        # Save the original working directory
        self.original_cwd = Path.cwd()
        
    def teardown_method(self):
        """Restore the original working directory after each test."""
        os.chdir(self.original_cwd)

    def test_fresh_pip_install(self):
        """Test installing the package with pip."""
        # Install the package
        run_command([self.venv_python, "-m", "pip", "install", str(self.project_root)])
        
        # Test that the binary works
        if sys.platform == "win32":
            binary = str(self.venv_path / "Scripts" / "lm.exe")
        else:
            binary = str(self.venv_path / "bin" / "lm")
            
        output = run_command([binary, "--help"])
        assert "Usage:" in output

    def test_development_install(self):
        """Test installing the package in development mode."""
        # Install in development mode
        run_command([self.venv_python, "-m", "pip", "install", "-e", str(self.project_root)])
        
        # Test that the binary works
        if sys.platform == "win32":
            binary = str(self.venv_path / "Scripts" / "lm.exe")
        else:
            binary = str(self.venv_path / "bin" / "lm")
            
        output = run_command([binary, "--help"])
        assert "Usage:" in output

    def test_config_file_locations(self, tmp_path):
        """Test config file creation in different scenarios."""
        # Install the package
        run_command([self.venv_python, "-m", "pip", "install", str(self.project_root)])
        
        # Create a test project directory
        project_dir = tmp_path / "test_project"
        project_dir.mkdir()
        
        # Initialize git repo
        run_command(["git", "init"], cwd=str(project_dir))
        
        # Run lm in the project directory
        if sys.platform == "win32":
            binary = str(self.venv_path / "Scripts" / "lm.exe")
        else:
            binary = str(self.venv_path / "bin" / "lm")
            
        run_command([binary, "tree"], cwd=str(project_dir))
        
        # Check that config file was created
        config_path = project_dir / ".lmprep.yml"
        assert config_path.exists(), f"Config file not found at {config_path}"
        assert config_path.is_file(), f"Config path exists but is not a file: {config_path}"

    def test_binary_execution(self, tmp_path):
        """Test that the binary executes correctly in different scenarios."""
        # Install the package
        run_command([self.venv_python, "-m", "pip", "install", str(self.project_root)])
        
        # Create test files and directories
        test_dir = tmp_path / "test_files"
        test_dir.mkdir()
        (test_dir / "file1.txt").write_text("test content")
        (test_dir / "file2.txt").write_text("more content")
        
        if sys.platform == "win32":
            binary = str(self.venv_path / "Scripts" / "lm.exe")
        else:
            binary = str(self.venv_path / "bin" / "lm")
        
        # Test tree command
        output = run_command([binary, "tree"], cwd=str(test_dir))
        assert "test_files" in output or "." in output
        
        # Test zip command (create a zip file of the test directory)
        output = run_command([binary, "--zip", "test_archive.zip"], cwd=str(test_dir))
        assert (test_dir / "test_archive.zip").exists()

    def test_config_file_locations_git(self, tmp_path):
        """Test config file creation in a git repository."""
        # Install the package
        run_command([self.venv_python, "-m", "pip", "install", str(self.project_root)])
        
        # Create and move to git project directory
        test_git_project = tmp_path / "git_project"
        test_git_project.mkdir()
        os.chdir(test_git_project)
        
        # Initialize git repo and run lm
        run_command(["git", "init"])
        
        if sys.platform == "win32":
            binary = str(self.venv_path / "Scripts" / "lm.exe")
        else:
            binary = str(self.venv_path / "bin" / "lm")
            
        run_command([binary, "tree"])
        
        # Check that config file was created
        config_path = test_git_project / ".lmprep.yml"
        assert config_path.exists(), f"Config not created in git root: {config_path}"
        assert config_path.is_file(), f"Config path exists but is not a file: {config_path}"

    def test_config_file_locations_regular_dir(self, tmp_path):
        """Test config file creation in a regular directory."""
        # Install the package
        run_command([self.venv_python, "-m", "pip", "install", str(self.project_root)])
        
        # Create and move to test directory
        test_dir = tmp_path / "regular_dir"
        test_dir.mkdir()
        os.chdir(test_dir)
        
        if sys.platform == "win32":
            binary = str(self.venv_path / "Scripts" / "lm.exe")
        else:
            binary = str(self.venv_path / "bin" / "lm")
            
        # Run lm and check config creation
        run_command([binary, "tree"])
        
        config_path = test_dir / ".lmprep.yml"
        assert config_path.exists(), f"Config not created in directory: {config_path}"
        assert config_path.is_file(), f"Config path exists but is not a file: {config_path}"
