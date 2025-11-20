# Chapter 19: Python API

## Overview

The TYPF Python API provides seamless bindings to the high-performance Rust text rendering engine, enabling Python developers to leverage TYPF's capabilities without leaving the Python ecosystem. Built with PyO3 and modern Python packaging standards, the API offers both simple convenience functions and deep control over the text rendering pipeline. This chapter covers the complete Python interface, from basic usage to advanced integration patterns.

## Architecture

### Python Binding Structure

```python
# Main Typf class
class Typf:
    def __init__(self, *, shaper="harfbuzz", renderer="skia", config=None)
    def render_text(self, text, font_path=None, font_size=16.0, export_format="png")
    def render_text_to_file(self, text, output_path, font_path=None, **kwargs)
    def shape_text(self, text, font_path=None, font_size=16.0, **kwargs)
    def get_font_info(self, font_path)

# Configuration classes
@dataclass
class TypfConfig:
    shaper: ShaperConfig
    renderer: RendererConfig
    cache: CacheConfig
    export: ExportConfig

@dataclass 
class ShaperConfig:
    backend: str
    enable_kerning: bool = True
    enable_ligatures: bool = True
    bidirectional_processing: bool = True
```

## Installation and Setup

### Modern Python Installation

```bash
# Using uv (recommended)
uv add typf

# Using pip (traditional)
pip install typf

# Install with specific features
pip install typf[skia,harfbuzz]  # Full feature set
pip install typf[minimal]      # Minimal dependencies
```

### Development Installation

```bash
# Clone repository
git clone https://github.com/fontlaborg/typf.git
cd typf

# Install with uv for development
uv venv
uv pip install -e ".[dev,docs,test]"
```

### Platform-Specific Notes

```python
import typf
import sys

def check_installation():
    """Verify TYPF installation and available backends."""
    print(f"TYPF version: {typf.__version__}")
    print(f"Python version: {sys.version}")
    print(f"Platform: {sys.platform}")
    
    # Check available backends
    available_shapers = typf.get_available_shapers()
    available_renderers = typf.get_available_renderers()
    
    print(f"Available shapers: {available_shapers}")
    print(f"Available renderers: {available_renderers}")

# System requirements check
def check_system_requirements():
    """Verify system meets TYPF requirements."""
    if sys.platform.startswith("darwin"):
        print("macOS detected - CoreText is available")
    elif sys.platform.startswith("win32"):
        print("Windows detected - DirectWrite is available")
    elif sys.platform.startswith("linux"):
        print("Linux detected - HarfBuzz is recommended")
    else:
        print(f"Unknown platform: {sys.platform}")
```

## Basic Usage

### Simple Text Rendering

```python
import typf

def basic_rendering():
    """Render text with default settings."""
    # Create Typf instance
    renderer = typf.Typf()
    
    # Render text to memory
    result = renderer.render_text(
        "Hello, TYPF!",
        font_path="/path/to/font.ttf",
        font_size=32.0,
        export_format="png"
    )
    
    print(f"Rendered {len(result.data)} bytes")
    print(f"Format: {result.format}")
    
    # Save to file
    with open("output.png", "wb") as f:
        f.write(result.data)
    
    print("Text saved to output.png")

# Alternative: render directly to file
def render_to_file():
    """Render text directly to file."""
    renderer = typf.Typf()
    
    renderer.render_text_to_file(
        "Direct file rendering",
        "direct_output.png",
        font_path="/path/to/font.otf",
        font_size=24.0,
        width=800,
        height=400,
        background_color="white",
        text_color="black"
    )
    
    print("Text rendered directly to file")
```

### Custom Configuration

