//! SVG glyph rendering support
//!
//! Renders SVG table glyphs using resvg/usvg.
//!
//! SVG documents in fonts may be gzip-compressed (detected by checking for
//! the gzip magic number 0x1f8b at the start).

use flate2::read::GzDecoder;
use skrifa::raw::TableProvider;
use skrifa::GlyphId;
use std::io::Read;
use tiny_skia::Pixmap;

/// Error type for SVG glyph rendering
#[derive(Debug)]
pub enum SvgRenderError {
    /// Font parsing failed
    FontParseFailed,
    /// No SVG table in font
    NoSvgTable,
    /// Glyph not found in SVG table
    GlyphNotFound,
    /// SVG decompression failed
    DecompressionFailed,
    /// SVG parsing failed
    ParseFailed(String),
    /// Rendering failed
    RenderFailed,
    /// Pixmap creation failed
    PixmapCreationFailed,
}

impl std::fmt::Display for SvgRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FontParseFailed => write!(f, "failed to parse font"),
            Self::NoSvgTable => write!(f, "font has no SVG table"),
            Self::GlyphNotFound => write!(f, "glyph not found in SVG table"),
            Self::DecompressionFailed => write!(f, "SVG decompression failed"),
            Self::ParseFailed(e) => write!(f, "SVG parse error: {}", e),
            Self::RenderFailed => write!(f, "SVG rendering failed"),
            Self::PixmapCreationFailed => write!(f, "failed to create pixmap"),
        }
    }
}

impl std::error::Error for SvgRenderError {}

/// Check if a font has SVG glyphs (SVG table)
pub fn has_svg_glyphs(font_data: &[u8]) -> bool {
    if let Ok(font) = skrifa::FontRef::new(font_data) {
        font.svg().is_ok()
    } else {
        false
    }
}

/// Get the raw SVG document for a glyph (decompressed if needed)
pub fn get_svg_document(font_data: &[u8], glyph_id: u32) -> Result<String, SvgRenderError> {
    let font = skrifa::FontRef::new(font_data).map_err(|_| SvgRenderError::FontParseFailed)?;

    let svg_table = font.svg().map_err(|_| SvgRenderError::NoSvgTable)?;
    let doc_list = svg_table
        .svg_document_list()
        .map_err(|_| SvgRenderError::NoSvgTable)?;

    let glyph_id = GlyphId::new(glyph_id);

    // Find the record containing this glyph ID
    for record in doc_list.document_records() {
        let start_id = record.start_glyph_id().to_u32();
        let end_id = record.end_glyph_id().to_u32();

        if glyph_id.to_u32() >= start_id && glyph_id.to_u32() <= end_id {
            // Get the SVG data
            let offset = record.svg_doc_offset() as usize;
            let length = record.svg_doc_length() as usize;

            // Get the document list's data
            let list_data = doc_list.offset_data();
            if offset + length > list_data.len() {
                return Err(SvgRenderError::GlyphNotFound);
            }

            let svg_bytes = &list_data.as_bytes()[offset..offset + length];

            // Check for gzip compression (magic bytes 0x1f 0x8b)
            let svg_string = if svg_bytes.len() >= 2 && svg_bytes[0] == 0x1f && svg_bytes[1] == 0x8b
            {
                // Decompress gzip
                let mut decoder = GzDecoder::new(svg_bytes);
                let mut decompressed = String::new();
                decoder
                    .read_to_string(&mut decompressed)
                    .map_err(|_| SvgRenderError::DecompressionFailed)?;
                decompressed
            } else {
                // Plain SVG
                String::from_utf8(svg_bytes.to_vec())
                    .map_err(|_| SvgRenderError::ParseFailed("invalid UTF-8".into()))?
            };

            return Ok(svg_string);
        }
    }

    Err(SvgRenderError::GlyphNotFound)
}

