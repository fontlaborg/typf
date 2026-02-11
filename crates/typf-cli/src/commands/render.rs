//! Render command implementation
//!
//! Handles text rendering with full pipeline control.
//! Supports both traditional (shaper+renderer) and linra (single-pass) modes.
//
// this_file: crates/typf-cli/src/commands/render.rs

use crate::cli::{OutputFormat, RenderArgs};
use skrifa::bitmap::BitmapStrikes;
use skrifa::raw::TableProvider;
use std::fs::File;
use std::io::{self, Read, Write};
use std::sync::Arc;
use typf::error::{Result, TypfError};
#[cfg(feature = "linra")]
use typf_core::linra::{LinraRenderParams, LinraRenderer};
use typf_core::Pipeline;
use typf_core::{
    traits::{Exporter, FontRef, Renderer, Shaper},
    types::{Direction, RenderOutput, VectorFormat},
    Color, GlyphSource, GlyphSourcePreference, RenderMode, RenderParams, ShapingParams,
    MAX_FONT_SIZE,
};
use typf_export::{PngExporter, PnmExporter};
use typf_fontdb::TypfFontFace;
use typf_render_opixa::OpixaRenderer;
use typf_render_svg::SvgRenderer;
use typf_shape_none::NoneShaper;
use typf_unicode::{UnicodeOptions, UnicodeProcessor};

