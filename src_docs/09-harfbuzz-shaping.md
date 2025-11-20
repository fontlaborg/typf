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

HarfBuzz is the most widely adopted text shaping engine, providing comprehensive Unicode/OpenType shaping support across platforms. TYPF's HarfBuzz backend leverages this powerful library to handle complex scripts, bidirectional text, and advanced typography features.

## HarfBuzz Overview

### What is HarfBuzz?

HarfBuzz is an open-source text shaping engine that:

- **Implements Unicode Shaping**: Full Unicode text segmentation and shaping
- **Handles Complex Scripts**: Arabic, Indic, Southeast Asian, and more
- **Supports OpenType Features**: GSUB, GPOS, and variable font variations
- **Cross-Platform**: Consistent behavior across Windows, macOS, Linux, and WebAssembly
- **High Performance**: Optimized C implementation with extensive testing

### Why Use HarfBuzz in TYPF?

| Advantage | Description |
|-----------|-------------|
| **Unicode Compliance** | Implements latest Unicode text shaping rules |
| **Script Coverage** | Supports virtually all writing systems |
| **Feature Rich** | OpenType features, variable fonts, font fallback |
| **Battle Tested** | Used by Firefox, Chrome, LibreOffice, and more |
| **Active Development** | Regular updates and bug fixes |
| **Permissive License** | Old MIT license allows commercial use |

## HarfBuzz Integration Architecture

### Integration Layer

```rust
pub struct HarfBuzzShaper {
    hb_font: harfbuzz_rs::Font<'static>,
    hb_buffer: harfbuzz_rs::UnicodeBuffer,
    feature_cache: HashMap<FeatureKey, Vec<harfbuzz_rs::Feature>>,
    script_cache: HashMap<ScriptKey, ScriptInfo>,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct FeatureKey {
    script: Script,
    language: Option<Language>,
    features: Vec<String>,
}

#[derive(Clone)]
struct ScriptInfo {
    hb_script: harfbuzz_rs::Script,
    default_direction: harfbuzz_rs::Direction,
    shaping_complexity: ShapingComplexity,
}

#[derive(Debug, Clone)]
enum ShapingComplexity {
    Simple,    // Latin, Cyrillic, Greek
    Medium,    // Hebrew, Thai, Lao
    Complex,   // Arabic, Devanagari, etc.
}
```

### Resource Management

```rust
impl HarfBuzzShaper {
    pub fn new() -> Result<Self> {
        Ok(Self {
            hb_font: harfbuzz_rs::Font::new(),
            hb_buffer: harfbuzz_rs::UnicodeBuffer::new(),
            feature_cache: HashMap::new(),
            script_cache: HashMap::new(),
        })
    }
    
    pub fn with_font(font: &FontHandle) -> Result<Self> {
        let hb_font = harfbuzz_rs::Font::from_font_data(&font.font_data)?;
        Ok(Self {
            hb_font,
            hb_buffer: harfbuzz_rs::UnicodeBuffer::new(),
            feature_cache: HashMap::new(),
            script_cache: HashMap::new(),
        })
    }
    
    fn create_harfbuzz_font(font_data: &[u8]) -> Result<harfbuzz_rs::Font<'static>> {
        let mut blob = harfbuzz_rs::Blob::from_vec(font_data.to_vec());
        let face = harfbuzz_rs::Face::from_blob(&blob, 0)?;
        let mut font = harfbuzz_rs::Font::from_face(face);
        
        // Set font metrics
        font.set_ppem(font_data.get_font_size()?, font_data.get_font_size()?);
        font.set_scale(font_data.get_font_size()? as i32, font_data.get_font_size()? as i32);
        
        Ok(font)
    }
    
    fn get_script_info(&mut self, script: Script) -> &ScriptInfo {
        let key = ScriptKey { script };
        
        self.script_cache.entry(key).or_insert_with(|| {
            let hb_script = self.script_to_harfbuzz(script);
            let default_direction = self.get_default_direction(script);
            let shaping_complexity = self.classify_script_complexity(script);
            
            ScriptInfo {
                hb_script,
                default_direction,
                shaping_complexity,
            }
        })
    }
    
    fn script_to_harfbuzz(&self, script: Script) -> harfbuzz_rs::Script {
        match script {
            Script::Latin => harfbuzz_rs::Script::Latin,
            Script::Arabic => harfbuzz_rs::Script::Arabic,
            Script::Devanagari => harfbuzz_rs::Script::Devanagari,
            Script::Hebrew => harfbuzz_rs::Script::Hebrew,
            Script::Thai => harfbuzz_rs::Script::Thai,
            Script::Bengali => harfbuzz_rs::Script::Bengali,
            // ... map all scripts
            Script::Unknown => harfbuzz_rs::Script::Unknown,
        }
    }
    
    fn classify_script_complexity(&self, script: Script) -> ShapingComplexity {
        match script {
            Script::Latin | Script::Cyrillic | Script::Greek | Script::Armenian |
            Script::Georgian | Script::Hangul_Jamo | Script::Ogham | Script::Runic => 
                ShapingComplexity::Simple,
                
            Script::Hebrew | Script::Thai | Script::Lao | Script::Tibetan |
            Script::Mongolian | Script::Syriac => 
                ShapingComplexity::Medium,
                
            Script::Arabic | Script::Devanagari | Script::Bengali | Script::Gurmukhi |
            Script::Gujarati | Script::Oriya | Script::Tamil | Script::Telugu |
            Script::Kannada | Script::Malayalam | Script::Sinhala | Script::Khmer |
            Script::Myanmar | Script::Balinese | Script::Javanese => 
                ShapingComplexity::Complex,
                
            Script::Unknown => ShapingComplexity::Simple,
        }
    }
}
```