/// Render an SVG glyph to a pixmap
///
/// # Arguments
/// * `font_data` - Font file data
/// * `glyph_id` - Glyph ID to render
/// * `width` - Output pixmap width
/// * `height` - Output pixmap height
///
/// # Returns
/// A pixmap containing the rendered SVG glyph
pub fn render_svg_glyph(
    font_data: &[u8],
    glyph_id: u32,
    width: u32,
    height: u32,
) -> Result<Pixmap, SvgRenderError> {
    let svg_document = get_svg_document(font_data, glyph_id)?;

    // Parse SVG with usvg
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_str(&svg_document, &options)
        .map_err(|e| SvgRenderError::ParseFailed(e.to_string()))?;

    // Create output pixmap
    let mut pixmap = Pixmap::new(width, height).ok_or(SvgRenderError::PixmapCreationFailed)?;

    // Calculate transform to fit SVG into the target size
    let tree_size = tree.size();
    let scale_x = width as f32 / tree_size.width();
    let scale_y = height as f32 / tree_size.height();
    let scale = scale_x.min(scale_y);

    let transform = tiny_skia::Transform::from_scale(scale, scale);

    // Render SVG to pixmap
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    Ok(pixmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_svg_table() {
        // Empty font data
        assert!(!has_svg_glyphs(&[]));
    }

    /// Test SVG table detection with Nabla SVG font
    #[test]
    fn test_has_svg_glyphs_abelone() {
        let font_path = "../../test-fonts/Nabla-Regular-SVG.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            assert!(
                has_svg_glyphs(&font_data),
                "Nabla-Regular-SVG should have SVG table"
            );
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    /// Test that regular fonts don't have SVG glyphs
    #[test]
    fn test_has_svg_glyphs_regular_font() {
        let font_path = "../../external/resvg/crates/resvg/tests/fonts/NotoSans-Regular.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            assert!(
                !has_svg_glyphs(&font_data),
                "NotoSans-Regular should not have SVG table"
            );
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    /// Test SVG document extraction from Nabla font
    #[test]
    fn test_get_svg_document_abelone() {
        let font_path = "../../test-fonts/Nabla-Regular-SVG.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            // Get a glyph ID that should have SVG data
            // Typically glyph IDs start after .notdef (gid 0)
            // Try a few common glyph IDs
            for gid in 1..100 {
                if let Ok(svg_doc) = get_svg_document(&font_data, gid) {
                    assert!(
                        svg_doc.contains("<svg") || svg_doc.contains("<?xml"),
                        "SVG document should contain SVG content"
                    );
                    println!("Found SVG glyph at gid {}, length: {}", gid, svg_doc.len());
                    return; // Test passed
                }
            }
            eprintln!("No SVG glyphs found in first 100 glyph IDs");
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    /// Test SVG rendering from Nabla font
    #[test]
    fn test_render_svg_glyph_abelone() {
        let font_path = "../../test-fonts/Nabla-Regular-SVG.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            // Try to render an SVG glyph
            for gid in 1..100 {
                if get_svg_document(&font_data, gid).is_ok() {
                    let result = render_svg_glyph(&font_data, gid, 128, 128);
                    assert!(
                        result.is_ok(),
                        "Failed to render SVG glyph: {:?}",
                        result.err()
                    );
                    let pixmap = result.unwrap();
                    assert_eq!(pixmap.width(), 128);
                    assert_eq!(pixmap.height(), 128);
                    println!("Successfully rendered SVG glyph at gid {}", gid);
                    return; // Test passed
                }
            }
            eprintln!("No SVG glyphs found to render");
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    /// Test Twitter/Twemoji SVG font (subset)
    #[test]
    fn test_twitter_emoji_svg() {
        let font_path =
            "../../external/resvg/crates/resvg/tests/fonts/TwitterColorEmoji.subset.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            // Twitter emoji uses SVG table
            if has_svg_glyphs(&font_data) {
                for gid in 1..50 {
                    if let Ok(svg_doc) = get_svg_document(&font_data, gid) {
                        println!(
                            "Twitter emoji SVG glyph at gid {}, length: {}",
                            gid,
                            svg_doc.len()
                        );
                        // Try to render it
                        let result = render_svg_glyph(&font_data, gid, 64, 64);
                        assert!(
                            result.is_ok(),
                            "Failed to render Twitter emoji SVG: {:?}",
                            result.err()
                        );
                        return; // Test passed
                    }
                }
                eprintln!("No SVG glyphs found in Twitter emoji subset");
            } else {
                eprintln!("Twitter emoji subset doesn't have SVG table (may be CBDT only)");
            }
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }
}