pub fn run(args: &RenderArgs) -> Result<()> {
    // Check if using linra (single-pass) renderer
    // "auto" now defaults to linra if available (unless SVG output requested)
    let use_linra = if args.renderer == "auto" {
        // Auto-select: use linra if available, but not for SVG output
        #[cfg(feature = "linra")]
        {
            !matches!(args.format, OutputFormat::Svg)
        }
        #[cfg(not(feature = "linra"))]
        {
            false
        }
    } else {
        is_linra_renderer(&args.renderer)
    };

    // Track if we're falling back from linra for SVG export
    let svg_fallback_shaper: Option<&str> = if use_linra && matches!(args.format, OutputFormat::Svg)
    {
        // SVG export extracts glyph outlines from font after shaping.
        // Linra combines shaping+rendering atomically, so we can't get shaping result.
        // Fall back to the matching system shaper for consistent results.
        let fallback = match args.renderer.as_str() {
            "linra-mac" | "linra" => {
                #[cfg(feature = "shaping-ct")]
                {
                    Some("ct")
                }
                #[cfg(not(feature = "shaping-ct"))]
                {
                    Some("hb")
                }
            },
            "linra-win" => {
                // TODO: DirectWrite shaper when available
                Some("hb")
            },
            _ => Some("hb"),
        };
        eprintln!(
            "⚠ SVG export needs shaping results. Falling back to {} shaper \
             (linra combines shaping+rendering atomically).",
            fallback.unwrap_or("default")
        );
        fallback
    } else if use_linra {
        #[cfg(feature = "linra")]
        return run_linra(args);

        #[cfg(not(feature = "linra"))]
        return Err(TypfError::Other(format!(
            "Linra renderer '{}' requested but linra feature is not enabled. \
             Build with --features linra-mac or linra-win",
            args.renderer
        )));
    } else {
        None
    };

    // Traditional pipeline (shaper + renderer)

    // 1. Get input text
    let text = get_input_text(args)?;

    if !args.quiet {
        eprintln!("Typf v{}", env!("CARGO_PKG_VERSION"));
        eprintln!("Rendering text...");
    }

    // 2. Load font
    let font: Arc<dyn FontRef> = load_font(args)?;

    // 3. Resolve direction (auto uses UnicodeProcessor)
    let direction = resolve_direction(
        &text,
        &args.direction,
        args.language.as_deref(),
        script_hint(&args.script),
    )?;

    // 4. Parse rendering parameters
    let font_size = parse_font_size(&args.font_size)?;
    let foreground = parse_color(&args.foreground)?;
    let background = parse_color(&args.background)?;

    // 5. Parse variable font variations
    let variations = parse_variations(&args.instance)?;

    // 6. Create shaping parameters
    let shaping_params = ShapingParams {
        size: font_size,
        direction,
        language: args.language.clone(),
        script: if args.script == "auto" {
            None
        } else {
            Some(args.script.clone())
        },
        features: parse_features(&args.features)?,
        variations: variations.clone(),
        letter_spacing: 0.0,
    };

    // 7. Create rendering parameters
    let output_mode = if matches!(args.format, OutputFormat::Svg) {
        RenderMode::Vector(VectorFormat::Svg)
    } else {
        RenderMode::Bitmap
    };

    let glyph_sources = parse_glyph_sources(&args.glyph_source)?;

    let render_params = RenderParams {
        foreground,
        background: Some(background),
        padding: args.margin,
        antialias: !matches!(args.format, OutputFormat::Pbm | OutputFormat::Png1),
        variations,
        color_palette: args.color_palette as u16,
        glyph_sources,
        output: output_mode,
    };

    let (shaping_cache_on, glyph_cache_on, caching_allowed) = cache_flags(
        use_linra,
        svg_fallback_shaper.is_some(),
        args.no_shaping_cache,
        args.no_glyph_cache,
    );

    // 8. Select backends
    // Use fallback shaper if we're falling back from linra for SVG
    let shaper_name = svg_fallback_shaper.unwrap_or(&args.shaper);
    let shaper = select_shaper(shaper_name)?;

    // Prefer requested renderer; if SVG output is requested but the renderer
    // cannot emit SVG, fall back to the dedicated SVG renderer.
    let mut renderer_name = args.renderer.as_str();
    let mut renderer = select_renderer(renderer_name)?;

    if matches!(args.format, OutputFormat::Svg) && !renderer.supports_format("svg") {
        renderer_name = "svg";
        renderer = select_renderer(renderer_name)?;
    }

    warn_if_vello_gpu_with_color_font(renderer_name, font.data(), args.quiet);

    // 9. Build pipeline
    let exporter = create_exporter(args.format)?;
    let pipeline = Pipeline::builder()
        .enable_shaping_cache(shaping_cache_on)
        .enable_glyph_cache(glyph_cache_on)
        .shaper(shaper.clone())
        .renderer(renderer.clone())
        .exporter(exporter.clone())
        .build()?;

    if args.verbose {
        eprintln!("Shaping with {} backend...", shaper_name);
        eprintln!("Rendering with {} backend...", renderer_name);
        eprintln!("Exporting to {} format...", args.format.as_str());
        eprintln!(
            "Caching: shaping={} glyph={}{}",
            if shaping_cache_on { "on" } else { "off" },
            if glyph_cache_on { "on" } else { "off" },
            if !caching_allowed {
                " (linra requested; disabled)"
            } else {
                ""
            }
        );
    }

    // 10. Execute pipeline
    let exported = pipeline.process(&text, font, &shaping_params, &render_params)?;
    let output_size = exported.len();

    // 11. Write output
    write_output(args, &exported)?;

    if !args.quiet {
        if let Some(ref path) = args.output_file {
            eprintln!("✓ Successfully rendered to {}", path.display());
        } else {
            eprintln!("✓ Successfully rendered to stdout");
        }
        eprintln!("  Format: {}", args.format.as_str().to_uppercase());
        eprintln!("  Size: {} bytes", output_size);
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ColorFontSupport {
    has_colr: bool,
    has_svg: bool,
    has_bitmap: bool,
}

impl ColorFontSupport {
    fn any(self) -> bool {
        self.has_colr || self.has_svg || self.has_bitmap
    }

    fn summary(self) -> String {
        let mut parts = Vec::new();
        if self.has_colr {
            parts.push("COLR");
        }
        if self.has_svg {
            parts.push("SVG");
        }
        if self.has_bitmap {
            parts.push("bitmap");
        }
        parts.join(", ")
    }
}

fn detect_color_font_support(font_data: &[u8]) -> Option<ColorFontSupport> {
    let font = skrifa::FontRef::new(font_data).ok()?;
    let has_colr = font.colr().is_ok();
    let has_svg = font.svg().is_ok();
    let has_bitmap = !BitmapStrikes::new(&font).is_empty();
    Some(ColorFontSupport {
        has_colr,
        has_svg,
        has_bitmap,
    })
}

fn warn_if_vello_gpu_with_color_font(renderer_name: &str, font_data: &[u8], quiet: bool) {
    if quiet || renderer_name != "vello" {
        return;
    }

    let Some(support) = detect_color_font_support(font_data) else {
        return;
    };
    if !support.any() {
        return;
    }

    eprintln!(
        "Warning: renderer 'vello' (GPU) has limited color-font support ({}). \
         If you see blank/missing glyphs, use `--renderer vello-cpu`.",
        support.summary()
    );
}

fn get_input_text(args: &RenderArgs) -> Result<String> {
    // Priority: text positional > --text > --text-file > stdin
    if let Some(ref text) = args.text {
        return Ok(decode_unicode_escapes(text));
    }

    if let Some(ref text) = args.text_arg {
        return Ok(decode_unicode_escapes(text));
    }

    if let Some(ref path) = args.text_file {
        let mut file = File::open(path)?;
        let mut text = String::new();
        file.read_to_string(&mut text)?;
        return Ok(text);
    }

    // Read from stdin
    let mut text = String::new();
    io::stdin().read_to_string(&mut text)?;
    Ok(text)
}

fn decode_unicode_escapes(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut result = String::with_capacity(text.len());
    let mut index = 0;

    while index < chars.len() {
        if chars[index] == '\\' && index + 1 < chars.len() && chars[index + 1] == 'u' {
            if let Some((decoded, consumed)) = parse_unicode_escape(&chars, index) {
                result.push(decoded);
                index += consumed;
                continue;
            }
        }

        result.push(chars[index]);
        index += 1;
    }

    result
}

fn parse_unicode_escape(chars: &[char], start: usize) -> Option<(char, usize)> {
    if start + 2 >= chars.len() {
        return None;
    }

    if chars[start + 2] == '{' {
        parse_braced_unicode_escape(chars, start)
    } else {
        parse_u4_unicode_escape(chars, start)
    }
}

fn parse_braced_unicode_escape(chars: &[char], start: usize) -> Option<(char, usize)> {
    let mut end = start + 3;
    while end < chars.len() && chars[end] != '}' {
        end += 1;
    }
    if end >= chars.len() {
        return None;
    }

    let digit_count = end - (start + 3);
    if !(1..=6).contains(&digit_count) {
        return None;
    }

    let mut value = 0u32;
    for ch in &chars[start + 3..end] {
        value = (value << 4) | ch.to_digit(16)?;
    }

    char::from_u32(value).map(|decoded| (decoded, end - start + 1))
}

fn parse_u4_unicode_escape(chars: &[char], start: usize) -> Option<(char, usize)> {
    let high = parse_u16_escape(chars, start)?;

    if (0xD800..=0xDBFF).contains(&high) {
        let next = start + 6;
        let low = parse_u16_escape(chars, next)?;
        if !(0xDC00..=0xDFFF).contains(&low) {
            return None;
        }
        let code = 0x1_0000 + ((((high as u32) - 0xD800) << 10) | ((low as u32) - 0xDC00));
        return char::from_u32(code).map(|decoded| (decoded, 12));
    }

    if (0xDC00..=0xDFFF).contains(&high) {
        return None;
    }

    char::from_u32(high as u32).map(|decoded| (decoded, 6))
}

fn parse_u16_escape(chars: &[char], start: usize) -> Option<u16> {
    if start + 6 > chars.len() {
        return None;
    }
    if chars[start] != '\\' || chars[start + 1] != 'u' {
        return None;
    }
    parse_hex_u16(&chars[start + 2..start + 6])
}

fn parse_hex_u16(chars: &[char]) -> Option<u16> {
    if chars.len() != 4 {
        return None;
    }
    let mut value = 0u16;
    for ch in chars {
        value = (value << 4) | (ch.to_digit(16)? as u16);
    }
    Some(value)
}

fn load_font(args: &RenderArgs) -> Result<Arc<dyn FontRef>> {
    let font_path = args.font_file.as_ref().ok_or_else(|| {
        TypfError::Other(
            "No font file specified. Use -f/--font-file <path> to provide a font.\n\
             Example: typf render -f /path/to/font.ttf 'Hello'"
                .into(),
        )
    })?;

    if args.verbose {
        eprintln!("Loading font from {}", font_path.display());
    }

    TypfFontFace::from_file(font_path).map(|f| Arc::new(f) as Arc<dyn FontRef>)
}

fn parse_font_size(size_str: &str) -> Result<f32> {
    let normalized = size_str.trim();

    let parsed: f32 = if normalized.eq_ignore_ascii_case("em") {
        1000.0 // UPM
    } else {
        normalized.parse().map_err(|_| {
            TypfError::Other(format!(
                "Invalid font size '{}': expected number or 'em'",
                normalized
            ))
        })?
    };

    if !parsed.is_finite() {
        return Err(TypfError::Other("Font size must be finite".into()));
    }
    if parsed <= 0.0 {
        return Err(TypfError::Other("Font size must be positive".into()));
    }
    if parsed > MAX_FONT_SIZE {
        return Err(TypfError::Other(format!(
            "Font size {} exceeds maximum {}",
            parsed, MAX_FONT_SIZE
        )));
    }

    Ok(parsed)
}

fn resolve_direction(
    text: &str,
    dir_str: &str,
    language: Option<&str>,
    script_hint: Option<&str>,
) -> Result<Direction> {
    let normalized = dir_str.trim();

    if normalized.eq_ignore_ascii_case("ltr") {
        Ok(Direction::LeftToRight)
    } else if normalized.eq_ignore_ascii_case("rtl") {
        Ok(Direction::RightToLeft)
    } else if normalized.eq_ignore_ascii_case("ttb") {
        Ok(Direction::TopToBottom)
    } else if normalized.eq_ignore_ascii_case("btt") {
        Ok(Direction::BottomToTop)
    } else if normalized.eq_ignore_ascii_case("auto") {
        let processor = UnicodeProcessor::new();
        let options = UnicodeOptions {
            detect_scripts: true,
            normalize: true,
            bidi_resolve: true,
            language: language.map(|l| l.to_string()),
        };

        let runs = processor.process(text, &options)?;

        let mut direction = runs
            .iter()
            .find_map(|run| {
                if matches!(
                    run.direction,
                    Direction::RightToLeft | Direction::BottomToTop
                ) {
                    Some(run.direction)
                } else {
                    None
                }
            })
            .unwrap_or(Direction::LeftToRight);

        if direction == Direction::LeftToRight {
            if let Some(script) = script_hint {
                if is_rtl_script(script) {
                    direction = Direction::RightToLeft;
                }
            }
        }

        Ok(direction)
    } else {
        Err(TypfError::Other(format!(
            "Invalid direction: {}",
            normalized
        )))
    }
}

fn script_hint(script: &str) -> Option<&str> {
    if script.eq_ignore_ascii_case("auto") {
        None
    } else {
        Some(script)
    }
}

fn is_rtl_script(script: &str) -> bool {
    matches!(
        script.to_ascii_lowercase().as_str(),
        "arab" | "hebr" | "syrc" | "thaa" | "nkoo" | "tfng" | "avst" | "phnx" | "armi"
    )
}

fn parse_color(color_str: &str) -> Result<Color> {
    let normalized = color_str.trim();
    let hex = normalized.strip_prefix('#').unwrap_or(normalized);

    let expanded;
    let hex = match hex.len() {
        3 | 4 => {
            expanded = hex.chars().flat_map(|ch| [ch, ch]).collect::<String>();
            expanded.as_str()
        },
        6 | 8 => hex,
        _ => {
            return Err(TypfError::Other(
                "Color must be in RGB, RGBA, RRGGBB, or RRGGBBAA hex format".into(),
            ))
        },
    };

    let (r, g, b, a) = if hex.len() == 6 {
        // RRGGBB format
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| TypfError::Other("Invalid color format".into()))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| TypfError::Other("Invalid color format".into()))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| TypfError::Other("Invalid color format".into()))?;
        (r, g, b, 255)
    } else if hex.len() == 8 {
        // RRGGBBAA format
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| TypfError::Other("Invalid color format".into()))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| TypfError::Other("Invalid color format".into()))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| TypfError::Other("Invalid color format".into()))?;
        let a = u8::from_str_radix(&hex[6..8], 16)
            .map_err(|_| TypfError::Other("Invalid color format".into()))?;
        (r, g, b, a)
    } else {
        return Err(TypfError::Other(
            "Color must be in RGB, RGBA, RRGGBB, or RRGGBBAA hex format".into(),
        ));
    };

    Ok(Color::rgba(r, g, b, a))
}

