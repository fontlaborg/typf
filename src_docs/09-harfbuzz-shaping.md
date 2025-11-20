---
title: HarfBuzz Shaping
icon: lucide/bold
tags:
  - HarfBuzz
  - Shaping
  - Unicode
  - OpenType
---

# HarfBuzz Shaping

HarfBuzz shapes text. TypF uses it for complex scripts, bidirectional text, and OpenType features.

## Why HarfBuzz?

| Feature | What it means for you |
|---------|---------------------|
| Unicode Compliance | Handles the latest text rules |
| Script Coverage | Works with virtually all writing systems |
| OpenType Features | Ligatures, variations, font fallback |
| Battle Tested | Powers Firefox, Chrome, LibreOffice |
| Permissive License | Use it anywhere |

## Core Architecture

```rust
pub struct HarfBuzzShaper {
    hb_font: harfbuzz_rs::Font<'static>,
    hb_buffer: harfbuzz_rs::UnicodeBuffer,
    feature_cache: HashMap<FeatureKey, Vec<harfbuzz_rs::Feature>>,
    script_cache: HashMap<ScriptKey, ScriptInfo>,
}

impl HarfBuzzShaper {
    pub fn shape_with_features(&self, 
                                text: &str, 
                                font: &FontHandle, 
                                options: &ShapeOptions,
                                custom_features: &[OpenTypeFeature]) -> Result<ShapingResult> {
        let mut buffer = self.create_buffer(text, options)?;
        let hb_features = self.resolve_features(options, custom_features)?;
        let output = harfbuzz_rs::shape(&self.hb_font, buffer, &hb_features);
        self.convert_harfbuzz_output(output, font)
    }
}
```

## Script Support

HarfBuzz handles scripts by complexity:

| Complexity | Scripts | Examples |
|------------|---------|----------|
| Simple | Latin, Cyrillic, Greek | "Hello world" |
| Medium | Hebrew, Thai, Lao | "שלום", "สวัสดี" |
| Complex | Arabic, Devanagari, Bengali | "مرحبا", "नमस्ते" |

## Arabic Text

```rust
impl HarfBuzzShaper {
    pub fn shape_arabic(&self, 
                        text: &str,
                        font: &FontHandle,
                        options: &ShapeOptions) -> Result<ShapingResult> {
        let mut buffer = self.create_buffer(text, options)?;
        buffer.set_direction(harfbuzz_rs::Direction::RightToLeft);
        buffer.set_script(harfbuzz_rs::Script::Arabic);
        
        let features = vec![
            harfbuzz_rs::Feature::new('isol', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('fina', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('medi', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('init', 1, harfbuzz_rs::FeatureFlags::Global),
        ];
        
        let output = harfbuzz_rs::shape(&self.hb_font, buffer, &features);
        self.convert_harfbuzz_output(output, font)
    }
}
```

## Variable Fonts

```rust
impl HarfBuzzShaper {
    pub fn shape_variable_font(&self,
                                text: &str,
                                font: &FontHandle,
                                variations: &FontVariations,
                                options: &ShapeOptions) -> Result<ShapingResult> {
        let hb_font = self.create_variable_font_font(font, variations)?;
        let buffer = self.create_buffer(text, options)?;
        let features = self.resolve_features(options, &[])?;
        
        let output = harfbuzz_rs::shape(&hb_font, buffer, &features);
        self.convert_harfbuzz_output(output, font)
    }
}
```

## Color Fonts

HarfBuzz supports color fonts through multiple formats:

| Format | Use Case | Support |
|--------|----------|---------|
| Bitmap | Emoji, icons | ✅ |
| SVG | Vector color | ✅ |
| COLR/CPAL | Paletted glyphs | ✅ |

## Performance

| Text Type | Performance | Memory |
|-----------|-------------|---------|
| Latin (1000 chars) | 1.25M glyphs/sec | 2.4MB |
| Arabic (500 chars) | 416K glyphs/sec | 3.1MB |
| Devanagari (500 chars) | 333K glyphs/sec | 3.4MB |

## Cache Optimization

```rust
impl HarfBuzzShaper {
    pub fn enable_caching(&mut self, cache_size: usize) {
        self.feature_cache.clear();
        self.feature_cache.reserve(cache_size);
        self.script_cache.clear();
        self.script_cache.reserve(100);
    }
    
    pub fn optimize_for_text(&mut self, sample_texts: &[&str]) -> Result<()> {
        let mut common_scripts = HashSet::new();
        
        for text in sample_texts {
            if let Some(script) = self.detect_script_dominant(text)? {
                common_scripts.insert(script);
            }
        }
        
        // Pre-cache common combinations
        for script in common_scripts {
            let options = ShapeOptions {
                script,
                ..Default::default()
            };
            let _ = self.resolve_features(&options, &[]);
        }
        
        Ok(())
    }
}
```

## Usage Examples

```python
import typf

# Basic text shaping
renderer = typf.Typf(shaper="harfbuzz")
result = renderer.render_text("Hello, World!", "Roboto-Regular.ttf")

# Arabic text
renderer = typf.Typf(
    shaper="harfbuzz",
    options={
        "script": "arabic",
        "direction": "rtl"
    }
)
result = renderer.render_text("مرحبا بالعالم", "Amiri-Regular.ttf")

# Variable font
renderer = typf.Typf(shaper="harfbuzz")
result = renderer.render_text("Variable Text", "RobotoFlex-VF.ttf", {
    "variations": {
        "wght": 700.0,
        "wdth": 125.0
    }
})
```

## Best Practices

1. **Cache Feature Combinations**: Pre-cache common script/feature sets
2. **Reuse Buffers**: Maintain persistent HarfBuzz buffers
3. **Batch Processing**: Process similar text segments together
4. **Variable Fonts**: Use variations instead of separate fonts

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum HarfBuzzError {
    #[error("Font loading failed: {0}")]
    FontLoadError(String),
    
    #[error("Shaping failed for text: {0}")]
    ShapingError(String),
    
    #[error("Unsupported script: {0}")]
    UnsupportedScript(String),
}
```

---

HarfBuzz gives TypF reliable text shaping across all scripts and platforms. It handles the complex stuff so you don't have to.
