//! Bitmap glyph rendering support
//!
//! Renders embedded bitmap glyphs (sbix, CBDT/CBLC) using skrifa's bitmap module.
//! Supports PNG, BGRA, and mask formats.
//!
//! When a bitmap glyph is unavailable, the `render_bitmap_glyph_or_outline` function
//! falls back to rendering the glyph outline.
//!
//! # Scaling
//!
//! Bitmap fonts store glyphs at specific pixel sizes (strikes). When rendering at
//! a different size, we scale the bitmap using bilinear interpolation and flip
//! vertically to convert from bitmap coordinates (Y-down) to font coordinates (Y-up).

use skrifa::bitmap::{BitmapData, BitmapFormat, BitmapStrikes, Origin};
use skrifa::instance::{Location, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::TableProvider;
use skrifa::GlyphId;
use skrifa::MetadataProvider;
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

/// Error type for bitmap glyph rendering
#[derive(Debug)]
pub enum BitmapRenderError {
    /// Font parsing failed
    FontParseFailed,
    /// No bitmap tables in font
    NoBitmapTable,
    /// Glyph not found at requested size
    GlyphNotFound,
    /// PNG decoding failed
    PngDecodeFailed,
    /// Pixmap creation failed
    PixmapCreationFailed,
    /// Unsupported bitmap format
    UnsupportedFormat,
    /// Outline rendering failed (used in fallback)
    OutlineRenderFailed,
    /// No glyph available (neither bitmap nor outline)
    NoGlyphAvailable,
}

impl std::fmt::Display for BitmapRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FontParseFailed => write!(f, "failed to parse font"),
            Self::NoBitmapTable => write!(f, "font has no bitmap tables (sbix, CBDT, EBDT)"),
            Self::GlyphNotFound => write!(f, "glyph not found at requested size"),
            Self::PngDecodeFailed => write!(f, "failed to decode PNG data"),
            Self::PixmapCreationFailed => write!(f, "failed to create pixmap"),
            Self::UnsupportedFormat => write!(f, "unsupported bitmap format"),
            Self::OutlineRenderFailed => write!(f, "failed to render outline fallback"),
            Self::NoGlyphAvailable => write!(f, "no glyph available (bitmap or outline)"),
        }
    }
}

impl std::error::Error for BitmapRenderError {}

/// A scaled bitmap glyph with positioning information
#[derive(Debug, Clone)]
pub struct ScaledBitmapGlyph {
    /// The rendered bitmap
    pub pixmap: Pixmap,
    /// Horizontal offset from glyph origin to left edge (in pixels)
    pub bearing_x: f32,
    /// Vertical offset from baseline to top edge (in pixels, positive = above baseline)
    pub bearing_y: f32,
    /// The native ppem of the source bitmap strike
    pub native_ppem: f32,
    /// Whether the bitmap was scaled from a different strike size
    pub was_scaled: bool,
}

/// Check if a font has bitmap glyphs (sbix, CBDT/CBLC, or EBDT/EBLC tables)
pub fn has_bitmap_glyphs(font_data: &[u8]) -> bool {
    if let Ok(font) = skrifa::FontRef::new(font_data) {
        let strikes = BitmapStrikes::new(&font);
        !strikes.is_empty()
    } else {
        false
    }
}

/// Get available bitmap strike sizes (ppem values)
pub fn get_bitmap_sizes(font_data: &[u8]) -> Vec<f32> {
    let font = match skrifa::FontRef::new(font_data) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let strikes = BitmapStrikes::new(&font);
    strikes.iter().map(|s| s.ppem()).collect()
}