fn parse_features(features_str: &Option<String>) -> Result<Vec<(String, u32)>> {
    let Some(features) = features_str else {
        return Ok(Vec::new());
    };

    let mut result = Vec::new();
    for part in split_csv_whitespace(features) {
        let (tag, value) = parse_feature_token(part)?;
        if let Some(existing) = result.iter_mut().find(|(existing, _)| existing == &tag) {
            // Keep stable output ordering while making duplicate tags deterministic.
            existing.1 = value;
        } else {
            result.push((tag, value));
        }
    }

    Ok(result)
}

fn parse_feature_token(part: &str) -> Result<(String, u32)> {
    let (tag, value) = if let Some(stripped) = part.strip_prefix('+') {
        (stripped, 1)
    } else if let Some(stripped) = part.strip_prefix('-') {
        (stripped, 0)
    } else if let Some(pos) = part.find('=') {
        let tag = &part[..pos];
        let val = part[pos + 1..]
            .parse()
            .map_err(|_| TypfError::Other(format!("Invalid feature value: {}", part)))?;
        (tag, val)
    } else {
        (part, 1)
    };

    if !tag
        .as_bytes()
        .iter()
        .all(|byte| byte.is_ascii() && (0x20..=0x7E).contains(byte))
    {
        return Err(TypfError::Other(format!(
            "Invalid OpenType feature tag '{}': expected printable ASCII characters",
            tag
        )));
    }

    if tag.len() != 4 {
        return Err(TypfError::Other(format!(
            "Invalid OpenType feature tag '{}': expected exactly 4 characters",
            tag
        )));
    }

    Ok((tag.to_string(), value))
}

