//! TYPF meets the terminal: Fast text rendering from the command line
//!
//! Render text to images, process batches, or explore interactively.
//! Perfect for scripts, testing, and quick typography experiments.

mod batch;
mod jsonl;
mod repl;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use typf::error::{Result, TypfError};
use typf_core::{
    traits::{Exporter, FontRef, Renderer, Shaper},
    types::{Direction, RenderOutput},
    Color, RenderParams, ShapingParams,
};
use typf_export::PnmExporter;
use typf_export_svg::SvgExporter;
use typf_fontdb::Font;
use typf_render_orge::OrgeRenderer;
use typf_shape_none::NoneShaper;

/// Command-line arguments for single text rendering
#[derive(Debug)]
struct Args {
    /// What we're rendering
    text: String,
    /// Where the output goes
    output: PathBuf,
    /// Font file (optional - uses stub if not provided)
    font: Option<PathBuf>,
    /// Which shaper to use (future: none, harfbuzz)
    _shaper: String,
    /// Which renderer to use (future: orge, skia)
    _renderer: String,
    /// How big the text should be
    size: f32,
    /// What format we're outputting
    format: String,
}

impl Args {
    fn parse() -> Result<Self> {
        let args: Vec<String> = std::env::args().collect();

        if args.len() < 2 {
            eprintln!("Usage: {} <text|--repl|--batch> [options]", args[0]);
            eprintln!();
            eprintln!("Single Mode Options:");
            eprintln!("  --font <file>         Font file path (required for real fonts)");
            eprintln!("  --shaper <backend>    Shaping backend: none, harfbuzz (default: none)");
            eprintln!("  --renderer <backend>  Rendering backend: orge (default: orge)");
            eprintln!("  --output <file>       Output file (default: output.ppm)");
            eprintln!("  --size <size>         Font size in points (default: 16)");
            eprintln!("  --format <fmt>        Output format: ppm, pgm, pbm, svg (default: ppm)");
            eprintln!();
            eprintln!("Batch Mode Options:");
            eprintln!("  --batch-input <file>  Input file with one text per line (default: stdin)");
            eprintln!("  --batch-output <dir>  Output directory (default: current)");
            eprintln!("  --batch-pattern <p>   Output filename pattern with {{}} (default: output_{{}}.ppm)");
            eprintln!("  --quiet, -q           Suppress progress output");
            eprintln!();
            eprintln!("Other Modes:");
            eprintln!("  --repl, -i            Start interactive REPL mode");
            eprintln!();
            eprintln!("Examples:");
            eprintln!("  {} \"Hello\" --font font.ttf --output hello.png", args[0]);
            eprintln!(
                "  {} \"Hello\" --font font.ttf --shaper harfbuzz --format svg -o out.svg",
                args[0]
            );
            eprintln!("  {} --batch-input lines.txt --batch-output out/ --size 20", args[0]);
            eprintln!("  {} --repl", args[0]);
            std::process::exit(1);
        }

        let text = args[1].clone();
        let mut font: Option<PathBuf> = None;
        let mut _shaper = "none".to_string();
        let mut _renderer = "orge".to_string();
        let mut output = PathBuf::from("output.ppm");
        let mut size = 16.0;
        let mut format = "ppm".to_string();

        let mut i = 2;
        while i < args.len() {
            match args[i].as_str() {
                "--font" => {
                    if i + 1 < args.len() {
                        font = Some(PathBuf::from(&args[i + 1]));
                        i += 2;
                    } else {
                        return Err(TypfError::Other("--font requires an argument".into()));
                    }
                },
                "--shaper" => {
                    if i + 1 < args.len() {
                        _shaper = args[i + 1].clone();
                        i += 2;
                    } else {
                        return Err(TypfError::Other("--shaper requires an argument".into()));
                    }
                },
                "--renderer" => {
                    if i + 1 < args.len() {
                        _renderer = args[i + 1].clone();
                        i += 2;
                    } else {
                        return Err(TypfError::Other("--renderer requires an argument".into()));
                    }
                },
                "--output" | "-o" => {
                    if i + 1 < args.len() {
                        output = PathBuf::from(&args[i + 1]);
                        i += 2;
                    } else {
                        return Err(TypfError::Other("--output requires an argument".into()));
                    }
                },
                "--size" | "-s" => {
                    if i + 1 < args.len() {
                        size = args[i + 1]
                            .parse()
                            .map_err(|_| TypfError::Other("Invalid size value".into()))?;
                        i += 2;
                    } else {
                        return Err(TypfError::Other("--size requires an argument".into()));
                    }
                },
                "--format" | "-f" => {
                    if i + 1 < args.len() {
                        format = args[i + 1].clone();
                        if !["ppm", "pgm", "pbm", "svg"].contains(&format.as_str()) {
                            return Err(TypfError::Other(
                                "Format must be ppm, pgm, pbm, or svg".into(),
                            ));
                        }
                        i += 2;
                    } else {
                        return Err(TypfError::Other("--format requires an argument".into()));
                    }
                },
                _ => {
                    return Err(TypfError::Other(format!("Unknown option: {}", args[i])));
                },
            }
        }

        Ok(Args {
            text,
            output,
            font,
            _shaper,
            _renderer,
            size,
            format,
        })
    }
}