```python
from dataclasses import dataclass

@dataclass
class CustomConfig:
    """Custom configuration for text rendering."""
    shaper_backend: str = "harfbuzz"
    renderer_backend: str = "skia"
    enable_ligatures: bool = True
    enable_kerning: bool = True
    antialiasing: bool = True
    subpixel_rendering: bool = True

def custom_configuration():
    """Render with custom configuration."""
    config = typf.TypfConfig(
        shaper=typf.ShaperConfig(
            backend="harfbuzz",
            enable_kerning=True,
            enable_ligatures=True,
            bidirectional_processing=True
        ),
        renderer=typf.RendererConfig(
            backend="skia",
            antialiasing=True,
            subpixel_rendering=True,
            hinting="slight"
        ),
        export=typf.ExportConfig(
            format="png",
            dpi=300,
            compression=9
        )
    )
    
    renderer = typf.Typf(config=config)
    
    result = renderer.render_text(
        "Custom configured text",
        font_path="/path/to/font.ttf",
        font_size=28.0
    )
    
    with open("custom_output.png", "wb") as f:
        f.write(result.data)
```

## Advanced Features

### Font Management

```python
class FontManager:
    """Manage fonts and font loading."""
    
    def __init__(self):
        self.font_cache = {}
        self._typf = typf.Typf()
    
    def load_font(self, path: str, alias: str = None) -> str:
        """Load font and return font identifier."""
        if alias is None:
            alias = path
        
        if alias not in self.font_cache:
            font_info = self._typf.get_font_info(path)
            self.font_cache[alias] = font_info
            print(f"Loaded font: {font_info.family} {font_info.weight}")
        
        return alias
    
    def get_available_fonts(self, family: str = None) -> list:
        """Get available fonts matching criteria."""
        fonts = []
        
        for alias, font_info in self.font_cache.items():
            if family is None or font_info.family == family:
                fonts.append({
                    'alias': alias,
                    'family': font_info.family,
                    'weight': font_info.weight,
                    'style': font_info.style,
                    'path': font_info.path
                })
        
        return fonts

# Font usage example
def font_management_example():
    font_manager = FontManager()
    
    # Load fonts
    font_manager.load_font("/path/to/regular.ttf", "text-regular")
    font_manager.load_font("/path/to/bold.ttf", "text-bold")
    font_manager.load_font("/path/to/italic.ttf", "text-italic")
    
    # Render with different fonts
    fonts = font_manager.get_available_fonts()
    
    for font in fonts:
        output_name = f"sample_{font['alias'].replace('/', '_')}.png"
        
        renderer.render_text_to_file(
            "Sample Text",
            output_name,
            font_path=font['path'],
            font_size=24.0
        )
        
        print(f"Created sample for {font['alias']}")
```

### Text Shaping

```python
def shaping_analysis():
    """Analyze text shaping results."""
    renderer = typf.Typf()
    
    # Shape text with detailed analysis
    shaping_result = renderer.shape_text(
        "Advanced shaping analysis",
        font_path="/path/to/font.otf",
        font_size=32.0,
        include_metrics=True,
        include_positions=True
    )
    
    print(f"Text: {shaping_result.text}")
    print(f"Glyph count: {len(shaping_result.glyphs)}")
    print(f"Font: {shaping_result.font.family} {shaping_result.font.weight}")
    
    # Analyze glyph sequence
    print("\nGlyph analysis:")
    for i, (glyph, position) in enumerate(zip(shaping_result.glyphs, shaping_result.positions)):
        print(f"  {i}: glyph {glyph.id} at ({position.x:.1f}, {position.y:.1f}) "
              f"advance {position.advance:.1f}")
    
    # Metrics
    if shaping_result.metrics:
        metrics = shaping_result.metrics
        print(f"\nMetrics:")
        print(f"  Width: {metrics.width:.1f}")
        print(f"  Height: {metrics.height:.1f}")
        print(f"  Ascent: {metrics.ascent:.1f}")
        print(f"  Descent: {metrics.descent:.1f}")

def script_detection():
    """Demonstrate script detection and handling."""
    text_samples = [
        "Hello World",           # Latin
        "مرحبا بالعالم",         # Arabic
        "שלום עולם",            # Hebrew
        "こんにちは世界",          # Japanese
        "你好世界",               # Chinese
    ]
    
    renderer = typf.Typf()
    
    for text in text_samples:
        try:
            shaping_result = renderer.shape_text(
                text,
                font_path="/path/to/font.ttf",  # Use font with good Unicode support
                font_size=24.0,
                script_detection=True,
                bidirectional_processing=True
            )
            
            print(f"Text: {text}")
            print(f"  Script: {shaping_result.script}")
            print(f"  Direction: {shaping_result.direction}")
            print(f"  Glyphs: {len(shaping_result.glyphs)}")
            print()
            
        except Exception as e:
            print(f"Failed to render '{text}': {e}")
```

