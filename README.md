# LMPrep 

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![PyPI version](https://badge.fury.io/py/lmprep.svg)](https://badge.fury.io/py/lmprep)
[![PyPI - Python Version](https://img.shields.io/pypi/pyversions/lmprep)](https://pypi.org/project/lmprep/)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Platform Support](https://img.shields.io/badge/platform-windows%20%7C%20macos%20%7C%20linux-lightgrey)](https://github.com/bcherb2/lmprep)

A lightning-fast utility for preparing and organizing your code for use with LLMs like Claude Projects.  LMPrep will collect and rename all of your project files to a flat directory, but preserving the structure within the filenames.  

For example, a file at `src/models/user.py` will be renamed to `src^models^user.py` in the output directory.


https://github.com/user-attachments/assets/27d49b03-76a0-4742-9883-e361b73bc10e


## Features

- **Smart File Organization**: Automatically flattens complex directory structures while preserving path information in the filenames and in a file tree
- **Configurable Filtering**: Specify which file extensions to include in your dataset to limit context size
- **Path Preservation**: Uses customizable delimiters to maintain original path information in filenames
- **Git-Aware**: Respects `.gitignore` patterns to exclude unwanted files or secrets
- **Flexible Output**: Generate individual files or create a zip archive
- **Visual Tree View**: Visualize your source and output file structure, or send the file tree to the LLM
- **Fast & Efficient**: Written in Rust for maximum performance

## Quick Start

### Installation

The easiest way to install LMPrep is to get it from PyPi:

```bash
pip install lmprep
```
wheels are built for Windows, Linux, and MacOS.

#### Manaul / Install Script

1. Download the latest release for your platform from [Releases](https://github.com/bcherb2/lmprep/releases):
   - Windows: `lm-x86_64-pc-windows-msvc.zip`
   - Linux: `lm-x86_64-unknown-linux-gnu.tar.gz`
   - macOS: `lm-x86_64-apple-darwin.tar.gz`

2. Install the binary:

**Linux/macOS**:
```bash
# Extract and copy binary
tar xzf lm-x86_64-*-*.tar.gz
sudo mv lm /usr/local/bin/

# Create config file
curl -O https://raw.githubusercontent.com/bcherb2/lmprep/main/src/config-example.yaml
mv config-example.yaml ~/.lmprep.yml
```

**Windows** (in PowerShell, run as Administrator):
```powershell
# Extract and copy binary
Expand-Archive lm-x86_64-pc-windows-msvc.zip
New-Item -ItemType Directory -Force -Path "C:\Program Files\lmprep"
Move-Item -Force lm.exe "C:\Program Files\lmprep\lm.exe"
$env:Path += ";C:\Program Files\lmprep"

# Create config file
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/bcherb2/lmprep/main/src/config-example.yaml" -OutFile "$env:USERPROFILE\.lmprep.yml"
```

3. Verify installation:
```bash
lm --help
```

#### Alternative: Build from Source

If you have Rust installed, you can build from source:
```bash
git clone https://github.com/bcherb2/lmprep
cd lmprep
cargo build --release
```
The binary will be in `target/release/lm` (or `lm.exe` on Windows). Follow step 2 above to set up the config file.

### Basic Usage

```bash
# Organize files in current directory
lm .

# Organize files from a specific directory
lm /path/to/source

# Use a custom config file
lm . -c /path/to/.lmprep.yml

# Create a zip archive instead of of individual files
lm . --zip
```

## Configuration

Create a `.lmprep.yml` file in your home directory to customize behavior, or create one in your project root directory. Here's an example:

```yaml
allowed_extensions:
  - py
  - rs
  - md
  - txt
delimiter: "^"
subfolder: context
zip: false
tree: true
respect_gitignore: true
```

>NOTE: The install script will create a default config file at `~/.lmprep.yml`

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `allowed_extensions` | File extensions to include | `[]` (all files) |
| `delimiter` | Character used to represent path hierarchy | `^` |
| `subfolder` | Output directory name within project | `context` |
| `zip` | Create zip archive instead of files | `false` |
| `tree` | Show file tree visualization | `true` |
| `respect_gitignore` | Honor .gitignore patterns | `true` |

## Command Line Options

```bash
lm [OPTIONS] [SOURCE]

Arguments:
  [SOURCE]  Source directory to organize files from [default: .]

Options:
  -c, --config <FILE>     Path to config file
  -s, --subfolder <NAME>  Override the subfolder name from config
  -z, --zip              Create a zip file instead of individual files
  -t, --tree             Show file tree of source and output
  -v, --verbose          Enable verbose logging
  -h, --help             Print help
  -V, --version          Print version
```

## Use Cases

- **ML Dataset Preparation**: Organize your training data into a flat structure while preserving context (works especially well with Claude Projects)
- **Code Analysis**: Prepare source code for analysis by LLMs
- **Document Processing**: Organize and prepare document collections for processing, logs, etc.
- **Version Control**: Easily create clean snapshots of your codebase for archival in zip format

## Building from Source

1. Install Rust using [rustup](https://rustup.rs/)
2. Clone the repository:
   ```bash
   git clone https://github.com/bcherb2/lmprep.git
   cd lmprep
   ```
3. Build the project:
   ```bash
   cargo build --release
   ```
4. The binary will be available at `target/release/lm`, copy it and add it to your PATH
5. Create the `.lmprep.yml` file in your home directory or project root

>
>NOTE: see [install/BUILD.md](https://github.com/bcherb2/lmprep/blob/main/install/BUILD.md) for more in depth building instructions.
>

## FAQ

**Q: Why use LMPrep instead of just copying files?**
A: LMPrep preserves directory structure information in filenames, making it easier for LLMs to understand file relationships and context.  Sure, you can do this manually, but it gets tedious.

**Q: How does path flattening work?**
A: A file at `src/models/user.py` becomes `src^models^user.py` in the output directory (using default delimiter).  Changing the delimiter to `+` would result in `src+models+user.py`.

**Q: Can I exclude certain files or directories?**
A: Yes! LMPrep respects `.gitignore` patterns and allows you to specify allowed file extensions.

**Q: Is it safe to use on large directories?**
A: Yes! LMPrep is written in Rust for performance and memory efficiency, making it suitable for large datasets.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