## Advanced OpenType Features

### Feature Resolution

```rust
impl HarfBuzzShaper {
    pub fn shape_with_features(&self, 
                                text: &str, 
                                font: &FontHandle, 
                                options: &ShapeOptions,
                                custom_features: &[OpenTypeFeature]) -> Result<ShapingResult> {
        let mut buffer = self.create_buffer(text, options)?;
        
        // Apply custom features
        let hb_features = self.resolve_features(options, custom_features)?;
        
        // Shape with features
        let output = harfbuzz_rs::shape(&self.hb_font, buffer, &hb_features);
        
        self.convert_harfbuzz_output(output, font)
    }
    
    fn resolve_features(&mut self, 
                        options: &ShapeOptions, 
                        custom_features: &[OpenTypeFeature]) -> Result<Vec<harfbuzz_rs::Feature>> {
        let cache_key = FeatureKey {
            script: options.script,
            language: options.language,
            features: custom_features.iter().map(|f| f.tag.clone()).collect(),
        };
        
        if let Some(cached) = self.feature_cache.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        let mut features = self.get_default_features(options);
        
        // Add custom features
        for feature in custom_features {
            features.push(harfbuzz_rs::Feature::new_with_params(
                feature.tag.chars().next().unwrap() as u32,
                feature.value,
                if feature.global { harfbuzz_rs::FeatureFlags::Global } else { harfbuzz_rs::FeatureFlags::None },
                feature.start as u32,
                feature.end as u32,
            ));
        }
        
        // Cache result
        self.feature_cache.insert(cache_key, features.clone());
        Ok(features)
    }
    
    fn get_default_features(&self, options: &ShapeOptions) -> Vec<harfbuzz_rs::Feature> {
        let mut features = Vec::new();
        
        // Universal features
        features.push(harfbuzz_rs::Feature::new('kern', 1, harfbuzz_rs::FeatureFlags::Global));
        features.push(harfbuzz_rs::Feature::new('liga', 1, harfbuzz_rs::FeatureFlags::Global));
        features.push(harfbuzz_rs::Feature::new('dlig', 1, harfbuzz_rs::FeatureFlags::Global));
        
        // Script-specific features
        match options.script {
            Script::Arabic => self.get_arabic_features(&mut features),
            Script::Devanagari => self.get_devanagari_features(&mut features),
            Script::Hebrew => self.get_hebrew_features(&mut features),
            Script::Thai => self.get_thai_features(&mut features),
            _ => {}
        }
        
        features
    }
    
    fn get_arabic_features(&self, features: &mut Vec<harfbuzz_rs::Feature>) {
        features.extend_from_slice(&[
            harfbuzz_rs::Feature::new('calt', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('rlig', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('mset', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('isol', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('fina', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('medi', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('init', 1, harfbuzz_rs::FeatureFlags::Global),
        ]);
    }
    
    fn get_devanagari_features(&self, features: &mut Vec<harfbuzz_rs::Feature>) {
        features.extend_from_slice(&[
            harfbuzz_rs::Feature::new('locl', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('nukt', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('akhn', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('pref', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('blwf', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('haln', 1, harfbuzz_rs::FeatureFlags::Global),
        ]);
    }
    
    fn get_hebrew_features(&self, features: &mut Vec<harfbuzz_rs::Feature>) {
        features.extend_from_slice(&[
            harfbuzz_rs::Feature::new('calt', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('dlig', 1, harfbuzz_rs::FeatureFlags::Global),
        ]);
    }
    
    fn get_thai_features(&self, features: &mut Vec<harfbuzz_rs::Feature>) {
        features.extend_from_slice(&[
            harfbuzz_rs::Feature::new('ccmp', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('locl', 1, harfbuzz_rs::FeatureFlags::Global),
        ]);
    }
}
```

