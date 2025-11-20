# Chapter 20: CLI Interface

## Overview

The TYPF Command Line Interface (CLI) provides fast, scriptable access to the text rendering engine's capabilities from the terminal. Built with Clap v4 and following modern CLI design principles, the interface offers both simple one-shot rendering and advanced batch processing. This chapter covers the complete CLI functionality, from basic usage to complex automation workflows, with practical examples for different use cases.

## Architecture

### CLI Structure

```bash
typf <COMMAND> [OPTIONS] <ARGS>

Commands:
  render     Render text to image file
  shape      Shape text and output positioning data
  font       Display font information
  info       Show system and backend information
  batch      Process multiple texts from file or stdin
  bench      Benchmark rendering performance
  repl       Interactive text rendering shell
  export     Export text in various formats
```

### Core Components

```rust
// Main CLI structure
#[derive(Parser)]
#[command(name = "typf")]
#[command(about = "TYPF Text Rendering CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Global verbosity level
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    Render(RenderArgs),
    Shape(ShapeArgs),
    Font(FontArgs),
    Info(InfoArgs),
    Batch(BatchArgs),
    Bench(BenchArgs),
    Repl(ReplArgs),
    Export(ExportArgs),
}
```

## Installation

### Binary Installation

```bash
# Install from cargo crates.io
cargo install typf-cli

# Install from GitHub (latest)
cargo install --git https://github.com/fontlaborg/typf --branch main typf-cli

# Install with specific features
cargo install typf-cli --features "shaping-hb render-skia"
```

### Build from Source

```bash
# Clone repository
git clone https://github.com/fontlaborg/typf.git
cd typf

# Install with minimal features
cargo install --path crates/typf-cli --features "minimal"

# Install with full features
cargo install --path crates/typf-cli --features "full"

# Development build
cargo build --release --features "full"
```

### Shell Completions

```bash
# Generate completions for your shell
typf --completion bash > typf-completion.bash
typf --completion zsh > typf-completion.zsh
typf --completion fish > typf-completion.fish

# Install completions (bash example)
source typf-completion.bash
echo 'source typf-completion.bash' >> ~/.bashrc

# Fish completion installation
typf-completion.fish > ~/.config/fish/completions/typf.fish
```

## Basic Usage

### Simple Text Rendering

```bash
# Basic rendering
typf render "Hello, World!" -o output.png

# Specify font and size
typf render "Text with font" \
  --font /path/to/font.ttf \
  --font-size 24 \
  --output styled.png

# Multiple formats
typf render "Vector output" \
  --font /path/to/font.otf \
  --font-size 32 \
  --output vector.svg \
  --format svg

# Custom dimensions
typf render "Custom dimensions" \
  --width 800 \
  --height 200 \
  --background white \
  --color black \
  --output custom.png
```

### Font Information

```bash
# Get font information
typf font /path/to/font.ttf

# Detailed font analysis
typf font /path/to/font.otf --verbose

# List supported characters
typf font /path/to/font.ttf --list-characters

# Check font capabilities
typf font /path/to/font.ttf --check-features
```

### System Information

```bash
# Show available backends
typf info --backends

# Show system capabilities
typf info --system

# Show version and build info
typf info --version
typf info --build

# Show configuration
typf info --config
```

## Command Reference

### Render Command

```bash
typf render [OPTIONS] <TEXT>

Arguments:
  <TEXT>                    Text to render

Options:
  -o, --output <FILE>       Output file path [required]
  -f, --font <FILE>         Font file path
  -s, --font-size <SIZE>    Font size in points [default: 16.0]
      --format <FORMAT>     Output format: png, svg, pdf, pnm, json [default: png]
      --width <PIXELS>      Image width in pixels
      --height <PIXELS>     Image height in pixels
      --color <COLOR>       Text color (hex or name) [default: black]
      --background <COLOR>  Background color (hex or name) [default: transparent]
      --shaper <BACKEND>    Shaping backend: none, harfbuzz, coretext, directwrite, icu-hb
      --renderer <BACKEND>  Rendering backend: skia, orge, coregraphics, direct2d, zeno
      --dpi <DPI>           Output resolution [default: 72]
      --quality <QUALITY>   Compression quality (0-100) [default: 90]
  -v, --verbose             Enable verbose output
  -h, --help                Print help
```

#### Examples