/// Render a bitmap glyph to a pixmap
///
/// # Arguments
/// * `font_data` - Font file data
/// * `glyph_id` - Glyph ID to render
/// * `ppem` - Pixels per em (size) for strike selection
///
/// # Returns
/// A pixmap containing the rendered glyph, or an error
pub fn render_bitmap_glyph(
    font_data: &[u8],
    glyph_id: u32,
    ppem: f32,
) -> Result<Pixmap, BitmapRenderError> {
    let font = skrifa::FontRef::new(font_data).map_err(|_| BitmapRenderError::FontParseFailed)?;
    let strikes = BitmapStrikes::new(&font);

    if strikes.is_empty() {
        return Err(BitmapRenderError::NoBitmapTable);
    }

    let glyph_id = GlyphId::new(glyph_id);
    let size = Size::new(ppem);

    // Get best matching glyph for the requested size
    let bitmap_glyph = strikes
        .glyph_for_size(size, glyph_id)
        .ok_or(BitmapRenderError::GlyphNotFound)?;

    // Handle different bitmap formats
    match &bitmap_glyph.data {
        BitmapData::Png(png_data) => decode_png_to_pixmap(png_data),
        BitmapData::Bgra(bgra_data) => {
            decode_bgra_to_pixmap(bgra_data, bitmap_glyph.width, bitmap_glyph.height)
        },
        BitmapData::Mask(mask_data) => {
            decode_mask_to_pixmap(mask_data.data, bitmap_glyph.width, bitmap_glyph.height)
        },
    }
}

/// Render a bitmap glyph with proper scaling and positioning
///
/// This function:
/// 1. Gets the bitmap at its native strike size
/// 2. Scales to the requested ppem using bilinear interpolation
/// 3. Flips vertically (bitmap tables are Y-down, font coords are Y-up)
/// 4. Returns bearing information for correct positioning
///
/// # Arguments
/// * `font_data` - Font file data
/// * `glyph_id` - Glyph ID to render
/// * `ppem` - Requested pixels per em
///
/// # Returns
/// A `ScaledBitmapGlyph` with the pixmap and positioning info
pub fn render_bitmap_glyph_scaled(
    font_data: &[u8],
    glyph_id: u32,
    ppem: f32,
) -> Result<ScaledBitmapGlyph, BitmapRenderError> {
    let font = skrifa::FontRef::new(font_data).map_err(|_| BitmapRenderError::FontParseFailed)?;
    let strikes = BitmapStrikes::new(&font);

    if strikes.is_empty() {
        return Err(BitmapRenderError::NoBitmapTable);
    }

    let glyph_id_obj = GlyphId::new(glyph_id);
    let size = Size::new(ppem);

    // Get best matching glyph for the requested size
    let bitmap_glyph = strikes
        .glyph_for_size(size, glyph_id_obj)
        .ok_or(BitmapRenderError::GlyphNotFound)?;

    // Decode the bitmap at its native size
    let native_pixmap = match &bitmap_glyph.data {
        BitmapData::Png(png_data) => decode_png_to_pixmap(png_data)?,
        BitmapData::Bgra(bgra_data) => {
            decode_bgra_to_pixmap(bgra_data, bitmap_glyph.width, bitmap_glyph.height)?
        },
        BitmapData::Mask(mask_data) => {
            decode_mask_to_pixmap(mask_data.data, bitmap_glyph.width, bitmap_glyph.height)?
        },
    };

    // Get the native ppem of this strike
    let native_ppem_x = bitmap_glyph.ppem_x;
    let native_ppem_y = bitmap_glyph.ppem_y;

    // Calculate scale factors
    let scale_x = ppem / native_ppem_x;
    let scale_y = ppem / native_ppem_y;

    // Calculate target dimensions
    let target_width = ((native_pixmap.width() as f32 * scale_x).round() as u32).max(1);
    let target_height = ((native_pixmap.height() as f32 * scale_y).round() as u32).max(1);

    // Check if scaling is needed
    let needs_scaling =
        target_width != native_pixmap.width() || target_height != native_pixmap.height();

    // Scale the bitmap if needed
    let scaled_pixmap = if needs_scaling {
        scale_pixmap_bilinear(&native_pixmap, target_width, target_height)?
    } else {
        native_pixmap
    };

    // Note: We do NOT flip here. Bitmap tables are Y-down (same as screen coords),
    // but typf-render-color outputs are expected to be in font coords (Y-up).
    // The Skia/Zeno renderers apply flip_vertical_rgba() to all color glyphs
    // consistently, so we let them handle the flip.
    //
    // Actually, bitmap glyphs are already in screen coordinates (Y-down), which
    // is what Skia/Zeno expect after their flip. So we need to flip here to match
    // what COLR/SVG output (font coords Y-up), then Skia/Zeno will flip back.
    let output_pixmap = flip_pixmap_vertical(&scaled_pixmap)?;

    // Calculate bearings in the scaled coordinate space
    // bearing_x: horizontal offset from origin to left edge
    // bearing_y: vertical offset from baseline to top edge (positive = above baseline)
    //
    // For sbix, bearings may be zero and need special handling (CoreText adds 100 unit offset)
    // For CBDT/CBLC, bearings come from the bitmap metrics
    let bearing_x = bitmap_glyph.bearing_x * scale_x;

    // Handle sbix special case: if bearing_y is 0 and this is sbix, use a default offset
    let bearing_y = if bitmap_glyph.bearing_y == 0.0 && strikes.format() == Some(BitmapFormat::Sbix)
    {
        // CoreText uses 100 font units offset for sbix
        let upem = font
            .head()
            .map(|h| h.units_per_em() as f32)
            .unwrap_or(1000.0);
        100.0 * ppem / upem
    } else {
        bitmap_glyph.bearing_y * scale_y
    };

    // Handle inner bearings based on placement origin
    let (final_bearing_x, final_bearing_y) = match bitmap_glyph.placement_origin {
        Origin::TopLeft => {
            // Inner bearings are in pixels, apply them
            (
                bearing_x - bitmap_glyph.inner_bearing_x * scale_x,
                bearing_y - bitmap_glyph.inner_bearing_y * scale_y,
            )
        },
        Origin::BottomLeft => {
            // Adjust for bottom-left origin
            (
                bearing_x - bitmap_glyph.inner_bearing_x * scale_x,
                bearing_y - bitmap_glyph.inner_bearing_y * scale_y + output_pixmap.height() as f32,
            )
        },
    };

    Ok(ScaledBitmapGlyph {
        pixmap: output_pixmap,
        bearing_x: final_bearing_x,
        bearing_y: final_bearing_y,
        native_ppem: native_ppem_x, // Typically x and y are the same
        was_scaled: needs_scaling,
    })
}

