//! Render command implementation
//!
//! Handles text rendering with full pipeline control.
//! Supports both traditional (shaper+renderer) and linra (single-pass) modes.

use crate::cli::{OutputFormat, RenderArgs};
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
};
use typf_export::{PngExporter, PnmExporter};
use typf_fontdb::Font;
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

    // 9. Build pipeline
    let exporter = create_exporter(args.format)?;
    let pipeline = Pipeline::builder()
        .shaper(shaper.clone())
        .renderer(renderer.clone())
        .exporter(exporter.clone())
        .build()?;

    if args.verbose {
        eprintln!("Shaping with {} backend...", shaper_name);
        eprintln!("Rendering with {} backend...", renderer_name);
        eprintln!("Exporting to {} format...", args.format.as_str());
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
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some('u') = chars.peek() {
                chars.next(); // consume 'u'
                if chars.peek() == Some(&'{') {
                    // \u{X...} format
                    chars.next(); // consume '{'
                    let mut hex = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == '}' {
                            chars.next(); // consume '}'
                            break;
                        }
                        hex.push(c);
                        chars.next();
                    }
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(unicode_char) = char::from_u32(code) {
                            result.push(unicode_char);
                            continue;
                        }
                    }
                } else {
                    // \uXXXX format (exactly 4 hex digits)
                    let mut hex = String::new();
                    for _ in 0..4 {
                        if let Some(c) = chars.next() {
                            hex.push(c);
                        }
                    }
                    if let Ok(code) = u16::from_str_radix(&hex, 16) {
                        result.push(char::from_u32(code as u32).unwrap_or('�'));
                        continue;
                    }
                }
            }
        }
        result.push(ch);
    }

    result
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

    Font::from_file(font_path).map(|f| Arc::new(f) as Arc<dyn FontRef>)
}

fn parse_font_size(size_str: &str) -> Result<f32> {
    if size_str == "em" {
        Ok(1000.0) // UPM
    } else {
        size_str
            .parse()
            .map_err(|_| TypfError::Other("Invalid font size".into()))
    }
}

fn resolve_direction(
    text: &str,
    dir_str: &str,
    language: Option<&str>,
    script_hint: Option<&str>,
) -> Result<Direction> {
    match dir_str {
        "ltr" => Ok(Direction::LeftToRight),
        "rtl" => Ok(Direction::RightToLeft),
        "ttb" => Ok(Direction::TopToBottom),
        "btt" => Ok(Direction::BottomToTop),
        "auto" => {
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
        },
        _ => Err(TypfError::Other(format!("Invalid direction: {}", dir_str))),
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
    let hex = color_str.trim_start_matches('#');

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
            "Color must be in RRGGBB or RRGGBBAA format".into(),
        ));
    };

    Ok(Color::rgba(r, g, b, a))
}

fn parse_features(features_str: &Option<String>) -> Result<Vec<(String, u32)>> {
    let Some(features) = features_str else {
        return Ok(Vec::new());
    };

    let mut result = Vec::new();
    for part in features.split([',', ' ']) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

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

        result.push((tag.to_string(), value));
    }

    Ok(result)
}

/// Parse variable font instance specification
///
/// Supports formats:
/// - "wght=700,wdth=100" - Axis values
/// - "wght:700 wdth:100" - Alternative separator
/// - "Bold" - Named instance (not yet supported, returns empty)
fn parse_variations(instance_str: &Option<String>) -> Result<Vec<(String, f32)>> {
    let Some(instance) = instance_str else {
        return Ok(Vec::new());
    };

    let instance = instance.trim();
    if instance.is_empty() {
        return Ok(Vec::new());
    }

    let mut result = Vec::new();

    // Split by comma or space
    for part in instance.split([',', ' ']) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // Try axis=value or axis:value format
        let (tag, value) = if let Some(pos) = part.find('=') {
            (&part[..pos], &part[pos + 1..])
        } else if let Some(pos) = part.find(':') {
            (&part[..pos], &part[pos + 1..])
        } else {
            // Named instance - skip for now (would need font parsing)
            continue;
        };

        let tag = tag.trim();
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

        result.push((tag.to_string(), val));
    }

    Ok(result)
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
    for token in list.split([',', ' ']) {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        sources.push(parse_glyph_source(token)?);
    }

    Ok(sources)
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
}