```bash
# Basic PNG rendering
typf render "Hello, World!" -o hello.png

# High-quality PDF with custom font
typf render "Professional text" \
  --font /usr/share/fonts/noto/NotoSerif-Regular.ttf \
  --font-size 18 \
  --format pdf \
  --dpi 300 \
  --quality 95 \
  --output professional.pdf

# SVG with specific dimensions
typf render "SVG Graphics" \
  --font-size 24 \
  --width 600 \
  --height 150 \
  --background "#f0f0f0" \
  --color "#333333" \
  --format svg \
  --output graphics.svg

# Platform-specific backends
typf render "Platform rendering" \
  --shaper coretext \
  --renderer coregraphics \
  --output macos_native.png

# Unicode text with bidirectional support
typf render "مرحبا بالعالم" \
  --font /path/to/arabic-font.ttf \
  --font-size 20 \
  --shaper harfbuzz \
  --output arabic.png
```

### Shape Command

```bash
typf shape [OPTIONS] <TEXT>

Arguments:
  <TEXT>                    Text to shape

Options:
  -f, --font <FILE>         Font file path
  -s, --font-size <SIZE>    Font size in points [default: 16.0]
  -o, --output <FILE>       Output JSON file
      --format <FORMAT>     Output format: json, yaml, toml [default: json]
      --include-metrics     Include detailed metrics
      --include-positions   Include glyph positions
      --script-analysis     Enable script detection
  -v, --verbose             Enable verbose output
  -h, --help                Print help
```

#### Examples

```bash
# Basic shaping
typf shape "Shaping analysis" \
  --font /path/to/font.ttf \
  --font-size 24

# Detailed shaping with metrics
typf shape "Detailed analysis" \
  --font /path/to/font.otf \
  --include-metrics \
  --include-positions \
  --script-analysis \
  --output detailed.json

# Format options
typf shape "YAML format" \
  --font /path/to/font.ttf \
  --format yaml \
  --output shaping.yaml
```

### Batch Command

```bash
typf batch [OPTIONS] <INPUT>

Arguments:
  <INPUT>                   Input file or "-" for stdin

Options:
  -o, --output-dir <DIR>    Output directory [default: .]
  -f, --font <FILE>         Font file path
  -s, --font-size <SIZE>    Font size in points [default: 16.0]
      --format <FORMAT>     Output format: png, svg, pdf [default: png]
      --template <TEMPLATE> Output filename template
      --parallel <JOBS>     Number of parallel jobs [default: 4]
      --config <FILE>       Batch configuration file
  -v, --verbose             Enable verbose output
  -h, --help                Print help
```

#### Batch Input Formats

**JSON Lines (JSONL)**
```json
{"text": "Sample 1", "font_size": 16, "output": "sample1.png"}
{"text": "Sample 2", "font_size": 24, "output": "sample2.svg"}
{"text": "Sample 3", "font_size": 32, "color": "red"}
```

**CSV Format**
```csv
text,font_size,output,color
"Sample 1",16,sample1.png,black
"Sample 2",24,sample2.svg,blue
"Sample 3",32,,red
```

**Plain Text (one line per output)**
```
First line of text
Second line of text
Third line of text
```

#### Examples

```bash
# Process JSONL file
typf batch inputs.jsonl \
  --output-dir renders/ \
  --font /path/to/font.ttf \
  --format png

# Process CSV
typf batch data.csv \
  --output-dir batch_output/ \
  --parallel 8

# Process stdin
echo -e "Line 1\nLine 2\nLine 3" | typf batch - \
  --font /path/to/font.ttf \
  --font-size 20 \
  --output-dir stdin_renders/

# Custom filename template
typf batch inputs.jsonl \
  --template "render_{index:04d}_{text_hash}.png" \
  --output-dir template_output/
```

### Benchmark Command

```bash
typf bench [OPTIONS]

Options:
  -f, --font <FILE>         Font file path
      --iterations <COUNT>  Number of iterations [default: 1000]
      --text <TEXT>         Test text [default: "Benchmark test"]
      --font-size <SIZE>    Font size in points [default: 16.0]
      --backends <LIST>     Backends to test (comma-separated)
      --output <FILE>       Output JSON results
      --warmup <COUNT>      Warmup iterations [default: 100]
  -v, --verbose             Enable verbose output
  -h, --help                Print help
```

#### Examples

```bash
# Basic benchmark
typf bench \
  --font /path/to/font.ttf \
  --iterations 1000

# Compare backends
typf bench \
  --backends "skia,orge,zeno" \
  --iterations 500 \
  --output benchmark_results.json

# Detailed performance analysis
typf bench \
  --font /path/to/font.otf \
  --iterations 2000 \
  --warmup 200 \
  --verbose \
  --output detailed_benchmark.json
```

