//! Info command implementation
//!
//! Displays information about available backends and formats.

use crate::cli::InfoArgs;
use typf::error::Result;

pub fn run(args: &InfoArgs) -> Result<()> {
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

    Ok(())
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