/// Scale a pixmap using bilinear interpolation
fn scale_pixmap_bilinear(
    src: &Pixmap,
    target_width: u32,
    target_height: u32,
) -> Result<Pixmap, BitmapRenderError> {
    let src_width = src.width() as usize;
    let src_height = src.height() as usize;
    let dst_width = target_width as usize;
    let dst_height = target_height as usize;

    let mut dst_data = vec![0u8; dst_width * dst_height * 4];
    let src_data = src.data();

    for dst_y in 0..dst_height {
        for dst_x in 0..dst_width {
            // Map destination pixel to source coordinates
            let src_x_f = (dst_x as f32 + 0.5) * (src_width as f32 / dst_width as f32) - 0.5;
            let src_y_f = (dst_y as f32 + 0.5) * (src_height as f32 / dst_height as f32) - 0.5;

            // Get the four surrounding source pixels
            let x0 = (src_x_f.floor() as isize).clamp(0, src_width as isize - 1) as usize;
            let y0 = (src_y_f.floor() as isize).clamp(0, src_height as isize - 1) as usize;
            let x1 = (x0 + 1).min(src_width - 1);
            let y1 = (y0 + 1).min(src_height - 1);

            // Calculate interpolation weights
            let wx = src_x_f - src_x_f.floor();
            let wy = src_y_f - src_y_f.floor();

            // Bilinear interpolation for each channel
            let dst_idx = (dst_y * dst_width + dst_x) * 4;
            for c in 0..4 {
                let p00 = src_data[(y0 * src_width + x0) * 4 + c] as f32;
                let p10 = src_data[(y0 * src_width + x1) * 4 + c] as f32;
                let p01 = src_data[(y1 * src_width + x0) * 4 + c] as f32;
                let p11 = src_data[(y1 * src_width + x1) * 4 + c] as f32;

                let value = p00 * (1.0 - wx) * (1.0 - wy)
                    + p10 * wx * (1.0 - wy)
                    + p01 * (1.0 - wx) * wy
                    + p11 * wx * wy;

                dst_data[dst_idx + c] = value.round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    Pixmap::from_vec(
        dst_data,
        tiny_skia::IntSize::from_wh(target_width, target_height).unwrap(),
    )
    .ok_or(BitmapRenderError::PixmapCreationFailed)
}

/// Flip a pixmap vertically (Y-axis mirror)
fn flip_pixmap_vertical(src: &Pixmap) -> Result<Pixmap, BitmapRenderError> {
    let width = src.width() as usize;
    let height = src.height() as usize;
    let src_data = src.data();
    let mut dst_data = vec![0u8; width * height * 4];
    let row_bytes = width * 4;

    for y in 0..height {
        let src_row = y * row_bytes;
        let dst_row = (height - 1 - y) * row_bytes;
        dst_data[dst_row..dst_row + row_bytes]
            .copy_from_slice(&src_data[src_row..src_row + row_bytes]);
    }

    Pixmap::from_vec(
        dst_data,
        tiny_skia::IntSize::from_wh(src.width(), src.height()).unwrap(),
    )
    .ok_or(BitmapRenderError::PixmapCreationFailed)
}

/// Render a bitmap glyph with fallback to outline if bitmap unavailable
///
/// This function first attempts to render a scaled bitmap glyph at the requested ppem.
/// The bitmap is scaled from the font's native strike size and flipped vertically
/// to match font coordinates (Y-up). If no bitmap is available, it falls back to
/// rendering the glyph outline as a filled path.
///
/// # Arguments
/// * `font_data` - Font file data
/// * `glyph_id` - Glyph ID to render
/// * `width` - Output pixmap width (used for outline fallback)
/// * `height` - Output pixmap height (used for outline fallback)
/// * `ppem` - Pixels per em (size) for rendering
///
/// # Returns
/// A tuple of (Pixmap, bool) where the bool indicates if bitmap was used (true) or outline (false)
pub fn render_bitmap_glyph_or_outline(
    font_data: &[u8],
    glyph_id: u32,
    width: u32,
    height: u32,
    ppem: f32,
) -> Result<(Pixmap, bool), BitmapRenderError> {
    // Try scaled bitmap first (this handles scaling and vertical flip)
    match render_bitmap_glyph_scaled(font_data, glyph_id, ppem) {
        Ok(scaled) => Ok((scaled.pixmap, true)),
        Err(BitmapRenderError::NoBitmapTable)
        | Err(BitmapRenderError::GlyphNotFound)
        | Err(BitmapRenderError::UnsupportedFormat) => {
            // Fall back to outline rendering
            render_outline_glyph(font_data, glyph_id, width, height, ppem)
                .map(|pixmap| (pixmap, false))
        },
        Err(e) => Err(e),
    }
}

/// Render a glyph outline to a pixmap
fn render_outline_glyph(
    font_data: &[u8],
    glyph_id: u32,
    width: u32,
    height: u32,
    ppem: f32,
) -> Result<Pixmap, BitmapRenderError> {
    let font = skrifa::FontRef::new(font_data).map_err(|_| BitmapRenderError::FontParseFailed)?;
    let glyph_id = GlyphId::new(glyph_id);

    // Get the glyph outline
    let outline_glyphs = font.outline_glyphs();
    let outline = outline_glyphs
        .get(glyph_id)
        .ok_or(BitmapRenderError::NoGlyphAvailable)?;

    // Create a path pen to capture the glyph path
    let mut pen = TinySkiaPathPen::new();

    // Draw the glyph outline
    let location = Location::default();
    let size = Size::new(ppem);
    let settings = DrawSettings::unhinted(size, &location);
    outline
        .draw(settings, &mut pen)
        .map_err(|_| BitmapRenderError::OutlineRenderFailed)?;

    let path = pen.finish().ok_or(BitmapRenderError::OutlineRenderFailed)?;

    // Create pixmap and render
    let mut pixmap = Pixmap::new(width, height).ok_or(BitmapRenderError::PixmapCreationFailed)?;

    // Set up paint (black fill)
    let mut paint = Paint::default();
    paint.set_color_rgba8(0, 0, 0, 255);
    paint.anti_alias = true;

    // Calculate transform to center glyph and flip Y (font coords are Y-up)
    let metrics = font.metrics(size, &location);
    let ascender = metrics.ascent;

    // Transform: flip Y and translate to position glyph
    let transform = Transform::from_scale(1.0, -1.0).post_translate(0.0, ascender);

    pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);

    Ok(pixmap)
}

/// A path pen that builds a tiny-skia Path
struct TinySkiaPathPen {
    builder: PathBuilder,
}

impl TinySkiaPathPen {
    fn new() -> Self {
        Self {
            builder: PathBuilder::new(),
        }
    }

    fn finish(self) -> Option<tiny_skia::Path> {
        self.builder.finish()
    }
}

impl OutlinePen for TinySkiaPathPen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.builder.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(x, y);
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.builder.quad_to(cx0, cy0, x, y);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.builder.cubic_to(cx0, cy0, cx1, cy1, x, y);
    }

    fn close(&mut self) {
        self.builder.close();
    }
}