### REPL Command

```bash
typf repl [OPTIONS]

Options:
  -f, --font <FILE>         Default font file path
  -s, --font-size <SIZE>    Default font size [default: 16.0]
      --history <FILE>      Command history file
      --prompt <STRING>     Custom prompt [default: "typf> "]
      --output-dir <DIR>    Default output directory [default: .]
  -h, --help                Print help
```

#### REPL Session Example

```bash
$ typf repl --font /path/to/font.ttf
typf> render "Hello REPL" -o repl_test.png
✓ Rendered to repl_test.png (256x64)

typf> render "Multi-line
text" -o multiline.png --width 300 --height 100
✓ Rendered to multiline.png (300x100)

typf> font /path/to/font.ttf
Font: Open Sans
Weight: Regular
Style: Normal
Glyphs: 895

typf> bench --iterations 100
Benchmark results:
  Backend: skia
  Iterations: 100
  Total time: 0.234s
  Avg time: 2.34ms
  Renders/sec: 427

typf> exit
Session ended. 3 commands executed.
```

## Advanced Features

### Configuration Files

```toml
# typf.toml
[general]
default_font = "/usr/share/fonts/noto/NotoSans-Regular.ttf"
default_font_size = 16.0
default_output_dir = "./renders"

[backends]
default_shaper = "harfbuzz"
default_renderer = "skia"

[rendering]
default_dpi = 72
default_quality = 90
default_background = "white"
default_color = "black"

[batch]
default_parallel_jobs = 4
default_template = "render_{index:04d}.png"

[benchmarks]
default_iterations = 1000
default_warmup = 100
```

### Environment Variables

```bash
# Set defaults
export TYPF_FONT="/usr/share/fonts/noto/NotoSans-Regular.ttf"
export TYPF_FONT_SIZE="16.0"
export TYPF_OUTPUT_DIR="./renders"
export TYPF_VERBOSITY="1"

# Backend selection
export TYPF_SHAPER="harfbuzz"
export TYPF_RENDERER="skia"

# Performance tuning
export TYPF_PARALLEL_JOBS="8"
export TYPF_CACHE_SIZE="1000"
```

### Custom Templates

```bash
# Filename template variables
{index}          # Sequential index
{text_hash}      # Hash of text content
{timestamp}      # Unix timestamp
{font_name}      # Font family name
{font_size}      # Font size value
{format}         # Output format

# Examples
--template "img_{index:06d}.png"
--template "{text_hash}_{timestamp}.svg"
--template "{font_name}_{font_size}pt.png"
```

## Automation and Scripting

### Shell Scripts

```bash
#!/bin/bash
# generate_thumbnails.sh - Generate font thumbnails

set -e

FONT_DIR="/usr/share/fonts"
OUTPUT_DIR="./thumbnails"
FONT_SIZE=24
TEXT="Sample Text"

mkdir -p "$OUTPUT_DIR"

# Process all TTF files
find "$FONT_DIR" -name "*.ttf" -type f | while read font; do
    font_name=$(basename "$font" .ttf)
    output_file="$OUTPUT_DIR/${font_name}_thumb.png"
    
    echo "Generating thumbnail for $font_name..."
    
    typf render "$TEXT" \
        --font "$font" \
        --font-size "$FONT_SIZE" \
        --width 300 \
        --height 100 \
        --output "$output_file" \
        --verbose
done

echo "Thumbnail generation complete!"
```

### Python Integration

