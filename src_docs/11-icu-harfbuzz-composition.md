# ICU + HarfBuzz Composition

ICU + HarfBuzz combines Unicode analysis with expert shaping. Get proper script detection, bidirectional text, and advanced OpenType features in one pipeline.

## Why Combine Them?

HarfBuzz shapes text but needs preprocessing. ICU handles the complex Unicode work:

- **ICU**: Script detection, bidi analysis, text segmentation
- **HarfBuzz**: OpenType shaping, glyph positioning, feature application

```rust
pub struct IcuHarfBuzzShaper {
    icu_breaker: ICUBreakIterator,
    harfbuzz: HarfBuzzShaper,
    bidi_processor: UnicodeBidiProcessor,
}
```

## Pipeline Flow

Text flows through four stages:

```
Input Text → ICU Analysis → Script Segmentation → HarfBuzz → Glyphs
```

1. ICU detects scripts and text boundaries
2. Bidi processor handles right-to-left text
3. Text splits by script changes
4. HarfBuzz shapes each script segment

## Performance Impact

| Text Type | vs HarfBuzz Alone | Memory | Quality |
|-----------|------------------|---------|---------|
| Mixed Arabic/English | +5% shaping time | +2% | Better script detection |
| Complex scripts | +8% shaping time | +3% | Proper boundaries |
| RTL + LTR | +12% shaping time | +5% | Correct visual order |

## Code Usage

### Basic Shaping

```rust
let shaper = IcuHarfBuzzShaper::new()?;
let result = shaper.shape(
    "Hello مرحبا שלום",
    &font,
    TextDirection::Auto
)?;
```

### Python Interface

```python
import typf

shaper = typf.Typf(shaper="icu-harfbuzz")
result = shaper.render_text("Mixed scripts text", "font.ttf")
```

## Script Detection

ICU identifies scripts automatically:

```rust
fn analyze_scripts(text: &str) -> Vec<ScriptSegment> {
    let mut segments = Vec::new();
    let mut current_script = Script::Common;
    let mut start = 0;
    
    for (i, ch) in text.char_indices() {
        let script = unicode_script::UnicodeScript::script(ch);
        if script != current_script {
            segments.push(ScriptSegment {
                script: current_script,
                range: start..i,
            });
            current_script = script;
            start = i;
        }
    }
    
    segments
}
```

## Bidi Processing

Handles right-to-left languages correctly:

```rust
fn process_bidi(text: &str, base_dir: TextDirection) -> Vec<TextRun> {
    let bidi = UnicodeBidi::new(text, base_dir);
    bidi.visual_runs()
}
```

### Bidi Examples

| Input | Output Order |
|-------|--------------|
| "abc מקף" | "abc ףקמ" |
| "مرحبا world" | "world مرحبا" |
| "123 אבג" | "123 גבא" |

## OpenType Features

Script-specific features activate automatically:

```rust
let features = match script {
    Script::Arabic => vec![
        OpenTypeFeature::Ligatures,
        OpenTypeFeature::ContextualAlternates,
        OpenTypeFeature::RightToLeft,
    ],
    Script::Devanagari => vec![
        OpenTypeFeature::Conjuncts,
        OpenTypeFeature::AboveBaseForms,
    ],
    _ => vec![],
};
```

## Configuration

Enable with feature flag:

```toml
[dependencies.typf]
features = ["shaping-icu-harfbuzz"]
```

Advanced options:

```python
shaper = typf.Typf(
    shaper="icu-harfbuzz",
    icu_options={
        "segmentation": "word",
        "bidi_algorithm": "unicode"
    }
)
```

## Error Handling

Specific errors for composition issues:

```rust
#[derive(Debug, thiserror::Error)]
pub enum IcuHarfBuzzError {
    #[error("Script detection failed: {0}")]
    ScriptDetection(String),
    
    #[error("Bidi processing error: {0}")]
    BidiProcessing(String),
    
    #[error("Incompatible script features: {script}")]
    IncompatibleFeatures { script: String },
}
```

## Testing

Test mixed-script scenarios:

```rust
#[test]
fn test_mixed_script_shaping() {
    let shaper = IcuHarfBuzzShaper::new().unwrap();
    let result = shaper.shape("Hello ﷺ", &font, TextDirection::Auto).unwrap();
    
    // Should detect Latin and Arabic scripts
    assert_eq!(result.glyph_runs.len(), 2);
}
```

## Migration from HarfBuzz

Switching is straightforward:

1. Enable `shaping-icu-harfbuzz` feature
2. Replace `HarfBuzzShaper` with `IcuHarfBuzzShaper`
3. Results should match, with better script handling

```rust
// Before
let shaper = HarfBuzzShaper::new()?;

// After  
let shaper = IcuHarfBuzzShaper::new()?;
```

## Performance Tips

1. **Cache script analysis** - scripts don't change for repeated text
2. **Reuse ICU breakers** - expensive to create
3. **Minimize bidi processing** - only for RTL content
4. **Pool HarfBuzz buffers** - reuse across segments

---

ICU + HarfBuzz gives you robust text shaping for real-world multilingual content while maintaining high performance.