### Batch Processing

```python
import asyncio
from concurrent.futures import ThreadPoolExecutor
from typing import List, Dict, Any

class BatchProcessor:
    """Process multiple text rendering tasks efficiently."""
    
    def __init__(self, max_workers: int = 4):
        self.max_workers = max_workers
        self._executor = ThreadPoolExecutor(max_workers=max_workers)
    
    async def process_batch(
        self, 
        tasks: List[Dict[str, Any]],
        progress_callback=None
    ) -> List[typf.RenderResult]:
        """Process a batch of rendering tasks."""
        loop = asyncio.get_event_loop()
        
        def render_task(task):
            """Render individual task."""
            renderer = typf.Typf()
            
            return renderer.render_text(
                task['text'],
                font_path=task.get('font_path'),
                font_size=task.get('font_size', 16.0),
                export_format=task.get('format', 'png'),
                **task.get('options', {})
            )
        
        # Submit all tasks
        futures = []
        for i, task in enumerate(tasks):
            future = loop.run_in_executor(
                self._executor, 
                render_task, 
                task
            )
            futures.append((i, future))
        
        # Collect results
        results = [None] * len(tasks)
        completed = 0
        
        for i, future in futures:
            try:
                result = await future
                results[i] = result
                completed += 1
                
                if progress_callback:
                    progress_callback(completed, len(tasks))
                    
            except Exception as e:
                results[i] = None
                print(f"Task {i} failed: {e}")
        
        return results
    
    def __del__(self):
        """Cleanup executor."""
        if hasattr(self, '_executor'):
            self._executor.shutdown(wait=False)

# Batch processing example
async def batch_processing_example():
    """Process multiple texts in parallel."""
    tasks = [
        {
            'text': 'Sample 1',
            'font_path': '/path/to/font.ttf',
            'font_size': 24.0,
            'format': 'png',
            'options': {'width': 400, 'height': 200}
        },
        {
            'text': 'Sample 2',
            'font_path': '/path/to/font.otf',
            'font_size': 32.0,
            'format': 'svg',
            'options': {'precision': 2}
        },
        {
            'text': 'Sample 3',
            'font_path': '/path/to/font-bold.ttf',
            'font_size': 28.0,
            'format': 'png',
            'options': {'background_color': 'lightgray'}
        },
    ]
    
    processor = BatchProcessor(max_workers=3)
    
    def progress_callback(completed, total):
        print(f"Progress: {completed}/{total} tasks completed")
    
    results = await processor.process_batch(tasks, progress_callback)
    
    # Save results
    for i, result in enumerate(results):
        if result and result.data:
            filename = f"batch_output_{i}.{result.format}"
            with open(filename, 'wb') as f:
                f.write(result.data)
            print(f"Saved: {filename}")
```

## Performance Optimization

### Caching and Memory Management