```python
#!/usr/bin/env python3
# batch_processor.py - Python script for batch processing

import subprocess
import json
import sys
from pathlib import Path

def run_typf_command(args):
    """Execute TYPF command and capture output."""
    cmd = ['typf'] + args
    
    try:
        result = subprocess.run(
            cmd, 
            capture_output=True, 
            text=True, 
            check=True
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError as e:
        print(f"TYPF command failed: {e}")
        print(f"Error output: {e.stderr}")
        sys.exit(1)

def process_jsonl(input_file, output_dir, font_path):
    """Process JSONL file with TYPF."""
    output_dir = Path(output_dir)
    output_dir.mkdir(exist_ok=True)
    
    with open(input_file, 'r') as f:
        for i, line in enumerate(f):
            try:
                data = json.loads(line)
                text = data['text']
                font_size = data.get('font_size', 16)
                output_format = data.get('format', 'png')
                output_file = output_dir / f"render_{i:04d}.{output_format}"
                
                # Build command
                args = [
                    'render',
                    text,
                    '--font', font_path,
                    '--font-size', str(font_size),
                    '--format', output_format,
                    '--output', str(output_file)
                ]
                
                # Add optional parameters
                if 'width' in data:
                    args.extend(['--width', str(data['width'])])
                if 'height' in data:
                    args.extend(['--height', str(data['height'])])
                if 'color' in data:
                    args.extend(['--color', data['color']])
                
                print(f"Processing: {text}")
                run_typf_command(args)
                
            except json.JSONDecodeError as e:
                print(f"Invalid JSON on line {i+1}: {e}")
                continue

def benchmark_backends(font_path, iterations=100):
    """Benchmark all available backends."""
    print("Running backend benchmarks...")
    
    # Get available backends
    info_output = run_typf_command(['info', '--backends'])
    print(f"Available backends: {info_output}")
    
    # Run benchmarks
    result = run_typf_command([
        'bench',
        '--font', font_path,
        '--iterations', str(iterations),
        '--output', 'benchmark_results.json'
    ])
    
    print(result)

if __name__ == '__main__':
    if len(sys.argv) < 4:
        print("Usage: python batch_processor.py <input.jsonl> <output_dir> <font.ttf>")
        sys.exit(1)
    
    input_file = sys.argv[1]
    output_dir = sys.argv[2]
    font_path = sys.argv[3]
    
    # Process batch
    process_jsonl(input_file, output_dir, font_path)
    
    # Run benchmarks
    benchmark_backends(font_path)
    
    print("Processing complete!")
```

### Makefile Integration

```makefile
# Makefile for text rendering project

FONT_DIR = /usr/share/fonts/noto
OUTPUT_DIR = ./renders
BATCH_FILE = inputs.jsonl
DEFAULT_FONT = $(FONT_DIR)/NotoSans-Regular.ttf

.PHONY: all batch clean benchmark info

all: batch info

batch:
	@echo "Processing batch file..."
	typf batch $(BATCH_FILE) \
		--output-dir $(OUTPUT_DIR) \
		--font $(DEFAULT_FONT) \
		--parallel 8

benchmark:
	@echo "Running benchmarks..."
	typf bench \
		--font $(DEFAULT_FONT) \
		--iterations 1000 \
		--output benchmark_results.json

info:
	@echo "System information:"
	typf info --backends
	typf info --system

clean:
	@echo "Cleaning output directory..."
	rm -rf $(OUTPUT_DIR)/*.png
	rm -rf $(OUTPUT_DIR)/*.svg
	rm -f benchmark_results.json

sample:
	@echo "Generating sample renders..."
	typf render "Sample Text" \
		--font $(DEFAULT_FONT) \
		--font-size 24 \
		--output $(OUTPUT_DIR)/sample.png
	
	typf render "Vector Sample" \
		--font $(DEFAULT_FONT) \
		--font-size 32 \
		--format svg \
		--output $(OUTPUT_DIR)/sample_vector.svg

help:
	@echo "Available targets:"
	@echo "  all       - Run batch processing and info"
	@echo "  batch     - Process batch file"
	@echo "  benchmark - Run performance benchmarks"
	@echo "  info      - Show system information"
	@echo "  clean     - Clean output files"
	@echo "  sample    - Generate sample renders"
	@echo "  help      - Show this help"
```

## Performance Optimization

### Parallel Processing

```bash
# Optimize batch processing
typf batch large_input.jsonl \
  --parallel 16 \
  --output-dir fast_renders/

# System-specific optimization
if [[ $(uname) == "Linux" ]]; then
  # Use all CPU cores
  CORES=$(nproc)
  typf batch input.jsonl --parallel $CORES
elif [[ $(uname) == "Darwin" ]]; then
  # macOS optimization
  CORES=$(sysctl -n hw.ncpu)
  typf batch input.jsonl --parallel $CORES
fi
```

### Caching

```bash
# Enable font caching
export TYPF_CACHE_ENABLED="true"
export TYPF_CACHE_SIZE="10000"
export TYPF_CACHE_DIR="$HOME/.typf/cache"

# Warm up cache
typf render "Cache warmup" --font /path/to/font.ttf -o /dev/null

# Process batch with cache
typf batch cached_input.jsonl --font-caching
```

### Memory Optimization