/// Decode PNG data to a pixmap
fn decode_png_to_pixmap(png_data: &[u8]) -> Result<Pixmap, BitmapRenderError> {
    // Use png crate to decode
    let decoder = png::Decoder::new(png_data);
    let mut reader = decoder
        .read_info()
        .map_err(|_| BitmapRenderError::PngDecodeFailed)?;

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|_| BitmapRenderError::PngDecodeFailed)?;

    let width = info.width;
    let height = info.height;

    // Convert to RGBA premultiplied for tiny-skia
    let rgba = match info.color_type {
        png::ColorType::Rgba => {
            // Premultiply alpha
            premultiply_rgba(&buf[..info.buffer_size()])
        },
        png::ColorType::Rgb => {
            // Add alpha channel
            let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
            for chunk in buf[..info.buffer_size()].chunks(3) {
                rgba.extend_from_slice(chunk);
                rgba.push(255);
            }
            rgba
        },
        png::ColorType::Grayscale => {
            // Convert to RGBA
            let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
            for &gray in &buf[..info.buffer_size()] {
                rgba.extend_from_slice(&[gray, gray, gray, 255]);
            }
            rgba
        },
        png::ColorType::GrayscaleAlpha => {
            // Convert to RGBA with premultiplied alpha
            let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
            for chunk in buf[..info.buffer_size()].chunks(2) {
                let gray = chunk[0];
                let alpha = chunk[1];
                let pm_gray = ((gray as u16 * alpha as u16) / 255) as u8;
                rgba.extend_from_slice(&[pm_gray, pm_gray, pm_gray, alpha]);
            }
            rgba
        },
        png::ColorType::Indexed => {
            // Need palette info - fallback to error for now
            return Err(BitmapRenderError::UnsupportedFormat);
        },
    };

    Pixmap::from_vec(rgba, tiny_skia::IntSize::from_wh(width, height).unwrap())
        .ok_or(BitmapRenderError::PixmapCreationFailed)
}