```python
class CachedRenderer:
    """Renderer with intelligent caching."""
    
    def __init__(self, max_cache_size: int = 100):
        self._typf = typf.Typf()
        self._cache = {}
        self._max_cache_size = max_cache_size
        self._cache_hits = 0
        self._cache_misses = 0
    
    def render_with_cache(
        self, 
        text: str, 
        font_path: str, 
        font_size: float,
        **kwargs
    ) -> typf.RenderResult:
        """Render with caching."""
        cache_key = self._generate_cache_key(text, font_path, font_size, **kwargs)
        
        if cache_key in self._cache:
            self._cache_hits += 1
            return self._cache[cache_key]
        
        self._cache_misses += 1
        
        result = self._typf.render_text(
            text, font_path, font_size, **kwargs
        )
        
        # Add to cache
        if len(self._cache) >= self._max_cache_size:
            # Remove oldest entry (simple LRU)
            oldest_key = next(iter(self._cache))
            del self._cache[oldest_key]
        
        self._cache[cache_key] = result
        return result
    
    def _generate_cache_key(self, text: str, font_path: str, font_size: float, **kwargs) -> str:
        """Generate cache key from parameters."""
        import hashlib
        key_data = f"{text}:{font_path}:{font_size}:{sorted(kwargs.items())}"
        return hashlib.md5(key_data.encode()).hexdigest()
    
    def get_cache_stats(self) -> Dict[str, Any]:
        """Get cache performance statistics."""
        total_requests = self._cache_hits + self._cache_misses
        hit_rate = (self._cache_hits / total_requests) if total_requests > 0 else 0
        
        return {
            'cache_size': len(self._cache),
            'max_cache_size': self._max_cache_size,
            'hits': self._cache_hits,
            'misses': self._cache_misses,
            'hit_rate': f"{hit_rate:.2%}"
        }

# Performance benchmarking
def performance_benchmark():
    """Benchmark TYPF performance."""
    import time
    import os
    
    renderer = typf.Typf()
    test_text = "Performance benchmark text"
    font_path = "/path/to/font.ttf"
    iterations = 100
    
    # Warm up
    renderer.render_text(test_text, font_path, 16.0)
    
    # Benchmark rendering
    start_time = time.time()
    
    for i in range(iterations):
        result = renderer.render_text(
            f"{test_text} {i}",
            font_path,
            16.0,
            export_format="png"
        )
    
    end_time = time.time()
    total_time = end_time - start_time
    
    avg_time = total_time / iterations
    renders_per_second = 1.0 / avg_time
    
    print(f"Benchmark Results:")
    print(f"  Iterations: {iterations}")
    print(f"  Total time: {total_time:.3f}s")
    print(f"  Average time: {avg_time * 1000:.2f}ms")
    print(f"  Renders per second: {renders_per_second:.1f}")
    
    # Memory usage
    import psutil
    process = psutil.Process(os.getpid())
    memory_info = process.memory_info()
    
    print(f"  Memory usage: {memory_info.rss / 1024 / 1024:.1f} MB")
```

## Error Handling

### Python Error Management

```python
class TypfErrorHandler:
    """Handle TYPF-specific errors in Python."""
    
    @staticmethod
    def handle_rendering_error(error: Exception):
        """Handle rendering errors with suggestions."""
        if isinstance(error, typf.FontNotFoundError):
            print(f"Font not found: {error.path}")
            print("Suggestions:")
            print("  - Check if font file exists")
            print("  - Verify font file permissions")
            print("  - Try a different font format (.ttf, .otf)")
            
        elif isinstance(error, typf.ShapingError):
            print(f"Text shaping failed: {error.message}")
            print("Suggestions:")
            print("  - Check if font supports the required script")
            print("  - Try enabling bidirectional processing")
            print("  - Verify text contains valid Unicode characters")
            
        elif isinstance(error, typf.RenderingError):
            print(f"Rendering failed: {error.message}")
            print("Suggestions:")
            print("  - Check rendering dimensions are valid")
            print("  - Verify font size is reasonable")
            print("  - Try different renderer backend")
            
        else:
            print(f"Unexpected error: {error}")

def robust_rendering():
    """Example of robust error handling."""
    try:
        renderer = typf.Typf()
        
        result = renderer.render_text(
            "Robust rendering example",
            font_path="/path/to/nonexistent.ttf",
            font_size=24.0
        )
        
    except Exception as e:
        TypfErrorHandler.handle_rendering_error(e)
        
        # Try with fallback font
        try:
            fallback_font = "/System/Library/Fonts/Arial.ttf"  # System font
            result = renderer.render_text(
                "Robust rendering example",
                font_path=fallback_font,
                font_size=24.0
            )
            print("Successfully rendered with fallback font")
            
        except Exception as fallback_error:
            print(f"Fallback also failed: {fallback_error}")
```

## Integration Examples

### Web Applications (FastAPI)

