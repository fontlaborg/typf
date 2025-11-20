---
title: Installation
icon: lucide/download
tags:
  - Installation
  - Setup
  - Dependencies
---

# Installation Guide

Comprehensive installation instructions for TYPF v2.0 across different platforms and use cases.

## System Requirements

### Minimum Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| Operating System | Windows 10, macOS 10.15, Ubuntu 20.04 | Windows 11, macOS 12+, Ubuntu 22.04+ |
| CPU | x86_64 or ARM64 | Multi-core x86_64 or Apple Silicon |
| RAM | 4GB | 8GB+ |
| Storage | 1GB free | 2GB+ |
| Python | 3.12 | 3.12+ |
| Rust | 1.70 | 1.75+ |

### Supported Architectures

- **x86_64**: Intel/AMD 64-bit (all platforms)
- **ARM64**: Apple Silicon, ARM Linux, Windows ARM
- **WASM32**: WebAssembly for browser/edge environments

## Prerequisites Installation

### Rust Toolchain

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version

# Install required components
rustup component add rustfmt clippy
```

### Python Environment

```bash
# Install uv (modern Python package manager)
pip install uv

# Create virtual environment
uv venv --python 3.12
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Verify Python
python --version
pip --version
```

### System Dependencies

#### Linux (Ubuntu/Debian)

```bash
# Base dependencies
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libfontconfig1-dev \
    libharfbuzz-dev \
    libfreetype6-dev \
    libpng-dev

# Optional: For Skia rendering
sudo apt-get install -y libskia-dev
```

#### Linux (Fedora/CentOS)

```bash
# Base dependencies
sudo dnf install -y \
    gcc \
    pkg-config \
    fontconfig-devel \
    harfbuzz-devel \
    freetype-devel \
    libpng-devel
```

#### macOS

```bash
# Install Xcode command line tools
xcode-select --install

# Install dependencies with Homebrew
brew install harfbuzz freetype fontconfig

# Optional: For platform-specific optimizations
brew install coretext  # CoreText backend
```

#### Windows

```powershell
# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

# Install vcpkg for C++ dependencies
git clone https://github.com/Microsoft/vcpkg.git
cd vcpkg
.\bootstrap-vcpkg.bat
.\vcpkg integrate install

# Install dependencies
.\vcpkg install harfbuzz[freetype]:x64-windows
.\vcpkg install freetype:x64-windows
```

## Installation Methods

### Method 1: Build from Source (Recommended)

This builds the complete TYPF workspace with all features:

```bash
# Clone the repository
git clone https://github.com/fontlaborg/typf.git
cd typf

# Run the comprehensive build script
./build.sh

# Manual build (alternative)
cargo build --release --workspace
cargo install --path crates/typf-cli
cd bindings/python && pip install -e .
```

**What this installs:**
- ✅ All Rust crates and backends
- ✅ `typf-cli` command-line tool
- ✅ Python bindings (`typfpy`)
- ✅ All example programs
- ✅ Development tools and linters

### Method 2: Cargo Install (CLI Only)

Install just the Rust CLI tool:

```bash
# Install from crates.io (when published)
cargo install typf-cli

# Install from source
git clone https://github.com/fontlaborg/typf.git
cd typf
cargo install --path crates/typf-cli
```

### Method 3: Python Package (Python Only)

```bash
# Install from PyPI (when published)
pip install typfpy

# Install from source
git clone https://github.com/fontlaborg/typf.git
cd typf/bindings/python
pip install -e .

# With development dependencies
pip install -e ".[dev]"
```

### Method 4: Pre-built Binaries

Download pre-built binaries for your platform:

```bash
# Download latest release
wget https://github.com/fontlaborg/typf/releases/latest/download/typf-cli-x86_64-unknown-linux-gnu.tar.gz
tar xzf typf-cli-x86_64-unknown-linux-gnu.tar.gz

# Add to PATH
sudo cp typf-cli /usr/local/bin/
```

## Feature Flag Configuration

### Available Features

```toml
[features]
# Shaping backends
shaping-hb = ["harfbuzz_rs"]           # HarfBuzz (default)
shaping-coretext = []                  # CoreText (macOS)
shaping-directwrite = []               # DirectWrite (Windows)
shaping-icu-hb = ["harfbuzz_rs"]       # ICU + HarfBuzz
shaping-none = []                      # No shaping (testing)

# Rendering backends
render-skia = ["skia-safe"]            # Skia (default)
render-coregraphics = []               # CoreGraphics (macOS)
render-direct2d = []                   # Direct2D (Windows)
render-orge = []                       # Orge rasterizer
render-zeno = []                       # Zeno GPU renderer
render-json = []                       # JSON data export

# Export formats
export-png = ["image"]                 # PNG images (default)
export-svg = []                        # SVG vectors
export-pnm = []                        # PNM raw data
export-jpeg = ["image/jpeg"]           # JPEG images
export-pdf = []                        # PDF documents

# Build presets
minimal = ["shaping-none", "render-orge", "export-pnm"]
default = ["shaping-hb", "render-skia", "export-png"]
full = [
    "shaping-hb", "shaping-coretext", "shaping-directwrite",
    "render-skia", "render-coregraphics", "render-direct2d",
    "export-png", "export-svg", "export-json"
]
```

### Custom Builds

```bash
# Minimal build (smallest size, basic features)
cargo build --release --no-default-features --features minimal

# Platform-optimized build (macOS)
cargo build --release --features "shaping-coretext,render-coregraphics,export-png"

# Full-featured build (all backends)
cargo build --release --all-features