/// Minimal font for when no real font file is provided
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
        // ASCII characters map directly, everything else gets .notdef
        if ch.is_ascii() {
            Some(ch as u32)
        } else {
            Some(0)
        }
    }

    fn advance_width(&self, _glyph_id: u32) -> f32 {
        // Fixed width for demonstration purposes
        600.0
    }
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Check for special modes first
    let cli_args: Vec<String> = std::env::args().collect();

    // REPL mode
    if cli_args.len() >= 2 && (cli_args[1] == "--repl" || cli_args[1] == "-i") {
        return repl::run_repl().map_err(|e| TypfError::Other(e.to_string()));
    }

    // JSONL batch mode (full JSON job spec)
    if cli_args.len() >= 2 && cli_args[1] == "batch" {
        return jsonl::run_batch().map_err(|e| TypfError::Other(e.to_string()));
    }

    // JSONL stream mode (line-by-line)
    if cli_args.len() >= 2 && cli_args[1] == "stream" {
        return jsonl::run_stream().map_err(|e| TypfError::Other(e.to_string()));
    }

    // Simple batch mode (text lines → files)
    if cli_args
        .iter()
        .any(|arg| arg == "--batch" || arg.starts_with("--batch-"))
    {
        let config = batch::BatchConfig::parse(&cli_args[1..])?;
        let font = Arc::new(StubFont::new());
        let shaper = Arc::new(NoneShaper::new());
        let renderer = Arc::new(OrgeRenderer::new());

        let exporter = match config.format.as_str() {
            "ppm" => Arc::new(PnmExporter::ppm()),
            "pgm" => Arc::new(PnmExporter::pgm()),
            "pbm" => Arc::new(PnmExporter::new(typf_export::PnmFormat::Pbm)),
            _ => unreachable!(),
        };

        batch::process_batch(&config, shaper, renderer, exporter, font)?;
        return Ok(());
    }

    // Parse command-line arguments
    let args = Args::parse()?;

    println!("TYPF CLI v2.0");
    println!("Rendering \"{}\" at {}pt", args.text, args.size);

    // Load font (real or stub)
    let font: Arc<dyn FontRef> = if let Some(font_path) = &args.font {
        println!("Loading font from {}", font_path.display());
        Arc::new(Font::from_file(font_path)?)
    } else {
        println!("Using stub font (no real font file provided)");
        Arc::new(StubFont::new())
    };

    // Create shaping parameters
    let shaping_params = ShapingParams {
        size: args.size,
        direction: Direction::LeftToRight,
        language: Some("en".to_string()),
        script: None,
        features: Vec::new(),
        variations: Vec::new(),
        letter_spacing: 0.0,
    };

    // Create rendering parameters
    let render_params = RenderParams {
        foreground: Color::black(),
        background: Some(Color::white()),
        padding: 10,
        antialias: true,
        variations: Vec::new(),
    };

    // Create backends
    let shaper = Arc::new(NoneShaper::new());

    // Shape the text
    println!("Shaping text...");
    let shaped = shaper.shape(&args.text, font.clone(), &shaping_params)?;

    // Handle SVG export separately (works directly from ShapingResult)
    if args.format == "svg" {
        println!("Exporting to SVG...");

        // SVG export requires real font data for outline extraction
        if font.data().is_empty() {
            eprintln!("ERROR: SVG export requires a real font file.");
            eprintln!("The current CLI uses a stub font for demonstration.");
            eprintln!();
            eprintln!("SVG export is fully implemented in the library.");
            eprintln!("To use it, either:");
            eprintln!("  1. Use the Python bindings with a real font file");
            eprintln!("  2. Use the Rust library directly");
            eprintln!("  3. Wait for font file loading in the CLI");
            return Err(TypfError::Other("SVG export requires real font data".into()));
        }

        let svg_exporter = SvgExporter::new();
        let svg_data = svg_exporter.export(&shaped, font, render_params.foreground)?;

        let mut file = File::create(&args.output)?;
        file.write_all(svg_data.as_bytes())?;

        println!("✓ Successfully exported to {}", args.output.display());
        println!("  Format: SVG (vector)");
        println!("  Glyphs: {}", shaped.glyphs.len());
        return Ok(());
    }

    // For bitmap formats, create renderer and exporter
    let renderer = Arc::new(OrgeRenderer::new());
    let exporter = match args.format.as_str() {
        "ppm" => Arc::new(PnmExporter::ppm()),
        "pgm" => Arc::new(PnmExporter::pgm()),
        "pbm" => Arc::new(PnmExporter::new(typf_export::PnmFormat::Pbm)),
        _ => return Err(TypfError::Other(format!("Unknown format: {}", args.format))),
    };

    // Render to bitmap
    println!("Rendering bitmap...");
    let rendered = renderer.render(&shaped, font, &render_params)?;

    // Export to file
    println!("Exporting to {}...", args.output.display());
    let exported = exporter.export(&rendered)?;

    // Write to file
    let mut file = File::create(&args.output)?;
    file.write_all(&exported)?;

    println!("✓ Successfully rendered to {}", args.output.display());
    println!("  Format: {}", args.format.to_uppercase());
    println!(
        "  Size: {}×{} pixels",
        if let RenderOutput::Bitmap(ref bitmap) = rendered {
            bitmap.width
        } else {
            0
        },
        if let RenderOutput::Bitmap(ref bitmap) = rendered {
            bitmap.height
        } else {
            0
        }
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_font() {
        let font = StubFont::new();
        assert_eq!(font.units_per_em(), 1000);
        assert_eq!(font.glyph_id('A'), Some(65));
        assert_eq!(font.glyph_id('😀'), Some(0)); // Non-ASCII returns .notdef
        assert_eq!(font.advance_width(65), 600.0);
    }
}