```python
from fastapi import FastAPI, HTTPException
from fastapi.responses import Response
from pydantic import BaseModel
import typing

app = FastAPI(title="TYPF Text Rendering Service")

class RenderRequest(BaseModel):
    text: str
    font_size: float = 16.0
    width: typing.Optional[int] = None
    height: typing.Optional[int] = None
    background_color: str = "white"
    text_color: str = "black"
    format: str = "png"

class TextRenderer:
    """TYPF renderer for web service."""
    
    def __init__(self):
        self._renderer = typf.Typf()
        self._font_cache = {}
    
    async def render(self, request: RenderRequest) -> bytes:
        """Render text for web request."""
        try:
            # Use default system font or load specific font
            font_path = "/System/Library/Fonts/Helvetica.ttc"
            
            result = self._renderer.render_text(
                text=request.text,
                font_path=font_path,
                font_size=request.font_size,
                width=request.width,
                height=request.height,
                background_color=request.background_color,
                text_color=request.text_color,
                export_format=request.format
            )
            
            return result.data
            
        except Exception as e:
            raise HTTPException(status_code=500, detail=str(e))

# Global renderer instance
text_renderer = TextRenderer()

@app.post("/render")
async def render_text(request: RenderRequest):
    """Render text endpoint."""
    # Validate format
    if request.format not in ["png", "jpeg", "svg", "pdf"]:
        raise HTTPException(status_code=400, detail="Unsupported format")
    
    # Render text
    data = await text_renderer.render(request)
    
    # Return appropriate response
    media_type = f"image/{request.format}"
    return Response(content=data, media_type=media_type)

@app.get("/backends")
async def get_backends():
    """Get available backends."""
    return {
        "shapers": typf.get_available_shapers(),
        "renderers": typf.get_available_renderers(),
        "formats": ["png", "jpeg", "svg", "pdf", "json"]
    }

@app.get("/health")
async def health_check():
    """Health check endpoint."""
    return {"status": "healthy", "typf_version": typf.__version__}
```

### Data Science Integration

```python
import pandas as pd
import numpy as np
from pathlib import Path

class DataVisualizationTextRenderer:
    """Render text for data visualization purposes."""
    
    def __init__(self):
        self._renderer = typf.Typf()
        self._output_dir = Path("text_renders")
        self._output_dir.mkdir(exist_ok=True)
    
    def render_dataframe_sample(
        self, 
        df: pd.DataFrame, 
        sample_size: int = 10,
        font_size: float = 12.0
    ) -> List[str]:
        """Render sample text from dataframe."""
        rendered_files = []
        
        # Sample text from different columns
        text_columns = df.select_dtypes(include=['object']).columns
        
        for col in text_columns[:5]:  # Limit to 5 columns
            sample_texts = df[col].dropna().head(sample_size).tolist()
            
            for i, text in enumerate(sample_texts):
                filename = f"{col}_{i:03d}.png"
                output_path = self._output_dir / filename
                
                # Render text
                self._renderer.render_text_to_file(
                    text=str(text),
                    output_path=str(output_path),
                    font_path="/System/Library/Fonts/Arial.ttf",
                    font_size=font_size,
                    width=300,
                    height=50,
                    background_color="white",
                    text_color="black"
                )
                
                rendered_files.append(str(output_path))
        
        return rendered_files
    
    def create_text_gallery(
        self, 
        texts: List[str], 
        output_path: str = "text_gallery.png"
    ):
        """Create a gallery of rendered texts."""
        from io import BytesIO
        from PIL import Image
        
        # Render each text
        rendered_images = []
        
        for text in texts:
            result = self._renderer.render_text(
                text=text,
                font_path="/System/Library/Fonts/Helvetica.ttc",
                font_size=16.0,
                export_format="png"
            )
            
            img = Image.open(BytesIO(result.data))
            rendered_images.append(img)
        
        # Create gallery
        if rendered_images:
            # Calculate gallery dimensions
            cols = 3
            rows = (len(rendered_images) + cols - 1) // cols
            
            max_width = max(img.width for img in rendered_images)
            max_height = max(img.height for img in rendered_images)
            
            # Create gallery image
            gallery = Image.new(
                'RGB', 
                (max_width * cols, max_height * rows), 
                'white'
            )
            
            # Paste rendered images
            for i, img in enumerate(rendered_images):
                row = i // cols
                col = i % cols
                
                x = col * max_width
                y = row * max_height
                
                gallery.paste(img, (x, y))
            
            gallery.save(output_path)
            print(f"Gallery saved to: {output_path}")

# Data science example
def data_science_integration():
    """Example of TYPF integration with data science workflows."""
    # Create sample data
    data = {
        'names': ['Alice Johnson', 'Bob Smith', 'Charlie Brown', 'Diana Prince', 'Edward Norton'],
        'categories': ['Engineering', 'Marketing', 'Finance', 'Legal', 'Operations'],
        'descriptions': ['Senior developer', 'Marketing manager', 'Financial analyst', 'Corporate lawyer', 'Operations director']
    }
    
    df = pd.DataFrame(data)
    
    # Render text samples
    renderer = DataVisualizationTextRenderer()
    rendered_files = renderer.render_dataframe_sample(df)
    
    print(f"Rendered {len(rendered_files)} text samples")
    
    # Create text gallery
    sample_texts = df['names'].tolist()[:6]
    renderer.create_text_gallery(sample_texts)
```