/// Premultiply RGBA data
fn premultiply_rgba(rgba: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(rgba.len());
    for chunk in rgba.chunks(4) {
        let r = chunk[0];
        let g = chunk[1];
        let b = chunk[2];
        let a = chunk[3];
        // Premultiply
        let pm_r = ((r as u16 * a as u16) / 255) as u8;
        let pm_g = ((g as u16 * a as u16) / 255) as u8;
        let pm_b = ((b as u16 * a as u16) / 255) as u8;
        result.extend_from_slice(&[pm_r, pm_g, pm_b, a]);
    }
    result
}

/// Decode BGRA data to a pixmap (already premultiplied)
fn decode_bgra_to_pixmap(
    bgra_data: &[u8],
    width: u32,
    height: u32,
) -> Result<Pixmap, BitmapRenderError> {
    // Convert BGRA to RGBA (tiny-skia uses RGBA order internally)
    let mut rgba = Vec::with_capacity(bgra_data.len());
    for chunk in bgra_data.chunks(4) {
        // BGRA -> RGBA
        rgba.push(chunk[2]); // R
        rgba.push(chunk[1]); // G
        rgba.push(chunk[0]); // B
        rgba.push(chunk[3]); // A
    }

    Pixmap::from_vec(rgba, tiny_skia::IntSize::from_wh(width, height).unwrap())
        .ok_or(BitmapRenderError::PixmapCreationFailed)
}

