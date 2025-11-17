# Quick Start

## CLI Usage

### Basic Rendering

```bash
typf render \
  --font=/path/to/font.ttf \
  --text="Hello World" \
  --size=48 \
  --output=hello.png
```

### With Options

```bash
typf render \
  --font=/path/to/font.ttf \
  --text="Custom Render" \
  --size=64 \
  --dpi=144 \
  --padding=20 \
  --auto-size \
  --crop \
  --output=custom.png
```

### Variable Fonts

```bash
typf render \
  --font=/path/to/variable.ttf \
  --text="Variable Font" \
  --size=48 \
  --variations="wght=700,wdth=125" \
  --output=variable.png
```

## Python Usage

### Basic Example

```python
import typf

result = typf.render_text(
    font_path="/path/to/font.ttf",
    text="Hello World",
    size=48,
    output_path="hello.png"
)
```

### Advanced Example

```python
import typf

result = typf.render_text(
    font_path="/path/to/font.ttf",
    text="Advanced",
    size=64,
    dpi=144,
    color=(255, 0, 0),
    background=(255, 255, 255),
    padding=20,
    auto_size=True,
    crop=True,
    output_path="advanced.png"
)
```

## Rust Usage

```rust
use typf::render::{RenderOptions, render_to_png};

let options = RenderOptions::new()
    .font_path("/path/to/font.ttf")
    .text("Hello World")
    .size(48);

render_to_png(&options, "hello.png")?;
```

## Next Steps

- [Variable Fonts](../guide/variable-fonts.md) - Working with variable fonts
- [Output Formats](../guide/output-formats.md) - Different output options
- [Backend Comparison](../backends/comparison.md) - Choosing the right backend

---

**Made by [FontLab](https://www.fontlab.com/)**