## Testing

### Python Testing Patterns

```python
import pytest
import tempfile
import os
from pathlib import Path

class TestTypfPython:
    """Test suite for TYPF Python bindings."""
    
    @pytest.fixture
    def temp_font(self):
        """Provide a temporary font file for testing."""
        # Copy test font to temporary location
        test_font = "/path/to/test_font.ttf"  # Update path
        with tempfile.NamedTemporaryFile(suffix='.ttf', delete=False) as tmp:
            with open(test_font, 'rb') as src:
                tmp.write(src.read())
            yield tmp.name
        os.unlink(tmp.name)
    
    @pytest.fixture
    def renderer(self):
        """Provide a TYPF renderer instance."""
        return typf.Typf()
    
    def test_basic_rendering(self, renderer, temp_font):
        """Test basic text rendering."""
        result = renderer.render_text(
            "Test text",
            font_path=temp_font,
            font_size=16.0,
            export_format="png"
        )
        
        assert isinstance(result, typf.RenderResult)
        assert result.format == "png"
        assert len(result.data) > 0
        
        # Verify PNG signature
        assert result.data[:8] == b'\x89PNG\r\n\x1a\n'
    
    def test_multiple_formats(self, renderer, temp_font):
        """Test rendering to multiple formats."""
        formats = ["png", "svg", "pdf", "json"]
        
        for format in formats:
            result = renderer.render_text(
                "Format test",
                font_path=temp_font,
                font_size=12.0,
                export_format=format
            )
            
            assert result.format == format
            assert len(result.data) > 0
    
    def test_text_shaping(self, renderer, temp_font):
        """Test text shaping functionality."""
        shaping_result = renderer.shape_text(
            "Shaping test",
            font_path=temp_font,
            font_size=24.0
        )
        
        assert isinstance(shaping_result, typf.ShapingResult)
        assert shaping_result.text == "Shaping test"
        assert len(shaping_result.glyphs) > 0
        assert len(shaping_result.positions) > 0
    
    def test_font_info(self, temp_font):
        """Test font information retrieval."""
        renderer = typf.Typf()
        font_info = renderer.get_font_info(temp_font)
        
        assert isinstance(font_info, typf.FontInfo)
        assert font_info.family is not None
        assert font_info.weight is not None
        assert font_info.style is not None
    
    def test_error_handling(self, renderer):
        """Test error handling for invalid inputs."""
        with pytest.raises(typf.TypfError):
            renderer.render_text(
                "Test",
                font_path="/nonexistent/font.ttf",
                font_size=16.0
            )
    
    def test_configuration(self):
        """Test custom configuration."""
        config = typf.TypfConfig(
            shaper=typf.ShaperConfig(
                backend="harfbuzz",
                enable_kerning=True,
                enable_ligatures=True
            ),
            renderer=typf.RendererConfig(
                backend="skia",
                antialiasing=True
            )
        )
        
        renderer = typf.Typf(config=config)
        
        # Should work without errors
        assert renderer is not None

# Performance testing
def test_performance():
    """Test performance characteristics."""
    renderer = typf.Typf()
    font_path = "/path/to/test_font.ttf"
    
    import time
    
    # Test rendering speed
    start_time = time.time()
    iterations = 50
    
    for i in range(iterations):
        result = renderer.render_text(
            f"Performance test {i}",
            font_path=font_path,
            font_size=16.0
        )
        assert len(result.data) > 0
    
    elapsed_time = time.time() - start_time
    avg_time = elapsed_time / iterations
    
    # Should be reasonably fast (adjust threshold as needed)
    assert avg_time < 0.1  # < 100ms per render
    print(f"Average render time: {avg_time * 1000:.2f}ms")

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
```