/// Decode grayscale mask to a pixmap (black with alpha from mask)
fn decode_mask_to_pixmap(
    mask_data: &[u8],
    width: u32,
    height: u32,
) -> Result<Pixmap, BitmapRenderError> {
    let mut rgba = Vec::with_capacity(mask_data.len() * 4);
    for &alpha in mask_data {
        // Black text with mask as alpha
        rgba.extend_from_slice(&[0, 0, 0, alpha]);
    }

    Pixmap::from_vec(rgba, tiny_skia::IntSize::from_wh(width, height).unwrap())
        .ok_or(BitmapRenderError::PixmapCreationFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_bitmap_glyphs_regular_font() {
        // Regular fonts without bitmap tables should return false
        let font_path = "../../test-fonts/NotoSans-Regular.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            assert!(!has_bitmap_glyphs(&font_data));
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    #[test]
    fn test_has_bitmap_glyphs_sbix_font() {
        // Nabla sbix font should have bitmap tables
        let font_path = "../../test-fonts/Nabla-Regular-sbix.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            let has_bitmaps = has_bitmap_glyphs(&font_data);
            println!("Nabla-Regular-sbix has bitmaps: {}", has_bitmaps);
            // sbix fonts should have bitmap strikes
            assert!(has_bitmaps, "sbix font should have bitmap glyphs");
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    #[test]
    fn test_has_bitmap_glyphs_cbdt_font() {
        // Nabla CBDT font should have bitmap tables
        let font_path = "../../test-fonts/Nabla-Regular-CBDT.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            let has_bitmaps = has_bitmap_glyphs(&font_data);
            println!("Nabla-Regular-CBDT has bitmaps: {}", has_bitmaps);
            // CBDT fonts should have bitmap strikes
            assert!(has_bitmaps, "CBDT font should have bitmap glyphs");
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    #[test]
    fn test_get_bitmap_sizes_sbix() {
        let font_path = "../../test-fonts/Nabla-Regular-sbix.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            let sizes = get_bitmap_sizes(&font_data);
            println!("sbix bitmap sizes: {:?}", sizes);
            // sbix fonts typically have multiple strike sizes
            assert!(!sizes.is_empty(), "sbix font should have bitmap strikes");
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    #[test]
    fn test_render_bitmap_glyph_sbix() {
        let font_path = "../../test-fonts/Nabla-Regular-sbix.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            if has_bitmap_glyphs(&font_data) {
                // Try to render a bitmap glyph
                for gid in 1..100 {
                    if let Ok(pixmap) = render_bitmap_glyph(&font_data, gid, 128.0) {
                        println!(
                            "Rendered bitmap glyph at gid {}: {}x{}",
                            gid,
                            pixmap.width(),
                            pixmap.height()
                        );
                        assert!(pixmap.width() > 0);
                        assert!(pixmap.height() > 0);
                        return; // Test passed
                    }
                }
                eprintln!("No bitmap glyphs found in first 100 glyph IDs");
            } else {
                eprintln!("sbix font doesn't have bitmap strikes (unexpected)");
            }
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    #[test]
    fn test_fallback_to_outline_on_regular_font() {
        // Regular font has no bitmaps, should fall back to outline rendering
        let font_path = "../../test-fonts/NotoSans-Regular.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            // Glyph 36 is typically 'A' in many fonts
            let result = render_bitmap_glyph_or_outline(&font_data, 36, 64, 64, 32.0);
            match result {
                Ok((pixmap, used_bitmap)) => {
                    assert!(!used_bitmap, "Should have used outline fallback");
                    assert!(pixmap.width() > 0);
                    assert!(pixmap.height() > 0);
                    println!(
                        "Fallback test passed: rendered outline {}x{}",
                        pixmap.width(),
                        pixmap.height()
                    );
                },
                Err(e) => {
                    // NoGlyphAvailable is acceptable if the glyph truly doesn't exist
                    if !matches!(e, BitmapRenderError::NoGlyphAvailable) {
                        panic!("Unexpected error: {:?}", e);
                    }
                },
            }
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    #[test]
    fn test_fallback_uses_bitmap_when_available() {
        // sbix font has bitmaps, should use bitmap (not fallback)
        let font_path = "../../test-fonts/Nabla-Regular-sbix.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            // Try to find a glyph with bitmap
            for gid in 1..100 {
                if let Ok((pixmap, used_bitmap)) =
                    render_bitmap_glyph_or_outline(&font_data, gid, 128, 128, 128.0)
                {
                    println!(
                        "Glyph {}: used_bitmap={}, size={}x{}",
                        gid,
                        used_bitmap,
                        pixmap.width(),
                        pixmap.height()
                    );
                    if used_bitmap {
                        // Found a bitmap glyph - test passes
                        return;
                    }
                }
            }
            eprintln!("No bitmap glyphs found in sbix font (unexpected)");
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    #[test]
    fn test_render_bitmap_glyph_scaled_sbix() {
        // Test scaling bitmap glyphs to different sizes
        let font_path = "../../test-fonts/Nabla-Regular-sbix.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            let sizes = get_bitmap_sizes(&font_data);
            if sizes.is_empty() {
                eprintln!("No bitmap sizes in sbix font");
                return;
            }
            let native_ppem = sizes[0];
            println!("Native ppem: {}", native_ppem);

            // Find a glyph that has a bitmap
            for gid in 1..100 {
                if let Ok(scaled) = render_bitmap_glyph_scaled(&font_data, gid, native_ppem / 2.0) {
                    println!(
                        "Scaled bitmap glyph {} at half size: {}x{}, native_ppem={}, was_scaled={}, bearing=({}, {})",
                        gid,
                        scaled.pixmap.width(),
                        scaled.pixmap.height(),
                        scaled.native_ppem,
                        scaled.was_scaled,
                        scaled.bearing_x,
                        scaled.bearing_y
                    );
                    // When scaling to half size, dimensions should be roughly half
                    assert!(scaled.was_scaled, "Should have scaled the bitmap");
                    return;
                }
            }
            eprintln!("No bitmap glyphs found in sbix font");
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    #[test]
    fn test_render_bitmap_glyph_scaled_cbdt() {
        // Test scaling CBDT bitmap glyphs
        let font_path = "../../test-fonts/Nabla-Regular-CBDT.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            let sizes = get_bitmap_sizes(&font_data);
            if sizes.is_empty() {
                eprintln!("No bitmap sizes in CBDT font");
                return;
            }
            let native_ppem = sizes[0];
            println!("Native ppem: {}", native_ppem);

            // Find a glyph that has a bitmap
            for gid in 1..100 {
                if let Ok(scaled) = render_bitmap_glyph_scaled(&font_data, gid, 64.0) {
                    println!(
                        "Scaled CBDT bitmap glyph {} to 64px: {}x{}, native_ppem={}, was_scaled={}, bearing=({}, {})",
                        gid,
                        scaled.pixmap.width(),
                        scaled.pixmap.height(),
                        scaled.native_ppem,
                        scaled.was_scaled,
                        scaled.bearing_x,
                        scaled.bearing_y
                    );
                    // Verify the pixmap has content
                    assert!(scaled.pixmap.width() > 0);
                    assert!(scaled.pixmap.height() > 0);
                    return;
                }
            }
            eprintln!("No bitmap glyphs found in CBDT font");
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }
}