```bash
# Process large files in chunks
split -l 1000 large_input.jsonl chunk_
for chunk in chunk_*; do
  typf batch "$chunk" --output-dir "output_$(basename $chunk .txt)/"
  rm "$chunk"  # Cleanup
done
```

## Error Handling and Troubleshooting

### Common Issues

```bash
# Font not found
typf render "Test" --font /nonexistent/font.ttf
# Error: Font not found: /nonexistent/font.ttf
# Solution: Check font path and file permissions

# Unsupported format
typf render "Test" --format webp -o output.webp
# Error: Unsupported format: webp
# Solution: Use supported format: png, svg, pdf, pnm, json

# Memory issues with large batches
typf batch huge_input.jsonl --parallel 32
# Error: Out of memory
# Solution: Reduce parallel jobs or process in smaller chunks

# Backend not available
typf render "Test" --shaper directwrite
# Error: DirectWrite shaper not available on this platform
# Solution: Use appropriate backend for your platform
```

### Debug Mode

```bash
# Enable verbose logging
typf render "Debug test" -o debug.png --verbose --verbose

# Debug font loading
typf font /path/to/font.ttf --verbose

# Debug backend selection
typf info --backends --verbose

# Debug batch processing
typf batch input.jsonl --verbose --parallel 1
```

### Recovery Strategies

```bash
#!/bin/bash
# fallback_render.sh - Render with fallback backends

INPUT_TEXT="$1"
OUTPUT_FILE="$2"

# Try primary backend first
if typf render "$INPUT_TEXT" \
  --shaper harfbuzz \
  --renderer skia \
  --output "$OUTPUT_FILE" 2>/dev/null; then
  echo "Success with primary backend"
  exit 0
fi

# Fallback to minimal backend
if typf render "$INPUT_TEXT" \
  --shaper none \
  --renderer orge \
  --output "$OUTPUT_FILE" 2>/dev/null; then
  echo "Success with fallback backend"
  exit 0
fi

echo "All backends failed"
exit 1
```

## Integration Examples

### CI/CD Pipeline

```yaml
# .github/workflows/text-rendering.yml
name: Text Rendering Tests

on: [push, pull_request]

jobs:
  render-tests:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        
    - name: Install TYPF CLI
      run: cargo install --path crates/typf-cli --features "full"
      
    - name: Download test fonts
      run: |
        curl -L -o noto-fonts.zip https://fonts.google.com/download?family=Noto%20Sans
        unzip noto-fonts.zip
        
    - name: Run rendering tests
      run: |
        # Basic rendering test
        typf render "Hello, CI!" -o test.png --font NotoSans-Regular.ttf
        
        # Backend compatibility test
        typf render "Backend test" -o backend_test.png --shaper none --renderer orge
        
        # Batch processing test
        echo -e "Test 1\nTest 2\nTest 3" | typf batch - --font NotoSans-Regular.ttf \
          --output-dir batch_output/
          
        # Benchmark test
        typf bench --font NotoSans-Regular.ttf --iterations 100 \
          --output benchmark.json
          
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: rendering-results-${{ matrix.os }}
        path: |
          *.png
          batch_output/
          benchmark.json
```

### Docker Integration

```dockerfile
# Dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .
RUN cargo install --path crates/typf-cli --features "minimal"

FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libfontconfig1 \
    && rm -rf /var/lib/apt/lists/*

# Copy CLI binary
COPY --from=builder /usr/local/cargo/bin/typf /usr/local/bin/typf

# Add sample fonts
RUN mkdir -p /usr/share/fonts
COPY fonts/ /usr/share/fonts/

# Create output directory
RUN mkdir -p /output

WORKDIR /workspace
ENTRYPOINT ["typf"]
CMD ["--help"]
```

```bash
# Build and run Docker container
docker build -t typf-cli .

# Run rendering in container
docker run -v $(pwd)/fonts:/usr/share/fonts \
  -v $(pwd)/output:/output \
  typf-cli render "Docker test" \
  --font /usr/share/fonts/test.ttf \
  --output /output/docker_test.png

# Batch processing in container
docker run -v $(pwd)/fonts:/usr/share/fonts \
  -v $(pwd)/data:/data \
  -v $(pwd)/output:/output \
  typf-cli batch /data/input.jsonl \
  --font /usr/share/fonts/test.ttf \
  --output-dir /output/
```

The TYPF CLI provides a comprehensive, performant interface for text rendering automation, scripting, and integration into various workflows while maintaining the speed and flexibility of the underlying Rust engine.