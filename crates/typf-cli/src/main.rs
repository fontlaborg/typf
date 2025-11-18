//! TYPF CLI - Command-line interface for the TYPF text rendering pipeline

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
use typf_render_orge::OrgeRenderer;
use typf_shape_none::NoneShaper;

/// Simple command-line arguments
#[derive(Debug)]
struct Args {
    /// Text to render
    text: String,
    /// Output file path
    output: PathBuf,
    /// Font size in points
    size: f32,
    /// Output format (ppm, pgm, pbm)
    format: String,
}

impl Args {
    fn parse() -> Result<Self> {
        let args: Vec<String> = std::env::args().collect();

        if args.len() < 2 {
            eprintln!("Usage: {} <text|--repl> [options]", args[0]);
            eprintln!("Options:");
            eprintln!("  --output <file>  Output file (default: output.ppm)");
            eprintln!("  --size <size>    Font size in points (default: 16)");
            eprintln!("  --format <fmt>   Output format: ppm, pgm, pbm (default: ppm)");
            eprintln!("  --repl           Start interactive REPL mode");
            eprintln!();
            eprintln!("Example:");
            eprintln!("  {} \"Hello World\" --output hello.ppm --size 24", args[0]);
            eprintln!("  {} --repl", args[0]);
            std::process::exit(1);
        }

        let text = args[1].clone();
        let mut output = PathBuf::from("output.ppm");
        let mut size = 16.0;
        let mut format = "ppm".to_string();

        let mut i = 2;
        while i < args.len() {
            match args[i].as_str() {
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
                        if !["ppm", "pgm", "pbm"].contains(&format.as_str()) {
                            return Err(TypfError::Other("Format must be ppm, pgm, or pbm".into()));
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
            size,
            format,
        })
    }
}

/// A stub font implementation for testing
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
        // Simple mapping: use Unicode code point as glyph ID for ASCII
        if ch.is_ascii() {
            Some(ch as u32)
        } else {
            Some(0) // .notdef
        }
    }

    fn advance_width(&self, _glyph_id: u32) -> f32 {
        // Fixed advance width for simplicity
        600.0
    }
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Check for REPL mode first
    let cli_args: Vec<String> = std::env::args().collect();
    if cli_args.len() >= 2 && (cli_args[1] == "--repl" || cli_args[1] == "-i") {
        return repl::run_repl().map_err(|e| TypfError::Other(e.to_string()));
    }

    // Parse command-line arguments
    let args = Args::parse()?;

    println!("TYPF CLI v2.0");
    println!("Rendering \"{}\" at {}pt", args.text, args.size);

    // Create a stub font
    let font = Arc::new(StubFont::new());

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
    };

    // Create backends
    let shaper = Arc::new(NoneShaper::new());
    let renderer = Arc::new(OrgeRenderer::new());

    // Create exporter based on format
    let exporter = match args.format.as_str() {
        "ppm" => Arc::new(PnmExporter::ppm()),
        "pgm" => Arc::new(PnmExporter::pgm()),
        "pbm" => Arc::new(PnmExporter::new(typf_export::PnmFormat::Pbm)),
        _ => unreachable!(),
    };

    // Shape the text
    println!("Shaping text...");
    let shaped = shaper.shape(&args.text, font.clone(), &shaping_params)?;

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
