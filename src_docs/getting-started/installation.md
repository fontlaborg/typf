# Installation

## Rust Library

Add to your `Cargo.toml`:

```toml
[dependencies]
typf = "0.3"

# Optional features
typf = { version = "0.3", features = ["mac", "icu"] }
```

Available features:

- `mac` - CoreText backend (macOS only)
- `windows` - DirectWrite backend (Windows only)
- `icu` - HarfBuzz + ICU backend (cross-platform)
- `python` - Python bindings via PyO3
- `svg` - SVG output support
- `png` - PNG output support

## Python Bindings

### From Source (Development)

```bash
# Clone repository
git clone https://github.com/fontlaborg/typf.git
cd typf

# Create virtual environment
uv venv --python 3.12
source .venv/bin/activate  # macOS/Linux

# Build Python bindings
cd python
maturin develop --release --features "python,icu,mac"  # macOS
# maturin develop --release --features "python,icu"    # Linux
# maturin develop --release --features "python,windows"  # Windows

# Verify
python -c "import typf; print(typf.__version__)"
```

### Platform-Specific Requirements

**macOS:**
```bash
xcode-select --install
```

**Linux (Debian/Ubuntu):**
```bash
sudo apt install python3-dev libharfbuzz-dev libfreetype6-dev
```

**Linux (Fedora/RHEL):**
```bash
sudo dnf install python3-devel harfbuzz-devel freetype-devel
```

**Windows:**
Install Visual Studio Build Tools with Python development workload.

## CLI Tool

```bash
# Install from source
cargo install --path typf-cli

# Verify
typf --version
```

## Next Steps

- [Quick Start](quick-start.md) - Render your first text
- [Python Bindings](python-bindings.md) - Using TYPF from Python

---

**Made by [FontLab](https://www.fontlab.com/)**
