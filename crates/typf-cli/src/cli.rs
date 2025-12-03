//! CLI argument definitions using Clap v4
//!
//! This module defines the command-line interface structure
//! following the linra Typf CLI specification.

use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Typf - Professional text rendering from the command line
#[derive(Parser, Debug)]
#[command(name = "typf")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Display information about available backends and formats
    #[command(alias = "i")]
    Info(InfoArgs),

    /// Render text to an image file
    #[command(alias = "r")]
    Render(Box<RenderArgs>),

    /// Process multiple rendering jobs from a JSONL file
    Batch(BatchArgs),
}

/// Arguments for the info command
#[derive(Parser, Debug)]
pub struct InfoArgs {
    /// List available shaping backends
    #[arg(long)]
    pub shapers: bool,

    /// List available rendering backends
    #[arg(long)]
    pub renderers: bool,

    /// List available output formats
    #[arg(long)]
    pub formats: bool,
}

/// Arguments for the render command
#[derive(Parser, Debug)]
pub struct RenderArgs {
    // Text Input
    /// Input text to render (reads from stdin if omitted)
    pub text: Option<String>,

    // Font Face Options
    /// Font file path (.ttf, .otf, .ttc, .otc)
    #[arg(short = 'f', long = "font-file")]
    pub font_file: Option<PathBuf>,

    /// Face index for TTC/OTC collections
    #[arg(short = 'y', long = "face-index", default_value = "0")]
    pub face_index: u32,

    /// Named/dynamic instance spec
    #[arg(short = 'i', long = "instance")]
    pub instance: Option<String>,

    // Text Input Options
    /// Input text (alternative to positional argument)
    #[arg(short = 't', long = "text", conflicts_with = "text_file")]
    pub text_arg: Option<String>,

    /// Read input text from file
    #[arg(short = 'T', long = "text-file", conflicts_with = "text_arg")]
    pub text_file: Option<PathBuf>,

    // Shaper and Renderer Options
    /// Shaping backend: auto, none, hb, icu-hb, mac, win
    /// (ignored when using linra renderer)
    #[arg(long = "shaper", default_value = "auto")]
    pub shaper: String,

    /// Rendering backend: auto, opixa, skia, zeno, mac, win, json, linra-mac, linra-win
    /// Linra backends (linra-mac, linra-win) perform shaping AND rendering in one pass
    #[arg(long = "renderer", default_value = "auto")]
    pub renderer: String,

    // Text Processing Options
    /// Text direction: auto, ltr, rtl, ttb, btt
    #[arg(short = 'd', long = "direction", default_value = "auto")]
    pub direction: String,

    /// Language tag (BCP 47), e.g., en, ar, zh-Hans
    #[arg(short = 'l', long = "language")]
    pub language: Option<String>,

    /// Script tag (ISO 15924), e.g., Latn, Arab, Hans
    #[arg(short = 'S', long = "script", default_value = "auto")]
    pub script: String,

    // Font Features Options
    /// Font feature settings (comma or space separated)
    #[arg(short = 'F', long = "features")]
    pub features: Option<String>,

    // Size and Crop Options
    /// Font size in pixels (or 'em' for UPM)
    #[arg(short = 's', long = "font-size", default_value = "200")]
    pub font_size: String,

    /// Line height as % of font size
    #[arg(short = 'L', long = "line-height", default_value = "120")]
    pub line_height: u32,

    /// Canvas size spec: <width>x<height>, <width>x, x<height>, or none
    #[arg(short = 'W', long = "width-height", default_value = "none")]
    pub width_height: String,

    /// Margin in pixels
    #[arg(short = 'm', long = "margin", default_value = "10")]
    pub margin: u32,

    /// Optical sizing: auto, none
    #[arg(long = "font-optical-sizing", default_value = "auto")]
    pub font_optical_sizing: String,

    // Color Options
    /// Text color (RRGGBB or RRGGBBAA)
    #[arg(short = 'c', long = "foreground", default_value = "000000FF")]
    pub foreground: String,

    /// Background color (RRGGBB or RRGGBBAA)
    #[arg(short = 'b', long = "background", default_value = "FFFFFF00")]
    pub background: String,

    /// Font CPAL palette index
    #[arg(short = 'p', long = "color-palette", default_value = "0")]
    pub color_palette: u32,

    /// Glyph source preferences or deny list.
    ///
    /// Control which glyph data sources are used and in what order.
    /// Sources: glyf, cff, cff2, colr0, colr1, svg, sbix, cbdt, ebdt
    ///
    /// Examples:
    ///   --glyph-source prefer=glyf,cff2         Use outline sources first
    ///   --glyph-source deny=colr0,colr1,svg     Disable all color sources
    ///   --glyph-source prefer=colr1,colr0,svg   Prefer COLR over SVG
    ///   --glyph-source deny=sbix,cbdt,ebdt      Disable bitmap sources
    #[arg(long = "glyph-source", action = ArgAction::Append, verbatim_doc_comment)]
    pub glyph_source: Vec<String>,

    /// Disable shaping cache (enabled by default)
    #[arg(long = "no-shaping-cache", action = ArgAction::SetTrue)]
    pub no_shaping_cache: bool,

    /// Disable glyph/render cache (enabled by default)
    #[arg(long = "no-glyph-cache", action = ArgAction::SetTrue)]
    pub no_glyph_cache: bool,

    // Output Options
    /// Output file path (stdout if omitted)
    #[arg(short = 'o', long = "output-file")]
    pub output_file: Option<PathBuf>,

    /// Output format: pbm, png1, pgm, png4, png8, png, svg
    #[arg(short = 'O', long = "format", default_value = "png")]
    pub format: OutputFormat,

    /// Silent mode (no progress info)
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Verbose output
    #[arg(long = "verbose")]
    pub verbose: bool,
}

/// Arguments for the batch command
#[derive(Parser, Debug)]
pub struct BatchArgs {
    /// Input JSONL file (one job per line)
    #[arg(short = 'i', long = "input")]
    pub input: Option<PathBuf>,

    /// Output directory for rendered files
    #[arg(short = 'o', long = "output", default_value = ".")]
    pub output: PathBuf,

    /// Output filename pattern with {} placeholder
    #[arg(short = 'p', long = "pattern", default_value = "output_{}")]
    pub pattern: String,

    /// Silent mode
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Verbose output
    #[arg(long = "verbose")]
    pub verbose: bool,
}

/// Supported output formats
#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Monochrome PBM (no antialiasing)
    Pbm,
    /// Monochrome PNG (no antialiasing)
    Png1,
    /// 8-bit grayscale PGM (antialiased)
    Pgm,
    /// 4-bit grayscale PNG (antialiased)
    Png4,
    /// 8-bit grayscale PNG (antialiased)
    Png8,
    /// RGBA PNG (antialiased, full color)
    Png,
    /// SVG vector paths
    Svg,
    /// PPM format (for backward compatibility)
    Ppm,
}

impl OutputFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pbm => "pbm",
            Self::Png1 => "png1",
            Self::Pgm => "pgm",
            Self::Png4 => "png4",
            Self::Png8 => "png8",
            Self::Png => "png",
            Self::Svg => "svg",
            Self::Ppm => "ppm",
        }
    }
}
