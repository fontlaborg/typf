# Python Bindings

TYPF provides Python bindings via PyO3, allowing you to use the Rust rendering engine from Python with minimal overhead.

## Build Requirements

**Critical:** Python bindings MUST be built inside an active virtual environment. System Python builds will fail with linker errors.

```bash
# 1. Create venv (REQUIRED)
cd github.fontlaborg/typf
uv venv --python 3.12

# 2. Activate venv (REQUIRED)
source .venv/bin/activate  # macOS/Linux
# .venv\Scripts\activate   # Windows

# 3. Install maturin IN the venv
uv pip install maturin

# 4. Build with platform features
cd python
maturin develop --release --features "python,icu,mac"  # macOS
# maturin develop --release --features "python,icu"    # Linux
# maturin develop --release --features "python,windows"  # Windows

# 5. Verify
python -c "import typf; print(typf.__version__)"
```

## Troubleshooting

### Linker errors / undefined symbols

**Cause:** Building outside venv or missing Python dev headers

**Solutions:**

- **macOS:** `xcode-select --install`
- **Linux (Debian/Ubuntu):** `sudo apt install python3-dev`
- **Linux (Fedora/RHEL):** `sudo dnf install python3-devel`
- **Windows:** Install Visual Studio Build Tools with Python development workload
- **Universal:** Activate venv before running `maturin develop`

### Import errors after build

- Verify venv is activated: `which python` should show `.venv/bin/python`
- Rebuild: `cd python && maturin develop --release --features "python,icu,mac"`

### Feature flag selection

- **macOS:** Use `mac` feature (CoreText backend)
- **Linux:** Use `icu` feature (HarfBuzz+ICU backend)
- **Windows:** Use `windows` feature (DirectWrite backend)
- **Cross-platform:** Always include `python` and `icu` features

## Usage Examples

### Basic Rendering

```python
import typf

# Render text to PNG
result = typf.render_text(
    font_path="/path/to/font.ttf",
    text="Hello World",
    size=48,
    output_path="hello.png"
)
print(f"Rendered {result.width}x{result.height} image")
```

### Advanced Options

```python
import typf

# Configure rendering options
result = typf.render_text(
    font_path="/path/to/font.ttf",
    text="Custom Render",
    size=64,
    dpi=144,
    color=(255, 0, 0),  # RGB
    background=(255, 255, 255),  # White background
    padding=20,
    output_path="custom.png"
)
```

### Variable Fonts

```python
import typf

# Render with variable font coordinates
result = typf.render_text(
    font_path="/path/to/variable.ttf",
    text="Variable Font",
    size=48,
    variations={"wght": 700, "wdth": 125},
    output_path="variable.png"
)
```

### NumPy Integration

```python
import typf
import numpy as np

# Get image as NumPy array
array = typf.render_to_array(
    font_path="/path/to/font.ttf",
    text="NumPy",
    size=48
)

print(f"Shape: {array.shape}")  # (height, width, channels)
print(f"Dtype: {array.dtype}")  # uint8
```

## API Reference

See [Python API](../api/python.md) for complete API documentation.

---

**Made by [FontLab](https://www.fontlab.com/)**