# Custom selection
cargo build --release --features "shaping-hb,render-skia,export-png,export-svg"
```

## Configuration

### Environment Setup

```bash
# Set default backends
export TYPF_SHAPER=harfbuzz
export TYPF_RENDERER=skia

# Set cache location
export TYPF_CACHE_DIR=$HOME/.typf/cache

# Performance tunables
export TYPF_MAX_FONTS=100
export TYPF_MAX_GLYPHS=10000
```

### Configuration File

Create `~/.typf/config.toml`:

```toml
[default]
# Default backends to use
shaper = "harfbuzz"
renderer = "skia"

# Rendering defaults
font_size = 24.0
color = "#000000"
dpi = 72.0

[cache]
# Cache configuration
max_fonts = 100
max_glyphs = 10000
cache_dir = "~/.typf/cache"
ttl = 3600  # seconds

[performance]
# Performance settings
enable_simd = true
thread_pool_size = 4
memory_limit = "1GB"

[fonts]
# Font configuration
system_font_dirs = [
    "/System/Library/Fonts",     # macOS
    "/usr/share/fonts",          # Linux
    "C:\\Windows\\Fonts"         # Windows
]
fallback_fonts = [
    "Arial",
    "DejaVu Sans",
    "Noto Sans"
]
```

### Python Configuration

```python
# ~/.typf/python_config.py
CONFIG = {
    "default_shaper": "harfbuzz",
    "default_renderer": "skia",
    "cache_size": 1000,
    "enable_profiling": False,
    "log_level": "INFO"
}
```

## Verification

### Installation Verification

```bash
# Check CLI installation
typf-cli --version
typf-cli --help

# List available backends
typf-cli backend-list

# Test basic rendering
typf-cli render --text "Hello World" --font /System/Library/Fonts/Arial.ttf

# Python verification
python -c "import typfpy; print(typfpy.__version__)"
```

### System Integration Test

```bash
# Run comprehensive test suite
cargo test --workspace

# Run integration tests
./typf-tester/typfme.py render

# Performance benchmark
./typf-tester/typfme.py bench
```

## Platform-Specific Notes

### macOS

```bash
# Enable CoreText backend
cargo build --features shaping-coretext,render-coregraphics

# System font integration
typf-cli font-list | grep -i arial

# Notarization for distribution (if building for distribution)
codesign --deep --force --verify --verbose --sign "Developer ID" target/release/typf-cli
xcrun altool --notarize-app --primary-bundle-id "com.fontlab.typf" --file typf-cli.zip
```

### Windows

```powershell
# Enable DirectWrite backend
cargo build --features shaping-directwrite,render-direct2d

# Font path handling (use forward slashes or raw strings)
typf-cli render --text "Hello" --font "C:/Windows/Fonts/arial.ttf"

# PowerShell execution policy (if needed)
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Linux

```bash
# Font discovery setup
fc-cache -fv  # Rebuild font cache
typf-cli font-list

# Install additional fonts for testing
sudo apt-get install fonts-liberation fonts-dejavu-core fonts-noto

# System integration
sudo cp target/release/typf-cli /usr/local/bin/
```

### WebAssembly

```bash
# Build for WASM
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --features minimal

# Build with wasm-bindgen (for JavaScript integration)
cargo build --target wasm32-unknown-unknown --features wasm-bindgen
```

## Troubleshooting

### Common Build Issues

#### Missing System Dependencies

```bash
# Error: Could not find `harfbuzz`
# Solution:
sudo apt-get install libharfbuzz-dev  # Linux
brew install harfbuzz                 # macOS

# Error: Could not find `freetype`
sudo apt-get install libfreetype6-dev
brew install freetype
```

#### Rust Compilation Errors

```bash
# Update Rust toolchain
rustup update stable
rustup component add rustfmt clippy

# Clean build cache
cargo clean
 cargo build --release
```

#### Python Build Errors

```bash
# Ensure Python development headers
sudo apt-get install python3-dev  # Linux
# Already included with Xcode tools on macOS

# Ensure virtual environment is active
source .venv/bin/activate
pip install --upgrade pip setuptools wheel
```

### Runtime Issues

#### Font Not Found

```bash
# Check font permissions
ls -la /path/to/font.ttf

# Test font loading
typf-cli font-check --font /path/to/font.ttf

# List available fonts
typf-cli font-list
```

#### Backend Not Available

```bash
# Check compiled features
typf-cli --help | grep -A 10 "Available backends"

# Rebuild with required features
cargo build --release --features shaping-hb,render-skia
```

#### Performance Issues

```bash
# Check system resources
free -h  # Memory
nproc    # CPU cores

# Clear caches
typf-cli cache-clear

# Use performance build
cargo build --release --features full
```

### Getting Help

#### Build Diagnostics

```bash
# Verbose build output
cargo build --release --verbose

# Check feature resolution
cargo tree --features full

# Dependency information
cargo tree --duplicate
```

#### Community Support

- **GitHub Issues**: [Report bugs](https://github.com/fontlaborg/typf/issues)
- **GitHub Discussions**: [Ask questions](https://github.com/fontlaborg/typf/discussions)
- **Discord**: [Live chat](https://discord.gg/typf)
- **Email**: support@fontlab.com

## Next Steps

After successful installation:

1. [**Quick Start Guide**](02-quick-start.md) - Your first text rendering
2. [**Architecture Overview**](03-architecture-overview.md) - Understand the system
3. [**Configuration Options**](20-configuration-options.md) - Customize your setup
4. [**API Reference**](17-rust-api.md) - Programmatic usage

---

**TYPF is now installed!** You're ready to start exploring high-performance text shaping and rendering. Let's begin with the Six-Stage Pipeline next.