/// Parse variable font instance specification
///
/// Supports formats:
/// - "wght=700,wdth=100" - Axis values
/// - "wght:700 wdth:100" - Alternative separator
///
/// Named instances like "Bold" are not supported by this parser.
fn parse_variations(instance_str: &Option<String>) -> Result<Vec<(String, f32)>> {
    use std::collections::BTreeMap;

    let Some(instance) = instance_str else {
        return Ok(Vec::new());
    };

    let instance = instance.trim();
    if instance.is_empty() {
        return Ok(Vec::new());
    }

    let mut parsed = BTreeMap::new();

    for part in split_csv_whitespace(instance) {
        // Try axis=value or axis:value format
        let (tag, value) = if let Some(pos) = part.find('=') {
            (&part[..pos], &part[pos + 1..])
        } else if let Some(pos) = part.find(':') {
            (&part[..pos], &part[pos + 1..])
        } else {
            return Err(TypfError::Other(format!(
                "Invalid axis token '{}': expected axis=value or axis:value",
                part
            )));
        };

        let tag = tag.trim();
        if !tag
            .as_bytes()
            .iter()
            .all(|byte| (0x20..=0x7E).contains(byte))
        {
            return Err(TypfError::Other(format!(
                "Invalid axis tag '{}': expected printable ASCII characters",
                tag
            )));
        }

        if tag.len() != 4 {
            return Err(TypfError::Other(format!(
                "Invalid axis tag '{}': must be exactly 4 characters",
                tag
            )));
        }

        let val: f32 = value
            .trim()
            .parse()
            .map_err(|_| TypfError::Other(format!("Invalid axis value: {}", part)))?;

        if !val.is_finite() {
            return Err(TypfError::Other(format!(
                "Invalid axis value '{}': must be finite",
                part
            )));
        }

        // Last occurrence wins so repeated tags are deterministic.
        parsed.insert(tag.to_string(), val);
    }

    Ok(parsed.into_iter().collect())
}

/// Parse glyph-source preference/deny arguments
fn parse_glyph_sources(specs: &[String]) -> Result<GlyphSourcePreference> {
    if specs.is_empty() {
        return Ok(GlyphSourcePreference::default());
    }

    let mut prefer = Vec::new();
    let mut deny = Vec::new();

    for spec in specs {
        let (kind, list) = spec.split_once('=').ok_or_else(|| {
            TypfError::Other("glyph-source expects prefer= or deny=<list>".into())
        })?;

        let sources = parse_glyph_source_list(list)?;
        match kind.to_ascii_lowercase().as_str() {
            "prefer" => prefer.extend(sources),
            "deny" => deny.extend(sources),
            other => {
                return Err(TypfError::Other(format!(
                    "Invalid glyph-source flag '{}'; use prefer= or deny=",
                    other
                )))
            },
        }
    }

    Ok(GlyphSourcePreference::from_parts(prefer, deny))
}

fn parse_glyph_source_list(list: &str) -> Result<Vec<GlyphSource>> {
    if list.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut sources = Vec::new();
    for token in split_csv_whitespace(list) {
        sources.push(parse_glyph_source(token)?);
    }

    Ok(sources)
}

fn split_csv_whitespace(input: &str) -> impl Iterator<Item = &str> {
    input.split(',').flat_map(str::split_whitespace)
}

