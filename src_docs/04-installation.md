---
title: Installation
icon: lucide/download
tags:
  - Installation
  - Setup
  - Dependencies
---

# Installation

Install Typf and start rendering text.

## Quick Install

```bash
# Clone and build everything
git clone https://github.com/fontlaborg/typf.git
cd typf
./build.sh

# Or install components separately
cargo install --path crates/typf-cli
cd bindings/python && pip install -e .
```

## Requirements

- **OS**: Windows 10+, macOS 10.15+, Ubuntu 20.04+
- **CPU**: x86_64 or ARM64
- **RAM**: 4GB minimum, 8GB recommended
- **Rust**: 1.70+
- **Python**: 3.12+

## Install Dependencies

### Linux (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libfontconfig1-dev \
    libharfbuzz-dev \
    libfreetype6-dev \
    libpng-dev
```

### Linux (Fedora/CentOS)

```bash
sudo dnf install -y \
    gcc \
    pkg-config \
    fontconfig-devel \
    harfbuzz-devel \
    freetype-devel \
    libpng-devel
```

### macOS

```bash
xcode-select --install
brew install harfbuzz freetype fontconfig
```

### Windows

Install Visual Studio Build Tools, then use vcpkg:

```powershell
git clone https://github.com/Microsoft/vcpkg.git
cd vcpkg
.\bootstrap-vcpkg.bat
.\vcpkg integrate install
.\vcpkg install harfbuzz[freetype]:x64-windows
```

## Installation Options

### Build from Source

```bash
git clone https://github.com/fontlaborg/typf.git
cd typf
cargo build --release --workspace
cargo install --path crates/typf-cli
cd bindings/python && pip install -e .
```

### Cargo Only (CLI)

```bash
cargo install typf-cli
```

### Python Only

```bash
pip install typfpy
```

### Pre-built Binaries

Download from [GitHub releases](https://github.com/fontlaborg/typf/releases/latest).

## Feature Selection

Build only what you need:

```bash
# Minimal build
cargo build --release --features minimal

# Platform optimized (macOS)
cargo build --release --features "shaping-coretext,render-coregraphics"

# Full featured
cargo build --release --all-features

# Custom selection
cargo build --release --features "shaping-hb,render-skia,export-png,export-svg"
```

## Configuration

Set defaults via environment:

```bash
export Typf_SHAPER=harfbuzz
export Typf_RENDERER=skia
export Typf_CACHE_DIR=$HOME/.typf/cache
```

Or create `~/.typf/config.toml`:

```toml
[default]
shaper = "harfbuzz"
renderer = "skia"
font_size = 24.0
color = "#000000"
dpi = 72.0

[cache]
max_fonts = 100
max_glyphs = 10000
cache_dir = "~/.typf/cache"
```

## Verify Installation

```bash
# Check CLI
typf-cli --version
typf-cli --help
typf-cli render --text "Hello World" --font /path/to/font.ttf

# Check Python
python -c "import typfpy; print(typfpy.__version__)"

# Run tests
cargo test --workspace
./typf-tester/typfme.py render
```

## Platform Notes

### macOS

```bash
# Enable native backends
cargo build --features shaping-coretext,render-coregraphics

# System fonts
typf-cli font-list | grep -i arial
```

### Windows

```bash
# Enable native backends
cargo build --features shaping-directwrite,render-direct2d

# Font paths (forward slashes work)
typf-cli render --text "Hello" --font "C:/Windows/Fonts/arial.ttf"
```

### Linux

```bash
# Rebuild font cache
fc-cache -fv
sudo apt-get install fonts-liberation fonts-dejavu-core fonts-noto
```

## Troubleshooting

**Missing dependencies?** Install the system packages listed above.

**Build fails?** Update Rust and clean cache:

```bash
rustup update stable
cargo clean
cargo build --release
```

**Font not found?** Check permissions and path:

```bash
ls -la /path/to/font.ttf
typf-cli font-check --font /path/to/font.ttf
```

**Backend missing?** Rebuild with features:

```bash
cargo build --release --features shaping-hb,render-skia
```

## Next Steps

- [Quick Start](02-quick-start.md) - Your first rendering
- [Architecture](03-architecture-overview.md) - How it works
- [Six-Stage Pipeline](05-six-stage-pipeline.md) - Core concepts

---

Typf is installed. Start rendering text.