## Best Practices

### Memory Performance

```python
def memory_optimization_tips():
    """Tips for optimizing memory usage in Python."""
    
    # 1. Reuse Typf instances
    # BAD: Creating new instance for each render
    for i in range(100):
        renderer = typf.Typf()  # Expensive!
        renderer.render_text(f"Text {i}", "font.ttf")
    
    # GOOD: Reuse single instance
    renderer = typf.Typf()
    for i in range(100):
        renderer.render_text(f"Text {i}", "font.ttf")
    
    # 2. Use streaming for large outputs
    def render_large_document(text_chunks):
        """Render large document efficiently."""
        renderer = typf.Typf()
        
        for i, chunk in enumerate(text_chunks):
            result = renderer.render_text(chunk, "font.ttf")
            yield result.data  # Generator yields, not holds all in memory
    
    # 3. Clean up resources
    def cleanup_resources():
        """Explicit cleanup of resources."""
        renderer = typf.Typf()
        
        # Use renderer
        result = renderer.render_text("Test", "font.ttf")
        
        # Explicit cleanup when done
        del renderer  # Helps garbage collection

### Performance Patterns

```python
import concurrent.futures
import threading

class ThreadPoolRenderer:
    """Thread-pooled rendering for high-throughput applications."""
    
    def __init__(self, num_threads: int = 4):
        self.num_threads = num_threads
        self._pool = None
        self._local_renderer = threading.local()
    
    def __enter__(self):
        self._pool = concurrent.futures.ThreadPoolExecutor(max_workers=self.num_threads)
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        if self._pool:
            self._pool.shutdown(wait=True)
    
    def _get_renderer(self):
        """Get thread-local renderer instance."""
        if not hasattr(self._local_renderer, 'renderer'):
            self._local_renderer.renderer = typf.Typf()
        return self._local_renderer.renderer
    
    def render_batch(self, tasks):
        """Render batch of texts using thread pool."""
        futures = []
        
        for task in tasks:
            future = self._pool.submit(self._render_single, task)
            futures.append(future)
        
        results = []
        for future in concurrent.futures.as_completed(futures):
            try:
                result = future.result()
                results.append(result)
            except Exception as e:
                print(f"Render failed: {e}")
                results.append(None)
        
        return results
    
    def _render_single(self, task):
        """Render single task."""
        renderer = self._get_renderer()
        return renderer.render_text(
            task['text'],
            task['font_path'],
            task['font_size'],
            **task.get('options', {})
        )

# Usage example
def thread_pool_example():
    """Example of thread-pooled rendering."""
    tasks = [
        {'text': f'Task {i}', 'font_path': 'font.ttf', 'font_size': 16.0}
        for i in range(100)
    ]
    
    with ThreadPoolRenderer(num_threads=4) as pool:
        results = pool.render_batch(tasks)
    
    print(f"Rendered {len(results)} tasks")
```

The TYPF Python API provides a powerful, feature-rich interface that brings high-performance text rendering to the Python ecosystem while maintaining the speed and efficiency of the underlying Rust implementation.