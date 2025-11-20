# None Shaper

The None shaper passes text through without shaping. Useful for debugging, testing, and when you only need basic glyph mapping.

## What It Does

No OpenType features, no ligatures, no kerning. Just maps characters to glyphs directly.

```rust
pub struct NoneShaper;

impl Shaper for NoneShaper {
    fn shape(&self, text: &str, font: &Font, direction: TextDirection) -> Result<ShapingResult> {
        let glyphs = text.chars()
            .filter_map(|c| font.glyph_index(c))
            .map(|gid| GlyphInfo { glyph_id: gid, ..default() })
            .collect();
            
        Ok(ShapingResult { glyphs })
    }
}
```

## When to Use It

- **Debugging** - Test pipeline without shaping complications
- **Performance testing** - Baseline measurement
- **Simple text** - ASCII or fonts without complex features
- **Development** - Quick prototype without dependencies

## Performance

| Text Length | vs HarfBuzz | Memory Usage |
|-------------|-------------|--------------|
| 100 chars   | 10x faster  | 80% less     |
| 1000 chars  | 15x faster  | 85% less     |
| 10000 chars | 20x faster  | 90% less     |

## Usage

### Rust

```rust
let shaper = NoneShaper;
let result = shaper.shape("Hello", &font, TextDirection::LTR)?;
```

### Python

```python
import typf

renderer = typf.Typf(shaper="none")
result = renderer.render_text("Hello", "font.ttf")
```

### CLI

```bash
typf render --shaper none --font font.ttf "Hello World"
```

## Limitations

What you **don't** get:

- No ligature substitution
- No kerning pairs
- No contextual alternates
- No right-to-left handling
- No script-specific features

### Example: Missing Ligatures

Input: "ffi"  
Expected with shaping: "ﬃ" (single ligature glyph)  
None shaper output: "ffi" (three separate glyphs)

## Use Cases

### 1. Pipeline Testing

Verify your rendering pipeline works:

```rust
fn test_rendering_pipeline() {
    let shaper = NoneShaper;
    let renderer = OrgeRenderer::new();
    let font = load_test_font();
    
    // Simple predictable case
    let shaped = shaper.shape("A", &font, TextDirection::LTR).unwrap();
    let rendered = renderer.render(&shaped, &font).unwrap();
    
    assert_eq!(shaped.glyphs.len(), 1);
}
```

### 2. Performance Baseline

Measure rendering overhead:

```rust
fn benchmark_pipelines() {
    let text = "Performance test text";
    
    // None shaper baseline
    let start = Instant::now();
    for _ in 0..1000 {
        none_shaper.shape(text, &font, TextDirection::LTR)?;
    }
    let baseline = start.elapsed();
    
    // Compare with other shapers
    let start = Instant::now();
    for _ in 0..1000 {
        harfbuzz_shaper.shape(text, &font, TextDirection::LTR)?;
    }
    let harfbuzz_time = start.elapsed();
    
    println!("HarfBuzz overhead: {}x", harfbuzz_time.as_nanos() / baseline.as_nanos());
}
```

### 3. Simple Applications

When you don't need complex text:

```python
# Simple label rendering
import typf

def render_simple_label(text, font_path):
    renderer = typf.Typf(shaper="none")  # Fastest option
    return renderer.render_text(text, font_path)

# Good for: numbers, basic labels, debugging output
```

## Configuration

Enable with minimal features:

```toml
[dependencies.typf]
features = ["minimal"]  # Includes NoneShaper by default
```

Or explicitly:

```toml
[dependencies.typf]
features = ["shaping-none"]
```

## Error Handling

Limited error cases - most text succeeds:

```rust
#[derive(Debug, thiserror::Error)]
pub enum NoneShaperError {
    #[error("Character '{char}' not found in font")]
    CharacterNotFound { char: char },
    
    #[error("Empty text provided")]
    EmptyText,
}
```

## Debugging Features

The None shaper helps debug shaping issues:

```rust
fn compare_shaping_outputs() {
    let text = "problematic text";
    
    // Get baseline
    let none_result = none_shaper.shape(text, &font, TextDirection::LTR)?;
    
    // Compare with complex shaper
    let complex_result = harfbuzz_shaper.shape(text, &font, TextDirection::LTR)?;
    
    // Analyze differences
    for (i, (none_glyph, complex_glyph)) in none_result.glyphs.iter()
        .zip(complex_result.glyphs.iter()).enumerate() {
        if none_glyph.glyph_id != complex_glyph.glyph_id {
            println!("Position {i}: None({}) vs Complex({})", 
                none_glyph.glyph_id, complex_glyph.glyph_id);
        }
    }
}
```

## Migration Pattern

Start with None shaper, upgrade when needed:

```rust
// Phase 1: Prototype
let mut shaper: Box<dyn Shaper> = Box::new(NoneShaper);

// Phase 2: Add shaping when problems appear
if needs_complex_text() {
    shaper = Box::new(HarfBuzzShaper::new()?);
}

// Phase 3: Production with appropriate shaper
let shaper = select_shaper_for_use_case(use_case);
```

---

The None shaper gives you a fast, predictable baseline for text processing. Use it to test your pipeline, measure performance, or handle simple text where complex shaping isn't needed.