fn parse_glyph_source(token: &str) -> Result<GlyphSource> {
    match token.to_ascii_lowercase().as_str() {
        "glyf" | "ttf" => Ok(GlyphSource::Glyf),
        "cff" => Ok(GlyphSource::Cff),
        "cff2" => Ok(GlyphSource::Cff2),
        "colr" | "colr0" => Ok(GlyphSource::Colr0),
        "colr1" => Ok(GlyphSource::Colr1),
        "svg" => Ok(GlyphSource::Svg),
        "sbix" => Ok(GlyphSource::Sbix),
        "cbdt" => Ok(GlyphSource::Cbdt),
        "ebdt" => Ok(GlyphSource::Ebdt),
        other => Err(TypfError::Other(format!(
            "unknown glyph source '{}'; expected glyf,cff,cff2,colr0,colr1,svg,sbix,cbdt,ebdt",
            other
        ))),
    }
}

fn select_shaper(shaper_name: &str) -> Result<Arc<dyn Shaper + Send + Sync>> {
    match shaper_name {
        "auto" | "none" => Ok(Arc::new(NoneShaper::new())),

        #[cfg(feature = "shaping-hb")]
        "hb" | "harfbuzz" => Ok(Arc::new(typf_shape_hb::HarfBuzzShaper::new())),

        #[cfg(feature = "shaping-ct")]
        "ct" | "coretext" | "mac" => Ok(Arc::new(typf_shape_ct::CoreTextShaper::new())),

        #[cfg(feature = "shaping-icu-hb")]
        "icu-hb" | "icu-harfbuzz" => Ok(Arc::new(typf_shape_icu_hb::IcuHarfBuzzShaper::new())),

        _ => Err(TypfError::Other(format!(
            "Unknown or unavailable shaper: {}",
            shaper_name
        ))),
    }
}

fn select_renderer(renderer_name: &str) -> Result<Arc<dyn Renderer + Send + Sync>> {
    match renderer_name {
        "auto" | "opixa" => Ok(Arc::new(OpixaRenderer::new())),

        "svg" => Ok(Arc::new(SvgRenderer::new())),

        #[cfg(feature = "render-cg")]
        "cg" | "coregraphics" | "mac" => Ok(Arc::new(typf_render_cg::CoreGraphicsRenderer::new())),

        #[cfg(feature = "render-skia")]
        "skia" => Ok(Arc::new(typf_render_skia::SkiaRenderer::new())),

        #[cfg(feature = "render-zeno")]
        "zeno" => Ok(Arc::new(typf_render_zeno::ZenoRenderer::new())),

        #[cfg(feature = "render-vello-cpu")]
        "vello-cpu" => Ok(Arc::new(typf_render_vello_cpu::VelloCpuRenderer::new())),

        #[cfg(feature = "render-vello")]
        "vello" => typf_render_vello::VelloRenderer::new()
            .map(|r| Arc::new(r) as Arc<dyn Renderer + Send + Sync>)
            .map_err(|e| TypfError::Other(format!("Failed to create GPU renderer: {}", e))),

        _ => Err(TypfError::Other(format!(
            "Unknown or unavailable renderer: {}",
            renderer_name
        ))),
    }
}

/// Check if the renderer name refers to a linra (single-pass) renderer
fn is_linra_renderer(renderer_name: &str) -> bool {
    matches!(
        renderer_name,
        "linra" | "linra-mac" | "linra-win" | "linra-os"
    )
}

/// Select and create a linra renderer based on name
#[cfg(feature = "linra")]
fn select_linra_renderer(renderer_name: &str) -> Result<Arc<dyn LinraRenderer>> {
    match renderer_name {
        #[cfg(feature = "linra-mac")]
        "auto" | "linra" | "linra-mac" | "linra-os" => {
            Ok(Arc::new(typf_os_mac::CoreTextLinraRenderer::new()))
        },

        #[cfg(all(feature = "linra-win", target_os = "windows"))]
        "auto" | "linra" | "linra-win" | "linra-os" => typf_os_win::DirectWriteLinraRenderer::new()
            .map(|r| Arc::new(r) as Arc<dyn LinraRenderer>),

        _ => Err(TypfError::Other(format!(
            "Unknown or unavailable linra renderer: {}. \
             Available: linra-mac (macOS), linra-win (Windows)",
            renderer_name
        ))),
    }
}

/// Render using the linra (single-pass) pipeline
#[cfg(feature = "linra")]
fn run_linra(args: &RenderArgs) -> Result<()> {
    // 1. Get input text
    let text = get_input_text(args)?;

    if !args.quiet {
        eprintln!("Typf v{} (linra mode)", env!("CARGO_PKG_VERSION"));
        eprintln!("Rendering text with single-pass pipeline...");
    }

    // 2. Load font
    let font: Arc<dyn FontRef> = load_font(args)?;

    // 3. Parse rendering parameters
    let font_size = parse_font_size(&args.font_size)?;
    let direction = resolve_direction(
        &text,
        &args.direction,
        args.language.as_deref(),
        script_hint(&args.script),
    )?;
    let foreground = parse_color(&args.foreground)?;
    let background = parse_color(&args.background)?;

    // 4. Parse variable font variations
    let variations = parse_variations(&args.instance)?;

    // 5. Create linra parameters (combines shaping + rendering params)
    let linra_params = LinraRenderParams {
        size: font_size,
        direction,
        foreground,
        background: Some(background),
        padding: args.margin,
        variations,
        features: parse_features(&args.features)?,
        language: args.language.clone(),
        script: if args.script == "auto" {
            None
        } else {
            Some(args.script.clone())
        },
        antialias: !matches!(args.format, OutputFormat::Pbm | OutputFormat::Png1),
        letter_spacing: 0.0,
        color_palette: args.color_palette as u16,
    };

    // 5. Select linra renderer
    let linra_renderer = select_linra_renderer(&args.renderer)?;

    if args.verbose {
        eprintln!("Using linra renderer: {}", linra_renderer.name());
    }

    // 6. Render text (single-pass: shaping + rendering combined)
    // Note: SVG is handled before calling run_linra() - we fall back to traditional pipeline
    if args.verbose {
        eprintln!("Rendering with linra backend...");
    }
    let rendered = linra_renderer.render_text(&text, font, &linra_params)?;

    // 8. Export to requested format
    if args.verbose {
        eprintln!("Exporting to {} format...", args.format.as_str());
    }
    let exporter = create_exporter(args.format)?;
    let exported = exporter.export(&rendered)?;

    // 9. Write output
    write_output(args, &exported)?;

    if !args.quiet {
        if let Some(ref path) = args.output_file {
            eprintln!("✓ Successfully rendered to {}", path.display());
        } else {
            eprintln!("✓ Successfully rendered to stdout");
        }
        eprintln!("  Format: {}", args.format.as_str().to_uppercase());
        eprintln!("  Size: {} bytes", exported.len());
        eprintln!("  Mode: linra (single-pass)");
    }

    Ok(())
}

