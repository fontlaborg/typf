//! Info command implementation
//!
//! Displays information about available backends and formats.
// this_file: crates/typf-cli/src/commands/info.rs

use crate::cli::InfoArgs;
use typf::error::Result;

pub fn run(args: &InfoArgs) -> Result<()> {
    if let Some(path) = &args.font {
        return print_font(path, args.face_index);
    }

    // If no specific flags, show all info
    let show_all = !args.shapers && !args.renderers && !args.formats;

    println!("Typf v{}", env!("CARGO_PKG_VERSION"));
    println!();

    if show_all || args.shapers {
        print_shapers();
        if show_all {
            println!();
        }
    }

    if show_all || args.renderers {
        print_renderers();
        if show_all {
            println!();
        }
    }

    if show_all || args.formats {
        print_formats();
    }

    if show_all {
        println!();
        print_combinations();
    }

    Ok(())
}

/// All traditional shaper backends (the documented 5×7 matrix is independent of
/// which features this binary was compiled with).
const ALL_SHAPERS: [&str; 5] = ["none", "harfbuzz", "harfrust", "icu-hb", "coretext"];

/// All traditional renderer backends.
const ALL_RENDERERS: [&str; 7] = [
    "opixa",
    "skia",
    "zeno",
    "vello-cpu",
    "vello",
    "coregraphics",
    "json",
];

/// Print the full shaper × renderer combination matrix.
///
/// Typf documents 5 shapers × 7 renderers = 35 traditional backend
/// combinations. These are listed here regardless of compiled features so the
/// matrix is discoverable. The single-pass "linra" backends are listed
/// separately because they bypass the shaper/renderer split entirely.
fn print_combinations() {
    let total = ALL_SHAPERS.len() * ALL_RENDERERS.len();
    println!(
        "Backend combinations ({} shapers x {} renderers = {} total):",
        ALL_SHAPERS.len(),
        ALL_RENDERERS.len(),
        total
    );
    for shaper in ALL_SHAPERS {
        for renderer in ALL_RENDERERS {
            println!("  combo: {shaper}+{renderer}");
        }
    }
    println!();
    println!("Single-pass (linra) backends bypass the matrix above:");
    println!("  coretext-linra    - CoreText single-pass (macOS)");
    println!("  directwrite-linra - DirectWrite single-pass (Windows)");
}

fn print_shapers() {
    println!("Shapers:");
    println!("  none              - No shaping (direct character mapping)");

    #[cfg(feature = "shaping-hb")]
    println!("  hb                - HarfBuzz (Unicode-aware text shaping)");

    #[cfg(feature = "shaping-icu-hb")]
    println!("  icu-hb            - ICU + HarfBuzz (advanced Unicode + shaping)");

    #[cfg(all(target_os = "macos", feature = "shaping-mac"))]
    println!("  mac               - CoreText (macOS native)");

    #[cfg(all(target_os = "windows", feature = "shaping-win"))]
    println!("  win               - DirectWrite (Windows native)");
}

fn print_renderers() {
    println!("Renderers (traditional - separate shaping step):");
    println!("  opixa             - Opixa (pure Rust, monochrome/grayscale)");

    #[cfg(feature = "render-skia")]
    println!("  skia              - TinySkia (cross-platform, antialiased)");

    #[cfg(feature = "render-zeno")]
    println!("  zeno              - Zeno (cross-platform vector rasterizer)");

    #[cfg(feature = "render-vello-cpu")]
    println!("  vello-cpu         - Vello CPU (high-quality rasterizer, no GPU)");

    #[cfg(feature = "render-vello")]
    println!("  vello             - Vello GPU (wgpu-accelerated, cross-platform)");

    #[cfg(all(target_os = "macos", feature = "render-mac"))]
    println!("  mac               - CoreGraphics (macOS native)");

    #[cfg(all(target_os = "windows", feature = "render-win"))]
    println!("  win               - Windows GDI+ (Windows native)");

    println!();
    println!("Linra Renderers (single-pass shaping+rendering):");

    #[cfg(feature = "linra-mac")]
    println!("  linra-mac         - CoreText CTLineDraw (macOS, optimal performance)");

    #[cfg(feature = "linra-win")]
    println!("  linra-win         - DirectWrite DrawTextLayout (Windows, optimal performance)");

    #[cfg(not(any(feature = "linra-mac", feature = "linra-win")))]
    println!("  (none available - build with --features linra-mac or linra-win)");
}

