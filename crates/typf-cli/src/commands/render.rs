///! Render command implementation
///!
///! Handles text rendering with full pipeline control.

use crate::cli::{OutputFormat, RenderArgs};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use typf::error::{Result, TypfError};
use typf_core::{
    traits::{FontRef, Renderer, Shaper},
    types::Direction,
    Color, RenderParams, ShapingParams,
};
use typf_export::PnmExporter;
use typf_export_svg::SvgExporter;
use typf_fontdb::Font;
use typf_render_orge::OrgeRenderer;
use typf_shape_none::NoneShaper;

/// Minimal stub font for demonstration
struct StubFont {
    units_per_em: u16,
}

impl StubFont {
    fn new() -> Self {
        Self { units_per_em: 1000 }
    }
}

impl FontRef for StubFont {
    fn data(&self) -> &[u8] {
        &[]
    }

    fn units_per_em(&self) -> u16 {
        self.units_per_em
    }

    fn glyph_id(&self, ch: char) -> Option<u32> {
        if ch.is_ascii() {
            Some(ch as u32)
        } else {
            Some(0)
        }
    }

    fn advance_width(&self, _glyph_id: u32) -> f32 {
        600.0
    }
}

pub fn run(args: &RenderArgs) -> Result<()> {
    // 1. Get input text
    let text = get_input_text(args)?;

    if !args.quiet {
        eprintln!("TYPF v{}", env!("CARGO_PKG_VERSION"));
        eprintln!("Rendering text...");
    }

    // 2. Load font
    let font: Arc<dyn FontRef> = load_font(args)?;

    // 3. Parse rendering parameters
    let font_size = parse_font_size(&args.font_size)?;
    let direction = parse_direction(&args.direction)?;
    let foreground = parse_color(&args.foreground)?;
    let background = parse_color(&args.background)?;

    // 4. Create shaping parameters
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
        variations: Vec::new(), // TODO: Parse from instance
        letter_spacing: 0.0,
    };

    // 5. Create rendering parameters
    let render_params = RenderParams {
        foreground,
        background: Some(background),
        padding: args.margin,
        antialias: !matches!(args.format, OutputFormat::Pbm | OutputFormat::Png1),
        variations: Vec::new(),
    };

    // 6. Select backends
    let shaper = select_shaper(&args.shaper)?;
    let renderer = select_renderer(&args.renderer)?;

    // 7. Shape text
    if args.verbose {
        eprintln!("Shaping with {} backend...", args.shaper);
    }
    let shaped = shaper.shape(&text, font.clone(), &shaping_params)?;

    if args.verbose {
        eprintln!("Shaped {} glyphs", shaped.glyphs.len());
    }

    // 8. Handle SVG export separately
    if matches!(args.format, OutputFormat::Svg) {
        return export_svg(args, &shaped, font, foreground);
    }

    // 9. Render to bitmap
    if args.verbose {
        eprintln!("Rendering with {} backend...", args.renderer);
    }
    let rendered = renderer.render(&shaped, font, &render_params)?;

    // 10. Export to requested format
    if args.verbose {
        eprintln!("Exporting to {} format...", args.format.as_str());
    }
    let exporter = create_exporter(args.format)?;
    let exported = exporter.export(&rendered)?;

    // 11. Write output
    write_output(args, &exported)?;

    if !args.quiet {
        if let Some(ref path) = args.output_file {
            eprintln!("✓ Successfully rendered to {}", path.display());
        } else {
            eprintln!("✓ Successfully rendered to stdout");
        }
        eprintln!("  Format: {}", args.format.as_str().to_uppercase());
        eprintln!("  Size: {} bytes", exported.len());
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
            match chars.peek() {
                Some('u') => {
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
                _ => {}
            }
        }
        result.push(ch);
    }

    result
}

fn load_font(args: &RenderArgs) -> Result<Arc<dyn FontRef>> {
    if let Some(ref font_path) = args.font_file {
        if args.verbose {
            eprintln!("Loading font from {}", font_path.display());
        }
        Ok(Arc::new(Font::from_file(font_path)?))
    } else {
        if args.verbose {
            eprintln!("Using stub font (no font file provided)");
        }
        Ok(Arc::new(StubFont::new()))
    }
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

fn parse_direction(dir_str: &str) -> Result<Direction> {
    match dir_str {
        "auto" | "ltr" => Ok(Direction::LeftToRight),
        "rtl" => Ok(Direction::RightToLeft),
        "ttb" => Ok(Direction::TopToBottom),
        "btt" => Ok(Direction::BottomToTop),
        _ => Err(TypfError::Other(format!("Invalid direction: {}", dir_str))),
    }
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
    for part in features.split(|c| c == ',' || c == ' ') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let (tag, value) = if part.starts_with('+') {
            (&part[1..], 1)
        } else if part.starts_with('-') {
            (&part[1..], 0)
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

fn select_shaper(shaper_name: &str) -> Result<Arc<dyn Shaper + Send + Sync>> {
    match shaper_name {
        "auto" | "none" => Ok(Arc::new(NoneShaper::new())),

        #[cfg(feature = "shaping-hb")]
        "hb" | "harfbuzz" => Ok(Arc::new(typf_shape_hb::HarfBuzzShaper::new())),

        _ => Err(TypfError::Other(format!(
            "Unknown or unavailable shaper: {}",
            shaper_name
        ))),
    }
}

fn select_renderer(renderer_name: &str) -> Result<Arc<dyn Renderer + Send + Sync>> {
    match renderer_name {
        "auto" | "orge" => Ok(Arc::new(OrgeRenderer::new())),

        _ => Err(TypfError::Other(format!(
            "Unknown or unavailable renderer: {}",
            renderer_name
        ))),
    }
}

fn create_exporter(format: OutputFormat) -> Result<Arc<dyn typf_core::traits::Exporter>> {
    match format {
        OutputFormat::Ppm => Ok(Arc::new(PnmExporter::ppm())),
        OutputFormat::Pgm => Ok(Arc::new(PnmExporter::pgm())),
        OutputFormat::Pbm => Ok(Arc::new(PnmExporter::new(typf_export::PnmFormat::Pbm))),
        OutputFormat::Png
        | OutputFormat::Png1
        | OutputFormat::Png4
        | OutputFormat::Png8 => {
            // For now, use PGM as placeholder
            // TODO: Implement PNG exporter
            Ok(Arc::new(PnmExporter::pgm()))
        }
        OutputFormat::Svg => Err(TypfError::Other(
            "SVG export handled separately".into(),
        )),
    }
}

fn export_svg(
    args: &RenderArgs,
    shaped: &typf_core::types::ShapingResult,
    font: Arc<dyn FontRef>,
    foreground: Color,
) -> Result<()> {
    if font.data().is_empty() {
        return Err(TypfError::Other(
            "SVG export requires a real font file (stub font not supported)".into(),
        ));
    }

    let svg_exporter = SvgExporter::new();
    let svg_data = svg_exporter.export(shaped, font, foreground)?;

    if let Some(ref path) = args.output_file {
        let mut file = File::create(path)?;
        file.write_all(svg_data.as_bytes())?;
    } else {
        io::stdout().write_all(svg_data.as_bytes())?;
    }

    if !args.quiet {
        if let Some(ref path) = args.output_file {
            eprintln!("✓ Successfully exported to {}", path.display());
        } else {
            eprintln!("✓ Successfully exported to stdout");
        }
        eprintln!("  Format: SVG (vector)");
        eprintln!("  Glyphs: {}", shaped.glyphs.len());
    }

    Ok(())
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