fn create_exporter(format: OutputFormat) -> Result<Arc<dyn Exporter>> {
    match format {
        OutputFormat::Ppm => Ok(Arc::new(PnmExporter::ppm())),
        OutputFormat::Pgm => Ok(Arc::new(PnmExporter::pgm())),
        OutputFormat::Pbm => Ok(Arc::new(PnmExporter::new(typf_export::PnmFormat::Pbm))),
        OutputFormat::Png | OutputFormat::Png8 => Ok(Arc::new(PngExporter::new())),
        OutputFormat::Png1 | OutputFormat::Png4 => {
            // PNG1 and PNG4 not yet supported - fall back to PNG8
            Ok(Arc::new(PngExporter::new()))
        },
        OutputFormat::Svg => Ok(Arc::new(SvgOutputExporter)),
    }
}

fn cache_flags(
    use_linra: bool,
    svg_fallback: bool,
    no_shaping_cache: bool,
    no_glyph_cache: bool,
) -> (bool, bool, bool) {
    let allowed = !use_linra || svg_fallback;
    let shaping = allowed && !no_shaping_cache;
    let glyph = allowed && !no_glyph_cache;
    (shaping, glyph, allowed)
}

struct SvgOutputExporter;

impl Exporter for SvgOutputExporter {
    fn name(&self) -> &'static str {
        "SvgExporter"
    }

    fn export(&self, output: &RenderOutput) -> Result<Vec<u8>> {
        match output {
            RenderOutput::Vector(vector) if vector.format == VectorFormat::Svg => {
                Ok(vector.data.as_bytes().to_vec())
            },
            RenderOutput::Vector(_) => Err(TypfError::Other(
                "SVG exporter received non-SVG vector data".into(),
            )),
            _ => Err(TypfError::Other(
                "SVG output requires a vector renderer".into(),
            )),
        }
    }

    fn extension(&self) -> &'static str {
        "svg"
    }

    fn mime_type(&self) -> &'static str {
        "image/svg+xml"
    }
}

#[cfg(test)]
mod cache_flag_tests {
    use super::cache_flags;

    #[test]
    fn caches_enabled_by_default() {
        let (shape, glyph, allowed) = cache_flags(false, false, false, false);
        assert!(allowed);
        assert!(shape);
        assert!(glyph);
    }

    #[test]
    fn linra_disables_caches() {
        let (shape, glyph, allowed) = cache_flags(true, false, false, false);
        assert!(!allowed);
        assert!(!shape);
        assert!(!glyph);
    }

    #[test]
    fn linra_svg_fallback_keeps_caches() {
        let (shape, glyph, allowed) = cache_flags(true, true, false, false);
        assert!(allowed);
        assert!(shape);
        assert!(glyph);
    }

    #[test]
    fn flags_turn_off_individually() {
        let (shape, glyph, allowed) = cache_flags(false, false, true, false);
        assert!(allowed);
        assert!(!shape);
        assert!(glyph);

        let (shape2, glyph2, allowed2) = cache_flags(false, false, false, true);
        assert!(allowed2);
        assert!(shape2);
        assert!(!glyph2);
    }
}

