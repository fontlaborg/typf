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

/// Substitute CSS custom properties (var(--colorN, fallback)) with actual colors from CPAL palette.
///
/// OpenType-SVG fonts use CSS custom properties to reference CPAL palette colors.
/// Since usvg doesn't support CSS variables, we preprocess the SVG to substitute them.
///
/// Pattern: `var(--color0, #RRGGBB)` or `var(--color0, colorname)`
/// The --colorN refers to CPAL palette index N.
fn substitute_css_variables(svg: &str, palette_colors: &[skrifa::color::Color]) -> String {
    use std::borrow::Cow;

    // Regex pattern: var(--color<N>, <fallback>)
    // We need to handle:
    // - var(--color0, #FFD214)
    // - var(--color9, white)
    // - Nested in style attributes or CSS
    let mut result = Cow::Borrowed(svg);

    // Process each potential color variable (0-15 should cover most fonts)
    for i in 0..16u16 {
        let var_pattern = format!("var(--color{}", i);

        if !result.contains(&var_pattern) {
            continue;
        }

        // Get the color from palette, or use a default
        let color_hex = if let Some(color) = palette_colors.get(i as usize) {
            format!(
                "#{:02X}{:02X}{:02X}",
                color.red, color.green, color.blue
            )
        } else {
            // No palette color available, we'll let the fallback be used
            continue;
        };

        // Replace all occurrences of var(--colorN, ...) with the actual color
        // We need to find the matching closing paren, handling the fallback value
        let mut new_result = String::with_capacity(result.len());
        let mut chars = result.char_indices().peekable();

        while let Some((idx, _)) = chars.next() {
            let remaining = &result[idx..];

            if remaining.starts_with(&var_pattern) {
                // Found a match, find the closing paren
                if let Some(close_paren) = remaining.find(')') {
                    // Replace the entire var(...) with the color
                    new_result.push_str(&color_hex);

                    // Skip past the var(...) in the source
                    let skip_count = close_paren;
                    for _ in 0..skip_count {
                        chars.next();
                    }
                    continue;
                }
            }

            new_result.push(result[idx..].chars().next().unwrap());
        }

        if !new_result.is_empty() {
            result = Cow::Owned(new_result);
        }
    }

    result.into_owned()
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
    // Render without palette (uses fallback colors in CSS vars)
    render_svg_glyph_with_palette(font_data, glyph_id, width, height, &[])
}

/// Render an SVG glyph to a pixmap with CPAL palette color substitution
///
/// # Arguments
/// * `font_data` - Font file data
/// * `glyph_id` - Glyph ID to render
/// * `width` - Output pixmap width
/// * `height` - Output pixmap height
/// * `palette_colors` - Colors from CPAL palette to substitute for CSS variables
///
/// # Returns
/// A pixmap containing the rendered SVG glyph with correct colors
pub fn render_svg_glyph_with_palette(
    font_data: &[u8],
    glyph_id: u32,
    width: u32,
    height: u32,
    palette_colors: &[skrifa::color::Color],
) -> Result<Pixmap, SvgRenderError> {
    // Call the ppem version with a default ppem based on output size
    // This maintains backwards compatibility for callers without ppem
    render_svg_glyph_with_palette_and_ppem(
        font_data,
        glyph_id,
        width,
        height,
        palette_colors,
        height as f32, // Use height as ppem for backwards compat
    )
}

