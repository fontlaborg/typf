# Python API

TYPF's Python bindings bring high-performance text rendering to Python with the same speed as the Rust implementation.

## Quick Start

```python
import typf

# Simple text rendering
renderer = typf.Typf()
result = renderer.render_text("Hello World", "font.ttf")

# Save to different formats
result.save("output.png")
result.save("output.svg") 
result.save("output.pdf")
```

## Installation

```bash
# Install from PyPI
pip install typf

# Or build from source with specific features
pip install typf --features="shaping-harfbuzz,render-skia"
```

## Core Classes

### Typf

Main class for text rendering operations.

```python
class Typf:
    def __init__(self, 
                 shaper: str = "harfbuzz",
                 renderer: str = "opixa",
                 font_db: Optional[FontDatabase] = None):
        """Initialize renderer with backends."""
        
    def render_text(self, 
                    text: str, 
                    font_path: str,
                    font_size: float = 16.0,
                    width: int = 800,
                    height: int = 600,
                    **kwargs) -> RenderResult:
        """Render text to bitmap."""
        
    def render_with_options(self,
                           text: str,
                           font_path: str,
                           options: RenderOptions) -> RenderResult:
        """Render with custom options."""
        
    def list_backends(self) -> Dict[str, List[str]]:
        """Get available backends."""
```

### RenderResult

Contains the rendered output and metadata.

```python
class RenderResult:
    @property
    def width(self) -> int:
        """Image width in pixels."""
        
    @property  
    def height(self) -> int:
        """Image height in pixels."""
        
    @property
    def glyph_count(self) -> int:
        """Number of glyphs rendered."""
        
    def save(self, filename: str, format: Optional[str] = None):
        """Save to file. Format inferred from extension."""
        
    def to_bytes(self, format: str) -> bytes:
        """Get encoded bytes."""
        
    def get_pixels(self) -> numpy.ndarray:
        """Get pixel data as NumPy array."""
        
    def get_glyphs(self) -> List[GlyphInfo]:
        """Get glyph positioning info."""
```

### RenderOptions

Configure rendering parameters.

```python
@dataclass
class RenderOptions:
    font_size: float = 16.0
    dpi: float = 72.0
    width: int = 800
    height: int = 600
    color: Tuple[int, int, int, int] = (0, 0, 0, 255)
    background: Tuple[int, int, int, int] = (255, 255, 255, 0)
    hinting: str = "normal"
    antialiasing: bool = True
    
    # Advanced options
    subpixel_positioning: bool = True
    embolden: float = 0.0
    slant: float = 0.0
```

## Font Management

### FontDatabase

Load and manage fonts efficiently.

```python
class FontDatabase:
    def __init__(self, include_system: bool = True):
        """Create font database."""
        
    def load_font(self, path: str) -> Font:
        """Load font from file path."""
        
    def load_font_bytes(self, name: str, data: bytes) -> Font:
        """Load font from bytes."""
        
    def get_font(self, name: str) -> Optional[Font]:
        """Get font by name."""
        
    def list_fonts(self) -> List[str]:
        """List available font names."""
        
    def search_family(self, family: str) -> List[Font]:
        """Find fonts in same family."""
```

### Font

Represents a loaded font.

```python
class Font:
    @property
    def name(self) -> str:
        """Font full name."""
        
    @property
    def family(self) -> str:
        """Font family name."""
        
    @property
    def style(self) -> str:
        """Font style (normal, italic, bold, etc.)."""
        
    @property
    def metrics(self) -> FontMetrics:
        """Font measurements."""
        
    def supports_glyph(self, codepoint: int) -> bool:
        """Check if font contains glyph."""
        
    def get_glyph_advance(self, glyph_id: int) -> float:
        """Get glyph advance width."""
```

## Backends Selection

### Available Shapers

```python
# List available text shapers
renderer = typf.Typf()
backends = renderer.list_backends()
print(backends['shapers'])
# ['none', 'harfbuzz', 'icu-harfbuzz', 'coretext', 'directwrite']

# Use specific shaper
hb_renderer = typf.Typf(shaper="harfbuzz")
icu_renderer = typf.Typf(shaper="icu-harfbuzz")
```