fn write_output(args: &RenderArgs, data: &[u8]) -> Result<()> {
    if let Some(ref path) = args.output_file {
        let mut file = File::create(path)?;
        file.write_all(data)?;
    } else {
        io::stdout().write_all(data)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_font(name: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop(); // crates
        path.pop(); // root
        path.push("test-fonts");
        path.push(name);
        path
    }

    fn load_font_bytes(name: &str) -> Vec<u8> {
        std::fs::read(test_font(name)).expect("test font should be readable")
    }

    #[test]
    fn test_detect_color_font_support_when_no_color_tables_then_all_false() {
        let bytes = load_font_bytes("NotoSans-Regular.ttf");
        let support = detect_color_font_support(&bytes).expect("font parse should succeed");
        assert_eq!(
            support,
            ColorFontSupport {
                has_colr: false,
                has_svg: false,
                has_bitmap: false
            },
            "expected no color tables for NotoSans-Regular.ttf"
        );
    }

    #[test]
    fn test_detect_color_font_support_when_colr_font_then_colr_true() {
        let bytes = load_font_bytes("Nabla-Regular-COLR.ttf");
        let support = detect_color_font_support(&bytes).expect("font parse should succeed");
        assert!(
            support.has_colr,
            "expected COLR support for Nabla-Regular-COLR.ttf"
        );
    }

    #[test]
    fn test_detect_color_font_support_when_svg_font_then_svg_true() {
        let bytes = load_font_bytes("Nabla-Regular-SVG.ttf");
        let support = detect_color_font_support(&bytes).expect("font parse should succeed");
        assert!(
            support.has_svg,
            "expected SVG table for Nabla-Regular-SVG.ttf"
        );
    }

    #[test]
    fn test_detect_color_font_support_when_bitmap_font_then_bitmap_true() {
        let bytes = load_font_bytes("Nabla-Regular-CBDT.ttf");
        let support = detect_color_font_support(&bytes).expect("font parse should succeed");
        assert!(
            support.has_bitmap,
            "expected bitmap strikes for Nabla-Regular-CBDT.ttf"
        );
    }

    #[test]
    fn direction_auto_detects_rtl_text() {
        let text = "مرحبا"; // Arabic should resolve to RTL via bidi runs
        let direction = resolve_direction(text, "auto", Some("ar"), None)
            .expect("direction detection should succeed");
        assert_eq!(direction, Direction::RightToLeft);
    }

    #[test]
    fn direction_auto_uses_script_hint_when_neutral() {
        let text = "1234"; // Neutral text, relies on script hint
        let direction = resolve_direction(text, "auto", None, Some("Arab"))
            .expect("direction detection should succeed");
        assert_eq!(direction, Direction::RightToLeft);
    }

    #[test]
    fn direction_errors_on_invalid_value() {
        let err = resolve_direction("text", "sideways", None, None)
            .expect_err("invalid direction should error");
        assert!(format!("{err}").contains("Invalid direction"));
    }

    #[test]
    fn direction_accepts_trimmed_case_insensitive_tokens() {
        let direction = resolve_direction("text", "  RTL\t", None, None)
            .expect("trimmed uppercase direction should parse");
        assert_eq!(direction, Direction::RightToLeft);
    }

    #[test]
    fn glyph_source_parsing_applies_prefer_and_deny() {
        let specs = vec!["prefer=glyf,svg".to_string(), "deny=svg,colr1".to_string()];

        let pref = parse_glyph_sources(&specs).expect("parsing should succeed");
        assert_eq!(pref.prefer, vec![GlyphSource::Glyf]);
        assert!(pref.deny.contains(&GlyphSource::Svg));
        assert!(pref.deny.contains(&GlyphSource::Colr1));
    }

    #[test]
    fn glyph_source_parsing_defaults_when_empty() {
        let pref = parse_glyph_sources(&[]).expect("empty specs should use default");
        assert_eq!(pref, GlyphSourcePreference::default());
    }

    #[test]
    fn glyph_source_parsing_errors_on_unknown_source() {
        let err = parse_glyph_sources(&["prefer=unknown".to_string()])
            .expect_err("unknown source should error");
        assert!(
            format!("{err}").contains("unknown glyph source"),
            "error message should mention source"
        );
    }

    #[test]
    fn parse_features_accepts_mixed_syntax() {
        let parsed = parse_features(&Some("+liga kern=0 smcp cv01=2".to_string()))
            .expect("feature parsing should succeed");
        assert_eq!(
            parsed,
            vec![
                ("liga".to_string(), 1),
                ("kern".to_string(), 0),
                ("smcp".to_string(), 1),
                ("cv01".to_string(), 2)
            ]
        );
    }

    #[test]
    fn parse_features_when_duplicate_tags_then_last_value_wins() {
        let parsed = parse_features(&Some("+liga kern=0 liga=1 cv01=1 cv01=3".to_string()))
            .expect("duplicate feature tags should parse deterministically");
        assert_eq!(
            parsed,
            vec![
                ("liga".to_string(), 1),
                ("kern".to_string(), 0),
                ("cv01".to_string(), 3)
            ]
        );
    }

    #[test]
    fn parse_features_rejects_tag_with_invalid_length() {
        let err = parse_features(&Some("ligature=1".to_string()))
            .expect_err("non-4-char tags should fail");
        assert!(
            format!("{err}").contains("expected exactly 4 characters"),
            "error should mention tag length"
        );
    }

    #[test]
    fn parse_features_rejects_non_ascii_tag() {
        let err = parse_features(&Some("liga=1,éab=1".to_string()))
            .expect_err("non-ascii tags should fail");
        assert!(
            format!("{err}").contains("printable ASCII"),
            "error should mention ASCII validation"
        );
    }

    #[test]
    fn parse_features_rejects_non_ascii_multibyte_tag() {
        let err = parse_features(&Some("éght=1".to_string()))
            .expect_err("multibyte non-ascii tags should fail");
        assert!(
            format!("{err}").contains("printable ASCII"),
            "error should mention ASCII validation"
        );
    }

    #[test]
    fn parse_features_accepts_tab_and_newline_separators() {
        let parsed = parse_features(&Some("+liga,\tkern=0\nsmcp".to_string()))
            .expect("tab/newline-delimited features should parse");
        assert_eq!(
            parsed,
            vec![
                ("liga".to_string(), 1),
                ("kern".to_string(), 0),
                ("smcp".to_string(), 1)
            ]
        );
    }

    #[test]
    fn parse_variations_accepts_tab_and_newline_separators() {
        let parsed = parse_variations(&Some("wght=700,\nwdth:95\topsz=14".to_string()))
            .expect("tab/newline-delimited variations should parse");
        assert_eq!(
            parsed,
            vec![
                ("opsz".to_string(), 14.0),
                ("wdth".to_string(), 95.0),
                ("wght".to_string(), 700.0)
            ]
        );
    }

    #[test]
    fn parse_variations_rejects_named_instance_token() {
        let err =
            parse_variations(&Some("Bold".to_string())).expect_err("named instances are rejected");
        assert!(
            format!("{err}").contains("expected axis=value or axis:value"),
            "error should explain supported axis syntax"
        );
    }

    #[test]
    fn parse_variations_rejects_non_ascii_tag() {
        let err = parse_variations(&Some("\u{7f}ght=700".to_string()))
            .expect_err("non-ascii variation axis tags should fail");
        assert!(
            format!("{err}").contains("printable ASCII"),
            "error should mention ASCII validation"
        );
    }

    #[test]
    fn parse_variations_when_duplicate_axis_then_last_value_wins() {
        let parsed = parse_variations(&Some("wght=400,wdth=90,wght=700".to_string()))
            .expect("duplicate axis tags should parse deterministically");
        assert_eq!(
            parsed,
            vec![("wdth".to_string(), 90.0), ("wght".to_string(), 700.0)]
        );
    }

    #[test]
    fn glyph_source_parsing_accepts_tab_and_newline_separators() {
        let pref = parse_glyph_sources(&["prefer=glyf,\nsvg\tcff2".to_string()])
            .expect("tab/newline-delimited sources should parse");
        assert_eq!(
            pref.prefer,
            vec![GlyphSource::Glyf, GlyphSource::Svg, GlyphSource::Cff2]
        );
    }

    #[test]
    fn parse_font_size_accepts_em_keyword() {
        let parsed = parse_font_size("em").expect("em keyword should parse");
        assert_eq!(parsed, 1000.0);
    }

    #[test]
    fn parse_font_size_accepts_trimmed_case_insensitive_em_keyword() {
        let parsed = parse_font_size("  EM\t").expect("trimmed uppercase em keyword should parse");
        assert_eq!(parsed, 1000.0);
    }

    #[test]
    fn parse_font_size_rejects_non_finite_values() {
        let err = parse_font_size("NaN").expect_err("NaN should be rejected");
        assert!(
            format!("{err}").contains("finite"),
            "expected finite-size validation message, got: {}",
            err
        );
    }

    #[test]
    fn parse_font_size_rejects_non_numeric_values_with_context() {
        let err = parse_font_size("large").expect_err("non-numeric size should fail");
        assert!(
            format!("{err}").contains("Invalid font size 'large'"),
            "expected contextual parse error, got: {}",
            err
        );
    }

    #[test]
    fn parse_font_size_rejects_non_positive_values() {
        let zero_err = parse_font_size("0").expect_err("zero size should be rejected");
        let negative_err = parse_font_size("-12").expect_err("negative size should be rejected");
        assert!(
            format!("{zero_err}").contains("positive"),
            "expected positive-size validation message, got: {}",
            zero_err
        );
        assert!(
            format!("{negative_err}").contains("positive"),
            "expected positive-size validation message, got: {}",
            negative_err
        );
    }

    #[test]
    fn parse_font_size_rejects_values_above_maximum() {
        let oversized = format!("{}", MAX_FONT_SIZE + 1.0);
        let err = parse_font_size(&oversized).expect_err("oversized font should be rejected");
        assert!(
            format!("{err}").contains("exceeds maximum"),
            "expected maximum-size validation message, got: {}",
            err
        );
    }

    #[test]
    fn parse_color_accepts_trimmed_six_digit_hex() {
        let color = parse_color("  #00ff7f\t").expect("trimmed six-digit hex should parse");
        assert_eq!(color, Color::rgba(0x00, 0xFF, 0x7F, 0xFF));
    }

    #[test]
    fn parse_color_accepts_rgb_shorthand_hex() {
        let color = parse_color("#0f8").expect("rgb shorthand should parse");
        assert_eq!(color, Color::rgba(0x00, 0xFF, 0x88, 0xFF));
    }

    #[test]
    fn parse_color_accepts_rgba_shorthand_hex() {
        let color = parse_color("0f8c").expect("rgba shorthand should parse");
        assert_eq!(color, Color::rgba(0x00, 0xFF, 0x88, 0xCC));
    }

    #[test]
    fn parse_color_rejects_invalid_hex_length() {
        let err = parse_color("#12").expect_err("invalid-length hex should fail");
        assert!(
            format!("{err}").contains("RGB, RGBA, RRGGBB, or RRGGBBAA"),
            "expected supported-format guidance, got: {}",
            err
        );
    }

    #[test]
    fn decode_unicode_escapes_decodes_basic_u4_sequence() {
        assert_eq!(decode_unicode_escapes(r"\u0041"), "A");
    }

    #[test]
    fn decode_unicode_escapes_decodes_braced_sequence() {
        assert_eq!(decode_unicode_escapes(r"\u{1F600}"), "😀");
    }

    #[test]
    fn decode_unicode_escapes_decodes_surrogate_pair_sequence() {
        assert_eq!(decode_unicode_escapes(r"\uD83D\uDE00"), "😀");
    }

    #[test]
    fn decode_unicode_escapes_preserves_malformed_sequences() {
        assert_eq!(decode_unicode_escapes(r"\u12"), r"\u12");
        assert_eq!(decode_unicode_escapes(r"\u{xyz}"), r"\u{xyz}");
        assert_eq!(decode_unicode_escapes(r"\uD83D"), r"\uD83D");
    }
}