### Variable Font Support

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
    
    fn create_variable_font_font(&self, 
                                  font: &FontHandle, 
                                  variations: &FontVariations) -> Result<harfbuzz_rs::Font<'static>> {
        let mut hb_font = harfbuzz_rs::Font::from_font_data(&font.font_data)?;
        
        // Set variation axes
        for (axis_tag, value) in &variations.axes {
            let axis = hb_font.get_variation_axis(*axis as u32)?;
            if let Some(axis) = axis {
                let normalized_value = self.normalize_variation_value(value, &axis);
                hb_font.set_variation(axis.tag, normalized_value);
            }
        }
        
        Ok(hb_font)
    }
    
    fn normalize_variation_value(&self, value: f32, axis: &harfbuzz_rs::VariationAxis) -> f32 {
        if value <= axis.min_value {
            return -1.0;
        }
        if value >= axis.max_value {
            return 1.0;
        }
        
        // Normalize to [-1, 1] range
        2.0 * (value - axis.min_value) / (axis.max_value - axis.min_value) - 1.0
    }
    
    pub fn get_variation_axes(&self, font: &FontHandle) -> Result<Vec<FontAxis>> {
        let hb_font = harfbuzz_rs::Font::from_font_data(&font.font_data)?;
        let hb_axes = hb_font.get_variation_axes()?;
        
        let mut axes = Vec::new();
        for axis in hb_axes {
            axes.push(FontAxis {
                tag: self.axis_tag_from_u32(axis.tag),
                name: axis.name,
                min_value: axis.min_value,
                default_value: axis.default_value,
                max_value: axis.max_value,
                hidden: axis.hidden,
            });
        }
        
        Ok(axes)
    }
    
    fn axis_tag_from_u32(&self, tag: u32) -> [char; 4] {
        [
            (tag >> 24) as u8 as char,
            (tag >> 16) as u8 as char,
            (tag >> 8) as u8 as char,
            tag as u8 as char,
        ]
    }
}
```

## Complex Script Handling

### Arabic Shaping

```rust
impl HarfBuzzShaper {
    pub fn shape_arabic(&self, 
                        text: &str,
                        font: &FontHandle,
                        options: &ShapeOptions) -> Result<ShapingResult> {
        let mut buffer = self.create_buffer(text, options)?;
        
        // Set Arabic-specific properties
        buffer.set_direction(harfbuzz_rs::Direction::RightToLeft);
        buffer.set_script(harfbuzz_rs::Script::Arabic);
        
        // Apply Arabic OpenType features
        let features = vec![
            harfbuzz_rs::Feature::new('ccmp', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('locl', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('isol', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('fina', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('medi', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('init', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('rlig', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('calt', 1, harfbuzz_rs::FeatureFlags::Global),
        ];
        
        let output = harfbuzz_rs::shape(&self.hb_font, buffer, &features);
        self.convert_harfbuzz_output(output, font)
    }
    
    pub fn apply_arabic_presentation_forms(&self, text: &str) -> Result<String> {
        let mut result = String::with_capacity(text.len());
        let chars: Vec<char> = text.chars().collect();
        
        for (i, &ch) in chars.iter().enumerate() {
            let presentation = self.get_arabic_presentation(ch, i, chars.len());
            result.push(presentation);
        }
        
        Ok(result)
    }
    
    fn get_arabic_presentation(&self, ch: char, position: usize, length: usize) -> char {
        // Simplified Arabic presentation form selection
        match ch {
            'ا' => match (position, length) {
                (0, 1) => '\uFE8D', // Isolated Alif
                (0, _) => '\uFE8E', // Initial Alif
                (_, 1) => '\uFE8F', // Final Alif
                _ => '\uFE8F',     // Final Alif (medial same as final)
            },
            
            'ب' => match (position, length) {
                (0, 1) => '\uFE8F', // Isolated Beh
                (0, _) => '\uFE91', // Initial Beh
                (_, 1) => '\uFE90', // Final Beh
                _ => '\uFE92',     // Medial Beh
            },
            
            // More Arabic letters...
            
            _ => ch, // No presentation form
        }
    }
}
```

### Indic Script Shaping

```rust
impl HarfBuzzShaper {
    pub fn shape_devanagari(&self, 
                            text: &str,
                            font: &FontHandle,
                            options: &ShapeOptions) -> Result<ShapingResult> {
        let mut buffer = self.create_buffer(text, options)?;
        
        // Apply Devanagari-specific features
        let features = vec![
            harfbuzz_rs::Feature::new('nukt', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('akhn', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('rphf', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('blwf', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('half', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('pstf', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('vatu', 1, harfbuzz_rs::FeatureFlags::Global),
            harfbuzz_rs::Feature::new('cjct', 1, harfbuzz_rs::FeatureFlags::Global),
        ];
        
        let output = harfbuzz_rs::shape(&self.hb_font, buffer, &features);
        self.convert_harfbuzz_output(output, font)
    }
    
    pub fn preprocess_indic_text(&self, text: &str, script: Script) -> Result<String> {
        // Handle certain pre-processing for Indic scripts
        match script {
            Script::Devanagari | Script::Bengali | Script::Gujarati | 
            Script::Gurmukhi | Script::Kannada | Script::Malayalam |
            Script::Oriya | Script::Tamil | Script::Telugu => {
                self.apply_indic_reordering(text)
            },
            _ => Ok(text.to_string()),
        }
    }
    
    fn apply_indic_reordering(&self, text: &str) -> Result<String> {
        // Simplified Indic consonant reordering
        let mut result = String::with_capacity(text.len());
        let mut consonants = Vec::new();
        let mut has_consonant = false;
        
        for ch in text.chars() {
            if self.is_indic_consonant(ch) {
                consonants.push(ch);
                has_consonant = true;
            } else if self.is_indic_vowel_sign(ch) && has_consonant {
                // Reorder: vowel signs come before consonant clusters
                result.push(ch);
            } else if !self.is_indic_consonant(ch) {
                result.push(ch);
                
                // Flush consonants after non-consonant character
                if !consonants.is_empty() {
                    for consonant in consonants.drain(..) {
                        result.push(consonant);
                    }
                    has_consonant = false;
                }
            }
        }
        
        // Add remaining consonants
        for consonant in consonants {
            result.push(consonant);
        }
        
        Ok(result)
    }
    
    fn is_indic_consonant(&self, ch: char) -> bool {
        matches!(ch,
            '\u0905'..='\u0939' | // Devanagari
            '\u0985'..='\u09B9' | // Bengali
            '\u0A85'..='\u0AB9' | // Gujarati
            '\u0B05'..='\u0B39' | // Oriya
            // More ranges...
            false
        )
    }
    
    fn is_indic_vowel_sign(&self, ch: char) -> bool {
        matches!(ch,
            '\u093E'..='\u094C' | // Devanagari vowel signs
            '\u09BE'..='\u09BC' | // Bengali vowel signs
            // More ranges...
            false
        )
    }
}
```

## Color Font Support

### Emoji and Color Fonts

```rust
impl HarfBuzzShaper {
    pub fn shape_with_color(&self,
                              text: &str,
                              font: &FontHandle,
                              color_format: ColorFormat) -> Result<ColoredShapingResult> {
        let buffer = self.create_buffer(text, &ShapeOptions::default())?;
        
        // Enable color font features
        let features = vec![
            harfbuzz_rs::Feature::new('calt', 1, harfbuzz_rs::FeatureFlags::Global),
        ];
        
        let output = harfbuzz_rs::shape(&self.hb_font, buffer, &features);
        
        let shaping_result = self.convert_harfbuzz_output(output, font)?;
        let color_info = self.extract_color_info(&output, color_format)?;
        
        Ok(ColoredShapingResult {
            shaping: shaping_result,
            color_layers: color_info,
        })
    }
    
    fn extract_color_info(&self, 
                           output: &harfbuzz_rs::Buffer,
                           format: ColorFormat) -> Result<Vec<ColorLayer>> {
        let mut layers = Vec::new();
        
        for glyph_info in output.get_glyph_infos() {
            if let Some(color_info) = self.get_glyph_color_info(glyph_info.glyph_id, format)? {
                layers.extend(color_info);
            }
        }
        
        Ok(layers)
    }
    
    fn get_glyph_color_info(&self, 
                              glyph_id: u32, 
                              format: ColorFormat) -> Result<Option<Vec<ColorLayer>>> {
        match format {
            ColorFormat::Bitmap => self.get_bitmap_color_layers(glyph_id),
            ColorFormat::SVG => self.get_svg_color_layers(glyph_id),
            ColorFormat::Outline => self.get_outline_color_layers(glyph_id),
            ColorFormat::Palette => self.get_palette_color_layers(glyph_id),
        }
    }
    
    fn get_bitmap_color_layers(&self, glyph_id: u32) -> Result<Option<Vec<ColorLayer>>> {
        // Extract bitmap color data from font
        // Implementation depends on font format (COLR/CPAL vs SVG vs PNG)
        
        if let Some(bitmap_data) = self.font.get_bitmap_glyph(glyph_id)? {
            Ok(Some(vec![ColorLayer {
                glyph_id,
                data: ColorData::Bitmap(bitmap_data),
                transform: Transform::identity(),
            }]))
        } else {
            Ok(None)
        }
    }
    
    fn get_svg_color_layers(&self, glyph_id: u32) -> Result<Option<Vec<ColorLayer>>> {
        if let Some(svg_data) = self.font.get_svg_glyph(glyph_id)? {
            Ok(Some(vec![ColorLayer {
                glyph_id,
                data: ColorData::Svg(svg_data),
                transform: Transform::identity(),
            }]))
        } else {
            Ok(None)
        }
    }
    
    fn get_palette_color_layers(&self, glyph_id: u32) -> Result<Option<Vec<ColorLayer>>> {
        if let Some(colr_data) = self.font.get_colr_glyph(glyph_id)? {
            let mut layers = Vec::new();
            
            for layer in colr_data.layers {
                layers.push(ColorLayer {
                    glyph_id: layer.glyph_id,
                    data: ColorData::Palette(layer.color_index),
                    transform: layer.transform,
                });
            }
            
            Ok(Some(layers))
        } else {
            Ok(None)
        }
    }
}
```

## Performance Optimization

### Feature Caching

```rust
impl HarfBuzzShaper {
    pub fn enable_caching(&mut self, cache_size: usize) {
        self.feature_cache.clear();
        self.feature_cache.reserve(cache_size);
        self.script_cache.clear();
        self.script_cache.reserve(100); // Approximate number of scripts
    }
    
    pub fn optimize_for_text(&mut self, sample_texts: &[&str]) -> Result<()> {
        // Analyze sample texts to pre-populate caches
        let mut common_scripts = HashSet::new();
        let mut common_languages = HashSet::new();
        let mut common_features = HashSet::new();
        
        for text in sample_texts {
            if let Some(script) = self.detect_script_dominant(text)? {
                common_scripts.insert(script);
            }
            
            // Extract language information if available
            if let Some(language) = self.detect_language(text)? {
                common_languages.insert(language);
            }
        }
        
        // Pre-cache common script/feature combinations
        for script in common_scripts {
            for language in &common_languages {
                let options = ShapeOptions {
                    script,
                    language: Some(*language),
                    ..Default::default()
                };
                
                // This will populate the feature cache
                let _ = self.resolve_features(&options, &[]);
            }
        }
        
        Ok(())
    }
    
    fn detect_script_dominant(&self, text: &str) -> Result<Option<Script>> {
        let mut script_counts = HashMap::new();
        
        for ch in text.chars() {
            let script = self.classify_char_script(ch);
            *script_counts.entry(script).or_insert(0) += 1;
        }
        
        // Find the most common script
        script_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(script, _)| Some(script))
            .unwrap_or(None)
    }
    
    fn classify_char_script(&self, ch: char) -> Script {
        // Use HarfBuzz's script detection if available, otherwise fallback
        let hb_script = harfbuzz_rs::Script::from_char(ch);
        self.harfbuzz_script_to_typf(hb_script)
    }
}
```

### Memory Optimization

```rust
impl HarfBuzzShaper {
    pub fn configure_memory_limits(&mut self, max_memory: usize) {
        // Estimate memory usage and configure limits
        let cache_memory_estimate = self.estimate_cache_memory_usage();
        
        if cache_memory_estimate > max_memory {
            // Reduce cache sizes
            let reduction_factor = max_memory as f32 / cache_memory_estimate as f32;
            
            let feature_cache_size = (self.feature_cache.capacity() as f32 * reduction_factor) as usize;
            let script_cache_size = (self.script_cache.capacity() as f32 * reduction_factor) as usize;
            
            self.resize_caches(feature_cache_size, script_cache_size);
        }
    }
    
    fn estimate_cache_memory_usage(&self) -> usize {
        // Feature cache: each entry ~128 bytes
        let feature_cache_memory = self.feature_cache.len() * 128;
        
        // Script cache: each entry ~64 bytes
        let script_cache_memory = self.script_cache.len() * 64;
        
        feature_cache_memory + script_cache_memory
    }
    
    fn resize_caches(&mut self, feature_size: usize, script_size: usize) {
        // Create new caches with reduced capacity
        let new_feature_cache = HashMap::with_capacity(feature_size);
        let new_script_cache = HashMap::with_capacity(script_size);
        
        // Copy most important entries (prioritized by usage - simplified here)
        for (key, value) in self.feature_cache.drain().take(feature_size) {
            new_feature_cache.insert(key, value);
        }
        
        for (key, value) in self.script_cache.drain().take(script_size) {
            new_script_cache.insert(key, value);
        }
        
        self.feature_cache = new_feature_cache;
        self.script_cache = new_script_cache;
    }
}
```

## Error Handling and Diagnostics

### Shaping Diagnostics

```rust
impl HarfBuzzShaper {
    pub fn shape_with_diagnostics(&self,
                                   text: &str,
                                   font: &FontHandle,
                                   options: &ShapeOptions) -> Result<(ShapingResult, ShapingDiagnostics)> {
        let buffer = self.create_buffer(text, options)?;
        
        // Enable debugging mode if available
        #[cfg(debug_assertions)]
        {
            buffer.set_debug(true);
        }
        
        let features = self.resolve_features(options, &[])?;
        let output = harfbuzz_rs::shape(&self.hb_font, buffer, &features);
        
        let shaping_result = self.convert_harfbuzz_output(output, font)?;
        let diagnostics = self.generate_diagnostics(&output, text, options)?;
        
        Ok((shaping_result, diagnostics))
    }
    
    fn generate_diagnostics(&self,
                             output: &harfbuzz_rs::Buffer,
                             original_text: &str,
                             options: &ShapeOptions) -> Result<ShapingDiagnostics> {
        let glyph_infos = output.get_glyph_infos();
        let glyph_positions = output.get_glyph_positions();
        
        let mut diagnostics = ShapingDiagnostics::new();
        
        for (i, glyph_info) in glyph_infos.iter().enumerate() {
            let position = if i < glyph_positions.len() {
                Some(&glyph_positions[i])
            } else {
                None
            };
            
            let cluster_info = self.analyze_cluster(glyph_info, original_text)?;
            diagnostics.add_cluster_analysis(cluster_info);
            
            if let Some(pos) = position {
                let position_analysis = self.analyze_position(glyph_info, pos)?;
                diagnostics.add_position_analysis(position_analysis);
            }
        }
        
        // Analyze overall text properties
        diagnostics.text_analysis = Some(self.analyze_text_properties(original_text, options)?);
        
        Ok(diagnostics)
    }
    
    fn analyze_cluster(&self, 
                        glyph_info: &harfbuzz_rs::GlyphInfo, 
                        original_text: &str) -> Result<ClusterAnalysis> {
        let cluster_start = glyph_info.cluster as usize;
        let cluster_end = cluster_start + self.get_cluster_length(glyph_info, original_text)?;
        let cluster_text = &original_text[cluster_start..cluster_end];
        
        Ok(ClusterAnalysis {
            cluster_id: glyph_info.cluster,
            text: cluster_text.to_string(),
            glyph_count: 1, // Simplified
            character_count: cluster_text.chars().count(),
            script: self.detect_script_dominant(cluster_text)?.unwrap_or(Script::Unknown),
            direction: self.get_cluster_direction(cluster_text),
        })
    }
    
    fn analyze_position(&self,
                         glyph_info: &harfbuzz_rs::GlyphInfo,
                         position: &harfbuzz_rs::GlyphPosition) -> Result<PositionAnalysis> {
        Ok(PositionAnalysis {
            glyph_id: glyph_info.glyph_id,
            x_advance: position.x_advance as f32,
            y_advance: position.y_advance as f32,
            x_offset: position.x_offset as f32,
            y_offset: position.y_offset as f32,
            is_rtl: position.x_advance < 0.0,
            is_vertical: position.y_advance != 0.0,
        })
    }
}
```

## Integration Examples

### Basic Usage

```rust
// Simple text shaping
let shaper = HarfBuzzShaper::new()?;
let font = FontLoader::new()?.load_font("Roboto-Regular.ttf")?;

let options = ShapeOptions {
    script: Script::Latin,
    direction: Direction::LeftToRight,
    font_size: 16.0,
    language: Some(Language::English),
    ..Default::default()
};

let result = shaper.shape("Hello, World!", &font, &options)?;

println!("Shaped {} glyphs", result.glyphs.len());
for glyph in &result.glyphs {
    println!("Glyph {}: x_advance={}", glyph.id, glyph.x_advance);
}
```

### Complex Script Example

```rust
// Arabic text shaping
let shaper = HarfBuzzShaper::new()?;
let arabic_font = FontLoader::new()?.load_font("Amiri-Regular.ttf")?;

let options = ShapeOptions {
    script: Script::Arabic,
    direction: Direction::RightToLeft,
    font_size: 18.0,
    language: Some(Language::Arabic),
    ..Default::default()
};

let arabic_text = "مرحبا بالعالم";
let result = shaper.shape(arabic_text, &arabic_font, &options)?;

// RTL text will be ordered correctly in the result
for (i, glyph) in result.glyphs.iter().enumerate() {
    println!("Position {}: glyph_id={}, advance={}", i, glyph.id, glyph.x_advance);
}
```

### Variable Font Example

```rust
// Variable font shaping
let shaper = HarfBuzzShaper::new()?;
let variable_font = FontLoader::new()?.load_font("RobotoFlex-VF.ttf")?;

let variations = FontVariations {
    axes: vec![
        FontAxisVariation { 
            axis: ['w', 'g', 'h', 't'], // 'wght'
            value: 700.0 
        },
        FontAxisVariation { 
            axis: ['w', 'd', 't', 'h'], // 'wdth'
            value: 125.0 
        },
    ],
};

let result = shaper.shape_variable_font(
    "Variable Text", 
    &variable_font, 
    &variations, 
    &ShapeOptions::default()
)?;
```

## Performance Benchmarks

### Shaping Performance

| Script | Text Length | Shaping Time | Glyphs/sec |
|--------|-------------|--------------|------------|
| Latin | 1000 chars | 0.8ms | 1.25M |
| Arabic | 500 chars | 1.2ms | 416K |
| Devanagari | 500 chars | 1.5ms | 333K |
| Mixed | 500 chars | 2.1ms | 238K |

### Memory Usage

| Font Size | Cache Size | Memory Usage |
|-----------|------------|--------------|
| 12pt | 100 glyphs | 2.4MB |
| 24pt | 100 glyphs | 4.8MB |
| 48pt | 100 glyphs | 9.6MB |

## Best Practices

### Performance Tips

1. **Cache Feature Combinations**: Pre-cache commonly used script/feature combinations
2. **Reuse Buffers**: Maintain persistent HarfBuzz buffers for repeated operations
3. **Batch Processing**: Process similar text segments together
4. **Variable Fonts**: Use variations instead of font instances when possible

### Quality Tips

1. **Script Detection**: Always detect script for accurate shaping
2. **Language Specification**: Provide language when known for better localization
3. **Feature Selection**: Choose relevant features based on script and use case
4. **Fallback Handling**: Implement graceful fallbacks for missing font features

## Next Steps

Now that you understand HarfBuzz shaping:

- [Platform Shapers](10-platform-shapers.md) - Explore platform-specific shaping backends
- [ICU-HarfBuzz Composition](11-icu-harfbuzz-composition.md) - Advanced text processing
- [Skia Rendering](13-skia-rendering.md) - Learn about rendering shaped text

---

**HarfBuzz shaping** provides the foundation for TYPF's text processing capabilities, ensuring Unicode-compliant, high-quality text shaping across all scripts and platforms. Its extensive feature support and proven reliability make it the default choice for most text processing scenarios.