### Available Renderers

```python
print(backends['renderers'])  
# ['opixa', 'skia', 'coregraphics', 'directwrite', 'zeno']

# Use specific renderer
skia_renderer = typf.Typf(renderer="skia")
zeno_renderer = typf.Typf(renderer="zeno")
```

### Backend Recommendations

| Use Case | Shaper | Renderer |
|----------|--------|----------|
| Web graphics | harfbuzz | skia |
| Print production | icu-harfbuzz | zeno |
| Simple text | none | opixa |
| macOS apps | coretext | coregraphics |
| Windows apps | directwrite | directwrite |
| Cross-platform | harfbuzz | opixa |

## Color and Formatting

### Colors

```python
# RGBA tuples
black = (0, 0, 0, 255)
red = (255, 0, 0, 255)
transparent = (0, 0, 0, 0)

# Helper functions
def rgb(r, g, b):
    return (r, g, b, 255)

def rgba(r, g, b, a):
    return (r, g, b, a)

# Use in rendering
options = typf.RenderOptions(
    color=rgb(50, 100, 200),
    background=rgba(255, 255, 255, 128)
)
```

### Text Effects

```python
# Embolden text
options = typf.RenderOptions(embolden=0.1)  # 10% bolder

# Slant text  
options = typf.RenderOptions(slant=0.2)     # 20% slant

# disable hinting for small text
options = typf.RenderOptions(
    font_size=8.0,
    hinting="none"
)
```

## Export Formats

### Supported Formats

```python
result.save("image.png")    # Raster image
result.save("vector.svg")   # Vector graphics  
result.save("document.pdf") # Print document
result.save("data.json")    # Debug data
```

### Format-Specific Options

```python
# PNG with compression
result.save("output.png", quality=9)  # PNG compression level

# SVG with embedded fonts
result.save("output.svg", embed_fonts=True)

# PDF with metadata
result.save("output.pdf", 
           title="My Document",
           author="My Name",
           subject="Text Rendering")
```

## NumPy Integration

### Get Pixel Data

```python
import numpy as np
import typf

# Render text
renderer = typf.Typf()
result = renderer.render_text("Hello NumPy", "font.ttf")

# Get as NumPy array
pixels = result.get_pixels()
print(pixels.shape)  # (height, width, 4) for RGBA

# Process pixels
gray = np.mean(pixels[:,:,:3], axis=2)  # Convert to grayscale
blurred = gaussian_filter(gray, sigma=1.0)  # Apply filter
```

### Create from NumPy

```python
# Create from existing pixel data
def create_from_numpy(pixels: np.ndarray) -> typf.RenderResult:
    """Create Result from NumPy array."""
    data = pixels.tobytes()
    return typf.RenderResult.from_bytes(
        data=data,
        width=pixels.shape[1],
        height=pixels.shape[0],
        format="rgba"
    )
```

## Batch Processing

### Render Multiple Texts

```python
def render_batch(texts: List[str], font_path: str) -> List[typf.RenderResult]:
    """Render multiple texts efficiently."""
    renderer = typf.Typf()
    results = []
    
    for text in texts:
        result = renderer.render_text(text, font_path)
        results.append(result)
        
    return results

# Usage
texts = ["Hello", "World", "TYPF", "Python"]
results = render_batch(texts, "Roboto-Regular.ttf")
```

### Parallel Processing

```python
from concurrent.futures import ThreadPoolExecutor
import typf

def render_worker(args):
    text, font_path = args
    renderer = typf.Typf()
    return renderer.render_text(text, font_path)

def parallel_render(texts: List[str], font_path: str, workers: int = 4):
    """Render texts in parallel."""
    with ThreadPoolExecutor(max_workers=workers) as executor:
        args = [(text, font_path) for text in texts]
        results = list(executor.map(render_worker, args))
    return results
```

## Error Handling

### Exception Types

