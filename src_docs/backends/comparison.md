# Backend Comparison

TYPF supports multiple rendering backends, each optimized for different platforms and use cases.

## Overview

| Backend | Platform | Performance | Completeness | Use Case |
|---------|----------|-------------|--------------|----------|
| **CoreText** | macOS | Excellent | ✅ Full | macOS primary |
| **DirectWrite** | Windows | Excellent | ✅ Full | Windows primary |
| **HarfBuzz+ICU** | All | Good | ✅ Full | Cross-platform fallback |
| **orge** | All | Excellent* | ⚠️ Partial | Custom rasterization (in progress) |
| **tiny-skia** | All | Good | ✅ Full | Vector rendering (feature-gated) |
| **zeno** | All | Good | ⚠️ Partial | Alternative rasterizer |

\* orge shows excellent performance where implemented, but coverage is incomplete

## Platform Recommendations

### macOS
**Primary:** CoreText  
**Fallback:** HarfBuzz+ICU

CoreText provides the best performance and native macOS text rendering behavior. Use HarfBuzz+ICU for cross-platform consistency.

### Windows
**Primary:** DirectWrite  
**Fallback:** HarfBuzz+ICU

DirectWrite offers optimal Windows text rendering with ClearType support. Use HarfBuzz+ICU for cross-platform consistency.

### Linux
**Primary:** HarfBuzz+ICU  
**Alternative:** tiny-skia

HarfBuzz+ICU is the recommended choice for Linux, providing good performance and full OpenType support.

## Feature Support

### CoreText (macOS)
- ✅ OpenType features
- ✅ Variable fonts
- ✅ Complex scripts
- ✅ Color fonts (COLR/CPAL)
- ✅ Emoji
- ✅ Native macOS rendering

### DirectWrite (Windows)
- ✅ OpenType features
- ✅ Variable fonts
- ✅ Complex scripts
- ✅ Color fonts (COLR/CPAL)
- ✅ ClearType rendering
- ✅ Native Windows rendering

### HarfBuzz+ICU
- ✅ OpenType features
- ✅ Variable fonts
- ✅ Complex scripts
- ✅ Color fonts (COLR/CPAL)
- ✅ Cross-platform consistency
- ✅ ICU text segmentation

### orge (Custom Rasterizer)
- ✅ Basic glyph outlines
- ✅ F26Dot6 fixed-point precision
- ⚠️ Limited OpenType feature support
- ⚠️ No color font support (yet)
- ✅ Lightweight, minimal dependencies

## Performance Benchmarks

Coming soon.

## Selecting a Backend

### Automatic Selection

TYPF automatically selects the best backend for your platform:

```bash
# Automatic backend selection
typf render --font=font.ttf --text="Auto" --output=out.png
```

### Manual Selection

You can override the backend selection:

```bash
# Force HarfBuzz backend
typf render --backend=harfbuzz --font=font.ttf --text="HB" --output=out.png

# Force CoreText (macOS only)
typf render --backend=coretext --font=font.ttf --text="CT" --output=out.png
```

### From Python

```python
import typf

result = typf.render_text(
    font_path="font.ttf",
    text="Backend",
    size=48,
    backend="harfbuzz",  # or "coretext", "directwrite", "orge"
    output_path="out.png"
)
```

## Backend Details

- [CoreText](coretext.md) - macOS native backend
- [DirectWrite](directwrite.md) - Windows native backend
- [HarfBuzz](harfbuzz.md) - Cross-platform backend
- [orge](orge.md) - Custom rasterizer

---

**Made by [FontLab](https://www.fontlab.com/)**