fn print_formats() {
    println!("Output Formats:");
    println!("  pbm               - Portable Bitmap (monochrome, no antialiasing)");
    println!("  png1              - PNG monochrome (1-bit)");
    println!("  pgm               - Portable Graymap (8-bit grayscale)");
    println!("  png4              - PNG grayscale (4-bit)");
    println!("  png8              - PNG grayscale (8-bit)");
    println!("  png               - PNG RGBA (full color with alpha)");
    println!("  svg               - Scalable Vector Graphics");
    println!("  ppm               - Portable Pixmap (RGB, legacy)");
}

/// Describe one font face: metrics, axes, and the TrueType hinting tables the
/// `tt` hinting mode depends on.
fn print_font(path: &std::path::Path, face_index: u32) -> Result<()> {
    use skrifa::raw::{types::Tag, TableProvider};
    use typf_core::traits::FontRef;
    use typf_fontdb::TypfFontFace;

    let face = TypfFontFace::from_file_index(path, face_index)?;
    let font = face
        .font_ref()
        .ok_or(typf::error::FontLoadError::InvalidData)?;

    println!("{}", path.display());
    println!("  face index:   {face_index}");
    println!("  unitsPerEm:   {}", face.units_per_em());
    println!("  glyphs:       {}", face.glyph_count().unwrap_or(0));
    match face.variation_axes() {
        Some(axes) if !axes.is_empty() => {
            for a in axes {
                println!(
                    "  axis {}:    {}..{}..{}{}",
                    a.tag,
                    a.min_value,
                    a.default_value,
                    a.max_value,
                    if a.hidden { " (hidden)" } else { "" }
                );
            }
        },
        _ => println!("  axes:         none (static)"),
    }

    let len = |tag: &[u8; 4]| font.table_data(Tag::new(tag)).map(|d| d.len()).unwrap_or(0);
    let has_glyf = font.table_data(Tag::new(b"glyf")).is_some();
    let fpgm = len(b"fpgm");
    let prep = len(b"prep");
    let cvt = len(b"cvt ") / 2;
    let hinted_glyphs = font
        .loca(None)
        .ok()
        .zip(font.glyf().ok())
        .map(|(loca, glyf)| {
            (0..face.glyph_count().unwrap_or(0))
                .filter(|&gid| {
                    matches!(
                        loca.get_glyf(skrifa::raw::types::GlyphId::new(gid), &glyf),
                        Ok(Some(skrifa::raw::tables::glyf::Glyph::Simple(g))) if g.instruction_length() > 0
                    ) || matches!(
                        loca.get_glyf(skrifa::raw::types::GlyphId::new(gid), &glyf),
                        Ok(Some(skrifa::raw::tables::glyf::Glyph::Composite(g))) if g.instructions().is_some_and(|i| !i.is_empty())
                    )
                })
                .count()
        })
        .unwrap_or(0);

    println!("  outlines:     {}", if has_glyf { "glyf (TrueType)" } else { "CFF/CFF2 (no TrueType hinting)" });
    println!("  fpgm:         {fpgm} bytes");
    println!("  prep:         {prep} bytes");
    println!("  cvt:          {cvt} entries");
    println!("  hinted glyphs: {hinted_glyphs}");
    let usable = has_glyf && (fpgm > 0 || prep > 0 || hinted_glyphs > 0);
    println!(
        "  --hinting tt: {}",
        if usable { "applicable" } else { "no effect (no TrueType instructions)" }
    );
    Ok(())
}