```python
try:
    result = renderer.render_text("Hello", "nonexistent.ttf")
except typf.TypfError as e:
    print(f"TypF error: {e}")
    
# Specific error types
try:
    result = renderer.render_text("Hello", "font.ttf")
except typf.FontError as e:
    print(f"Font loading failed: {e}")
except typf.RenderError as e:
    print(f"Rendering failed: {e}")
except typf.BackendError as e:
    print(f"Backend not available: {e}")
```

### Best Practices

```python
def safe_render(text: str, font_path: str) -> Optional[typf.RenderResult]:
    """Render with error handling."""
    try:
        renderer = typf.Typf()
        return renderer.render_text(text, font_path)
    except typf.FontError:
        print(f"Could not load font: {font_path}")
        return None
    except typf.RenderError as e:
        print(f"Rendering failed: {e}")
        return None
    except Exception as e:
        print(f"Unexpected error: {e}")
        return None
```

## Performance Tips

### Caching

```python
# Create renderer once, reuse many times
class TextRenderer:
    def __init__(self):
        self.renderer = typf.Typf(shaper="harfbuzz", renderer="skia")
        
    def render(self, text: str, font_path: str) -> typf.RenderResult:
        return self.renderer.render_text(text, font_path)

# Usage
renderer = TextRenderer()  # Initialize once
for text in texts:
    result = renderer.render(text, "font.ttf")  # Reuse
```

### Memory Management

```python
# Process large texts in chunks
def render_large_text(text: str, font_path: str, chunk_size: int = 1000):
    """Render text in manageable chunks."""
    renderer = typf.Typf()
    results = []
    
    for i in range(0, len(text), chunk_size):
        chunk = text[i:i + chunk_size]
        result = renderer.render_text(chunk, font_path)
        results.append(result)
        
    return results
```

### Async Support

```python
import asyncio
import typf

async def render_async(texts: List[str], font_path: str):
    """Render texts asynchronously."""
    loop = asyncio.get_event_loop()
    
    def render_text(text):
        renderer = typf.Typf()
        return renderer.render_text(text, font_path)
    
    tasks = []
    for text in texts:
        task = loop.run_in_executor(None, render_text, text)
        tasks.append(task)
        
    results = await asyncio.gather(*tasks)
    return results
```

## Web Framework Integration

### Flask Example

```python
from flask import Flask, send_file
import typf
import io

app = Flask(__name__)
renderer = typf.Typf()

@app.route('/render/<text>')
def render_text(text):
    """Render text as PNG."""
    result = renderer.render_text(text, "Roboto-Regular.ttf")
    
    # Convert to BytesIO for Flask
    img_bytes = result.to_bytes("png")
    return send_file(io.BytesIO(img_bytes), 
                    mimetype='image/png')
```

### Django Example

```python
from django.http import HttpResponse
import typf

def render_text_view(request, text):
    """Render text and return as response."""
    renderer = typf.Typf()
    result = renderer.render_text(text, "font.ttf")
    
    png_bytes = result.to_bytes("png")
    return HttpResponse(png_bytes, content_type='image/png')
```

## Testing

### Unit Tests

```python
import unittest
import typf

class TestTypf(unittest.TestCase):
    def setUp(self):
        self.renderer = typf.Typf()
        
    def test_basic_render(self):
        """Test basic text rendering."""
        result = self.renderer.render_text("Test", "font.ttf")
        self.assertIsNotNone(result)
        self.assertEqual(result.width, 800)
        self.assertEqual(result.height, 600)
        
    def test_invalid_font(self):
        """Test error handling for invalid font."""
        with self.assertRaises(typf.FontError):
            self.renderer.render_text("Test", "nonexistent.ttf")
```

### Performance Tests

```python
import time
import typf

def benchmark_render():
    """Benchmark rendering performance."""
    renderer = typf.Typf()
    text = "Performance test text"
    iterations = 100
    
    start_time = time.time()
    for _ in range(iterations):
        result = renderer.render_text(text, "font.ttf")
    end_time = time.time()
    
    avg_time = (end_time - start_time) / iterations
    print(f"Average render time: {avg_time:.3f}s")
```

---

The Python API provides the same performance as Rust while integrating seamlessly with Python's ecosystem. Use NumPy for image processing, async for concurrent work, and your favorite web framework for serving rendered text.