/// Extract just the content for a specific glyph from an OpenType-SVG document.
///
/// OpenType-SVG documents may contain multiple glyphs. Each glyph is identified
/// by a `<g id="glyphXX">` element where XX is the glyph ID. This function
/// extracts just that glyph's content and wraps it in a standalone SVG.
///
/// The `upem` parameter is used to set a proper viewBox in font units.
fn extract_glyph_svg(
    svg_document: &str,
    glyph_id: u32,
    upem: u16,
) -> Result<String, SvgRenderError> {
    // Look for the glyph's group element: <g id="glyphXX">...</g>
    let glyph_pattern = format!(r#"id="glyph{}""#, glyph_id);

    // If the document doesn't have our glyph pattern, return as-is
    // (might be a single-glyph document)
    if !svg_document.contains(&glyph_pattern) {
        return Ok(svg_document.to_string());
    }

    // Extract the defs section (gradients, patterns, etc. that glyphs reference)
    let defs_section = extract_defs(svg_document);

    // Extract the namespace declarations from the original svg tag
    let namespaces = extract_namespaces(svg_document);

    // Find the glyph group and extract its content
    // Pattern: <g id="glyphXX" ...>content</g>
    let start_tag = format!(r#"<g id="glyph{}""#, glyph_id);
    let start_pos = match svg_document.find(&start_tag) {
        Some(pos) => pos,
        None => return Err(SvgRenderError::GlyphNotFound),
    };

    // Find the end of the opening tag
    let tag_end = match svg_document[start_pos..].find('>') {
        Some(pos) => start_pos + pos + 1,
        None => return Err(SvgRenderError::ParseFailed("malformed glyph group".into())),
    };

    // Find the matching closing </g> tag
    // We need to handle nested <g> elements
    let glyph_content = extract_group_content(&svg_document[tag_end..])
        .ok_or(SvgRenderError::ParseFailed("malformed glyph group".into()))?;

    // Build a new standalone SVG document with just this glyph
    // For OpenType-SVG, coordinates are in font units (Y-up, origin at baseline).
    // Y=0 is the baseline, positive Y is above baseline (ascenders), negative Y is below (descenders).
    // We use viewBox covering -upem to +upem in Y to capture both ascenders and descenders.
    // The viewBox height is 2*upem to cover this full range.
    let standalone_svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg"{} viewBox="0 -{upem} {upem} {double_upem}">{}{}</svg>"#,
        namespaces, defs_section, glyph_content, upem = upem, double_upem = upem * 2
    );

    Ok(standalone_svg)
}

/// Extract the <defs> section from an SVG document
fn extract_defs(svg: &str) -> String {
    // Find all defs sections and concatenate them
    let mut defs_content = String::new();
    let mut search_from = 0;

    while let Some(start) = svg[search_from..].find("<defs") {
        let abs_start = search_from + start;
        if let Some(end) = svg[abs_start..].find("</defs>") {
            let abs_end = abs_start + end + 7; // 7 = len("</defs>")
            defs_content.push_str(&svg[abs_start..abs_end]);
            search_from = abs_end;
        } else {
            break;
        }
    }

    defs_content
}

/// Extract namespace declarations from the svg tag (e.g., xmlns:xlink)
fn extract_namespaces(svg: &str) -> String {
    let mut namespaces = String::new();

    // Check if xlink: prefix is used anywhere in the document
    // If so, we need to declare the namespace
    if svg.contains("xlink:") {
        namespaces.push_str(r#" xmlns:xlink="http://www.w3.org/1999/xlink""#);
    }

    namespaces
}

/// Extract content from a group element, handling nested groups
fn extract_group_content(svg_after_tag: &str) -> Option<String> {
    let mut depth = 1;
    let mut pos = 0;
    let bytes = svg_after_tag.as_bytes();

    while pos < bytes.len() && depth > 0 {
        if pos + 2 < bytes.len() && &bytes[pos..pos + 2] == b"<g" {
            // Check if it's actually a <g> tag (not <gradient, etc.)
            if pos + 3 >= bytes.len() || bytes[pos + 2] == b' ' || bytes[pos + 2] == b'>' {
                depth += 1;
            }
        } else if pos + 3 < bytes.len() && &bytes[pos..pos + 4] == b"</g>" {
            depth -= 1;
            if depth == 0 {
                return Some(svg_after_tag[..pos].to_string());
            }
        }
        pos += 1;
    }

    None
}

/// Render an SVG glyph to a pixmap with proper font-unit to pixel scaling
///
/// # Arguments
/// * `font_data` - Font file data
/// * `glyph_id` - Glyph ID to render
/// * `_width` - Ignored (output size is computed from SVG tree and ppem)
/// * `_height` - Ignored (output size is computed from SVG tree and ppem)
/// * `palette_colors` - Colors from CPAL palette to substitute for CSS variables
/// * `ppem` - Pixels per em (font size in pixels)
///
/// # Returns
/// A pixmap containing the rendered SVG glyph with correct colors and scaling
/// Note: The output pixmap size is calculated from the SVG's actual dimensions
/// scaled by ppem/upem, not from the passed width/height parameters.
pub fn render_svg_glyph_with_palette_and_ppem(
    font_data: &[u8],
    glyph_id: u32,
    _width: u32,
    _height: u32,
    palette_colors: &[skrifa::color::Color],
    ppem: f32,
) -> Result<Pixmap, SvgRenderError> {
    let font = skrifa::FontRef::new(font_data).map_err(|_| SvgRenderError::FontParseFailed)?;

    // Get font's units per em - needed for proper viewBox in extracted SVG
    let upem = font
        .head()
        .map(|h| h.units_per_em())
        .unwrap_or(1000);

    let svg_document = get_svg_document(font_data, glyph_id)?;

    log::debug!(
        "SVG render: glyph_id={}, palette_colors={}, ppem={}, svg_len={}",
        glyph_id,
        palette_colors.len(),
        ppem,
        svg_document.len()
    );

    // Extract just this glyph's content from the shared SVG document
    let glyph_svg = extract_glyph_svg(&svg_document, glyph_id, upem)?;

    log::debug!(
        "SVG extraction: original_len={}, glyph_len={}",
        svg_document.len(),
        glyph_svg.len()
    );

    // Substitute CSS variables with actual palette colors
    let processed_svg = if !palette_colors.is_empty() {
        let result = substitute_css_variables(&glyph_svg, palette_colors);
        log::debug!(
            "SVG substitution: had var()={}, still has var()={}",
            glyph_svg.contains("var("),
            result.contains("var(")
        );
        result
    } else {
        log::debug!("SVG render: no palette colors provided");
        glyph_svg
    };

    // Parse SVG with usvg
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_str(&processed_svg, &options)
        .map_err(|e| SvgRenderError::ParseFailed(e.to_string()))?;

    // Calculate scale to convert from font units to pixels
    // SVG documents are designed to fill 1em x 1em in font units
    // ppem = pixels per em, so scale = ppem / upem
    let scale = ppem / upem as f32;

    // Calculate the output pixmap size based on SVG tree dimensions and scale
    // The SVG tree dimensions are in font units, so we scale them to pixels
    let svg_width = tree.size().width();
    let svg_height = tree.size().height();
    let output_width = ((svg_width * scale).ceil() as u32).max(1);
    let output_height = ((svg_height * scale).ceil() as u32).max(1);

    log::debug!(
        "SVG scaling: upem={}, ppem={}, scale={}, tree_size={}x{}, output={}x{}",
        upem,
        ppem,
        scale,
        svg_width,
        svg_height,
        output_width,
        output_height
    );

    // Create output pixmap with the correct size
    let mut pixmap =
        Pixmap::new(output_width, output_height).ok_or(SvgRenderError::PixmapCreationFailed)?;

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

    #[test]
    fn test_css_variable_substitution() {
        // Test the CSS variable substitution logic
        let svg = r#"<svg><path fill="var(--color0, #FFD214)" d="..."/></svg>"#;
        let palette = vec![skrifa::color::Color {
            red: 255,
            green: 0,
            blue: 0,
            alpha: 255,
        }];
        let result = substitute_css_variables(svg, &palette);
        assert!(
            result.contains("#FF0000"),
            "Should substitute with red: {}",
            result
        );
        assert!(
            !result.contains("var("),
            "Should not contain var(): {}",
            result
        );
    }

    #[test]
    fn test_css_variable_substitution_multiple() {
        let svg = r#"<svg>
            <path fill="var(--color0, yellow)"/>
            <path fill="var(--color1, blue)"/>
            <path fill="var(--color0, yellow)"/>
        </svg>"#;
        let palette = vec![
            skrifa::color::Color {
                red: 255,
                green: 0,
                blue: 0,
                alpha: 255,
            },
            skrifa::color::Color {
                red: 0,
                green: 255,
                blue: 0,
                alpha: 255,
            },
        ];
        let result = substitute_css_variables(svg, &palette);
        // Should have two occurrences of #FF0000 (color0) and one of #00FF00 (color1)
        assert_eq!(
            result.matches("#FF0000").count(),
            2,
            "Should have 2 color0 substitutions: {}",
            result
        );
        assert_eq!(
            result.matches("#00FF00").count(),
            1,
            "Should have 1 color1 substitution: {}",
            result
        );
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
                    // Output size is now computed from tree dimensions, not passed params.
                    // Width should match the em-square scaled to ppem (128).
                    // Height is 2x em-square to cover both ascenders and descenders.
                    assert_eq!(pixmap.width(), 128, "Width should match ppem");
                    assert_eq!(pixmap.height(), 256, "Height should be 2x ppem for full em-square");
                    println!("Successfully rendered SVG glyph at gid {}", gid);
                    return; // Test passed
                }
            }
            eprintln!("No SVG glyphs found to render");
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    /// Test that palette colors are correctly applied to SVG
    #[test]
    fn test_render_svg_with_palette_colors() {
        use skrifa::color::ColorPalettes;

        let font_path = "../../test-fonts/Nabla-Regular-SVG.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            let font = skrifa::FontRef::new(&font_data).unwrap();

            // Get palette colors
            let palettes = ColorPalettes::new(&font);
            let palette_colors: Vec<_> = palettes
                .get(0)
                .map(|p| p.colors().to_vec())
                .unwrap_or_default();

            println!("Palette has {} colors", palette_colors.len());
            for (i, c) in palette_colors.iter().enumerate() {
                println!(
                    "  --color{}: #{:02X}{:02X}{:02X}",
                    i, c.red, c.green, c.blue
                );
            }

            // Get SVG document to check what vars are used
            let svg_doc = get_svg_document(&font_data, 2).unwrap();
            let has_vars = svg_doc.contains("var(--color");
            println!("SVG has CSS vars: {}", has_vars);

            // Render with palette
            let pixmap =
                render_svg_glyph_with_palette(&font_data, 2, 128, 128, &palette_colors).unwrap();

            // Count colored pixels (non-black, non-transparent)
            let colored_pixels = pixmap
                .data()
                .chunks(4)
                .filter(|p| p[3] > 0 && (p[0] > 10 || p[1] > 10 || p[2] > 10))
                .count();

            println!("Colored (non-black) pixels: {}", colored_pixels);

            // Nabla font should have colored pixels after substitution
            assert!(
                colored_pixels > 0,
                "Should have colored pixels after palette substitution"
            );
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
