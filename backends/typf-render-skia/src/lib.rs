//! Skia Renderer - Professional-grade rasterization via tiny-skia
//!
//! When you need production-quality text rendering, Skia delivers.
//! This backend transforms font outlines into crisp anti-aliased bitmaps
//! using the same path rendering tech that powers Chrome and Android.
//!
//! ## What Makes Skia Special
//!
//! - Sub-pixel precision that makes text readable at any size
//! - True vector path rendering with proper Bézier curve handling
//! - Winding fill rules that match font designer expectations
//! - Clean alpha extraction for perfect compositing
//!
//! Crafted with care by FontLab - https://www.fontlab.org/

use kurbo::Shape;
use skrifa::MetadataProvider;
use std::sync::Arc;
use typf_core::{
    error::{RenderError, Result},
    traits::{FontRef, Renderer},
    types::{BitmapData, BitmapFormat, RenderOutput, ShapingResult, VectorFormat},
    GlyphSource, GlyphSourcePreference, RenderMode, RenderParams,
};
use typf_render_color::{compute_content_bounds, render_glyph_with_preference};
use typf_render_svg::SvgRenderer;

/// tiny-skia powered renderer for pristine glyph output
///
/// This isn't just another bitmap renderer—it's a precision instrument
/// that extracts glyph outlines and renders them using industry-proven
/// algorithms. Perfect when quality matters more than raw speed.
pub struct SkiaRenderer {
    /// Maximum canvas dimension to prevent memory exhaustion
    /// Keeps even the most ambitious rendering jobs within bounds
    max_size: u32,
}

impl SkiaRenderer {
    /// Creates a renderer that treats every glyph with professional care
    pub fn new() -> Self {
        Self { max_size: 65535 }
    }

    /// Converts a single glyph from outline to bitmap with surgical precision
    ///
    /// This method extracts the glyph outline using skrifa, builds a path,
    /// and renders it with tiny-skia's advanced anti-aliasing. The result
    /// is a clean alpha bitmap ready for compositing.
    fn render_glyph(
        &self,
        font: &Arc<dyn FontRef>,
        glyph_id: u32,
        font_size: f32,
        location: &skrifa::instance::Location,
        params: &RenderParams,
    ) -> Result<GlyphBitmap> {
        use kurbo::{BezPath, PathEl};
        use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

        // Pull raw font data for skrifa to parse
        let font_data = font.data();
        let font_ref = skrifa::FontRef::new(font_data).map_err(|_| RenderError::InvalidFont)?;
        let color_allowed = allows_color_sources(&params.glyph_sources);

        // Navigate to the outline glyph collection
        let outlines = font_ref.outline_glyphs();
        // Use GlyphId::new to support full u32 range (>65k glyph IDs)
        let glyph_id = skrifa::GlyphId::new(glyph_id);

        // Find the specific glyph we need to render
        // For bitmap-only fonts (like CBDT), glyphs may not have outlines
        let glyph = outlines.get(glyph_id);
        let has_outline = glyph.is_some();

        // If no outline and color sources are allowed, try bitmap/color rendering first
        if !has_outline && color_allowed {
            let fallback_size = font_size.max(1.0);
            let width = fallback_size.ceil() as u32;
            let height = fallback_size.ceil() as u32;
            let bbox = kurbo::Rect::new(0.0, 0.0, fallback_size as f64, fallback_size as f64);

            if let Some(color_bitmap) = self.try_color_glyph(
                font,
                glyph_id.to_u32(),
                width,
                height,
                font_size,
                &bbox,
                params,
            )? {
                return Ok(color_bitmap);
            }
        }

        // Now require outline for non-color rendering
        let glyph = glyph.ok_or_else(|| RenderError::GlyphNotFound(glyph_id.to_u32()))?;

        // Build a kurbo path from the glyph's outline data
        let mut path = BezPath::new();
        // skrifa's DrawSettings handles the tricky font-unit-to-pixel scaling
        // for us, so our PathPen can stay simple and focused
        let mut pen = PathPen {
            path: &mut path,
            scale: 1.0, // skrifa does the heavy lifting on scaling
        };

        // Request unhinted outlines at the exact size we need
        let size = skrifa::instance::Size::new(font_size);
        // Use provided location for variable font support
        let settings = skrifa::outline::DrawSettings::unhinted(size, location.coords());

        // Trace the glyph outline into our kurbo path
        glyph
            .draw(settings, &mut pen)
            .map_err(|_| RenderError::OutlineExtractionFailed)?;

        // Figure out how much canvas space this glyph needs
        let mut bbox = path.bounding_box();

        let outline_empty = bbox.width() == 0.0 || bbox.height() == 0.0;

        // Guard against malformed glyphs that could crash the renderer
        if bbox.x0.is_infinite()
            || bbox.y0.is_infinite()
            || bbox.x1.is_infinite()
            || bbox.y1.is_infinite()
        {
            return Err(RenderError::PathBuildingFailed.into());
        }
        if outline_empty && color_allowed {
            let fallback = font_size.max(1.0) as f64;
            bbox = kurbo::Rect::new(0.0, 0.0, fallback, fallback);
        } else if bbox.width() == 0.0 || bbox.height() == 0.0 {
            return Err(RenderError::InvalidDimensions {
                width: bbox.width() as u32,
                height: bbox.height() as u32,
            }
            .into());
        }

        // Ensure we always have at least 1x1 pixels for rendering
        let width = (bbox.width().ceil() as u32).max(1);
        let height = (bbox.height().ceil() as u32).max(1);

        log::debug!(
            "Skia: glyph_id={}, bbox=({}, {}, {}, {}), size={}x{}",
            glyph_id,
            bbox.x0,
            bbox.y0,
            bbox.x1,
            bbox.y1,
            width,
            height
        );

        // Prefer color/SVG/bitmap glyph sources when requested
        // BUT: If outline is empty, skip COLR sources. COLR glyphs are based on outlines,
        // so an empty outline means no actual content - just a bounding box fill.
        // This prevents space characters from rendering as colored squares.
        if color_allowed && !outline_empty {
            if let Some(color_bitmap) = self.try_color_glyph(
                font,
                glyph_id.to_u32(),
                width,
                height,
                font_size,
                &bbox,
                params,
            )? {
                return Ok(color_bitmap);
            }
        }

        let outline_allowed = params
            .glyph_sources
            .effective_order()
            .iter()
            .any(|s| matches!(s, GlyphSource::Glyf | GlyphSource::Cff | GlyphSource::Cff2));
        if !outline_allowed {
            return Err(RenderError::BackendError(
                "outline glyph sources disabled and no color glyph available".to_string(),
            )
            .into());
        }

        // Translate kurbo's path format into tiny-skia's native format
        let mut builder = PathBuilder::new();
        for element in path.elements() {
            match *element {
                PathEl::MoveTo(p) => builder.move_to(p.x as f32, p.y as f32),
                PathEl::LineTo(p) => builder.line_to(p.x as f32, p.y as f32),
                PathEl::QuadTo(ctrl, end) => {
                    builder.quad_to(ctrl.x as f32, ctrl.y as f32, end.x as f32, end.y as f32)
                },
                PathEl::CurveTo(c1, c2, end) => builder.cubic_to(
                    c1.x as f32,
                    c1.y as f32,
                    c2.x as f32,
                    c2.y as f32,
                    end.x as f32,
                    end.y as f32,
                ),
                PathEl::ClosePath => builder.close(),
            }
        }

        let skia_path = builder.finish().ok_or(RenderError::PathBuildingFailed)?;

        // Create our rendering surface
        let mut pixmap = Pixmap::new(width, height).ok_or(RenderError::PixmapCreationFailed)?;

        // Set up painter with anti-aliasing for smooth edges
        let paint = Paint {
            anti_alias: true,
            ..Default::default()
        };

        // Critical coordinate transform:
        // 1. Flip Y (fonts use y-up, bitmaps use y-down)
        // 2. Shift so bbox fits perfectly in our pixmap
        let transform =
            Transform::from_scale(1.0, -1.0).post_translate(-bbox.x0 as f32, bbox.y1 as f32);

        // Render the filled path to our pixmap
        pixmap.fill_path(&skia_path, &paint, FillRule::Winding, transform, None);

        // Extract just the alpha channel (tiny-skia gives us RGBA, we need grayscale)
        let data = pixmap.data();
        let mut alpha = vec![0u8; (width * height) as usize];
        for i in 0..(width * height) as usize {
            alpha[i] = data[i * 4 + 3]; // Alpha lives in channel 4
        }

        // Return positioning info so the glyph lands in the right place
        // bearing_x: how far from origin the leftmost pixel appears
        // bearing_y: how far above baseline the topmost pixel appears
        Ok(GlyphBitmap {
            width,
            height,
            data: GlyphBitmapData::Mask(alpha),
            bearing_x: bbox.x0.floor() as i32,
            bearing_y: bbox.y1.ceil() as i32,
        })
    }

    /// Attempt to render a color/SVG/bitmap glyph when requested.
    ///
    /// Returns `Ok(Some(...))` if a color glyph was successfully rendered,
    /// `Ok(None)` if no color glyph is available (allowing outline fallback),
    /// or `Err(...)` for actual rendering failures.
    #[allow(clippy::too_many_arguments)]
    fn try_color_glyph(
        &self,
        font: &Arc<dyn FontRef>,
        glyph_id: u32,
        width: u32,
        height: u32,
        font_size: f32,
        bbox: &kurbo::Rect,
        params: &RenderParams,
    ) -> Result<Option<GlyphBitmap>> {
        if width == 0 || height == 0 {
            return Ok(None);
        }

        let variations: Vec<(&str, f32)> = params
            .variations
            .iter()
            .map(|(tag, value)| (tag.as_str(), *value))
            .collect();

        match render_glyph_with_preference(
            font.data(),
            glyph_id,
            width,
            height,
            font_size,
            params.color_palette,
            &variations,
            &params.glyph_sources,
        ) {
            Ok((rendered, source_used)) => {
                let pixmap = rendered.pixmap;
                let pixmap_data = pixmap.data();

                // Check for fully-transparent color glyphs (spaces, empty glyphs)
                // to avoid compositing black squares
                if is_fully_transparent(pixmap_data) {
                    log::debug!(
                        "Skia: color glyph {} via {:?} is fully transparent, skipping",
                        glyph_id,
                        source_used
                    );
                    return Ok(None);
                }

                log::debug!(
                    "Skia: rendered glyph {} via {:?} into {}x{}",
                    glyph_id,
                    source_used,
                    pixmap.width(),
                    pixmap.height()
                );

                // Use bearing info from RenderResult if available (bitmap glyphs),
                // otherwise compute from actual pixmap content bounds (COLR/SVG)
                let (bearing_x, bearing_y) = if let (Some(bx), Some(by)) =
                    (rendered.bearing_x, rendered.bearing_y)
                {
                    // Bitmap glyphs: use computed bearings from font metrics
                    (bx.floor() as i32, by.ceil() as i32)
                } else {
                    // COLR/SVG: compute bearings from actual rendered content
                    // This ensures vertical positioning matches the actual color glyph content,
                    // not the outline bbox which may differ.
                    if let Some(bounds) = compute_content_bounds(&pixmap) {
                        // Content bounds are in pixmap coords (Y-down, origin at top-left)
                        // The pixmap is positioned at (bbox.x0, bbox.y0) in font coords
                        // bearing_x = left edge of content in font coords
                        // bearing_y = top edge of content in font coords (Y-up)
                        //
                        // In the pixmap (before flip):
                        // - min_y is the topmost row with content
                        // - This corresponds to the highest Y value in font coords
                        //
                        // Font coords Y range: [bbox.y0, bbox.y1]
                        // Pixmap row 0 = bbox.y1 (top of bbox in font coords)
                        // Pixmap row (height-1) = bbox.y0 (bottom of bbox in font coords)
                        //
                        // So: font_y = bbox.y1 - pixmap_y * (bbox.y1 - bbox.y0) / (height - 1)
                        // For the topmost content (min_y in pixmap):
                        // font_y_top = bbox.y1 - min_y * height_scale
                        let height = pixmap.height() as f64;
                        let height_scale = if height > 1.0 {
                            (bbox.y1 - bbox.y0) / (height - 1.0)
                        } else {
                            1.0
                        };
                        let content_top_font_y = bbox.y1 - (bounds.min_y as f64) * height_scale;

                        // bearing_x: left edge of content
                        let width = pixmap.width() as f64;
                        let width_scale = if width > 1.0 {
                            (bbox.x1 - bbox.x0) / (width - 1.0)
                        } else {
                            1.0
                        };
                        let content_left_font_x = bbox.x0 + (bounds.min_x as f64) * width_scale;

                        (content_left_font_x.floor() as i32, content_top_font_y.ceil() as i32)
                    } else {
                        // Fully transparent - use outline bbox as fallback
                        (bbox.x0.floor() as i32, bbox.y1.ceil() as i32)
                    }
                };

                // Flip vertically: typf-render-color outputs in font coords (Y-up),
                // but we need bitmap coords (Y-down) for compositing
                let mut rgba_data = pixmap_data.to_vec();
                flip_vertical_rgba(&mut rgba_data, pixmap.width(), pixmap.height());

                Ok(Some(GlyphBitmap {
                    width: pixmap.width(),
                    height: pixmap.height(),
                    data: GlyphBitmapData::RgbaPremul(rgba_data),
                    bearing_x,
                    bearing_y,
                }))
            },
            Err(typf_render_color::ColorRenderError::GlyphNotFound) => {
                // No color glyph available - allow outline fallback
                log::debug!("Skia: no color glyph for {}, falling back to outline", glyph_id);
                Ok(None)
            },
            Err(typf_render_color::ColorRenderError::NoColrTable) => {
                // Font has no COLR table - allow outline fallback
                Ok(None)
            },
            Err(typf_render_color::ColorRenderError::NoPalette) => {
                // No palette available - allow outline fallback
                log::debug!("Skia: no palette for glyph {}, falling back to outline", glyph_id);
                Ok(None)
            },
            Err(err) => {
                // Actual rendering error (pixmap creation failed, paint error, etc.)
                Err(RenderError::BackendError(format!(
                    "color glyph {} render failed: {:?}",
                    glyph_id, err
                ))
                .into())
            },
        }
    }
}

impl Default for SkiaRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Flip a bitmap vertically (convert between Y-up and Y-down coordinate systems)
///
/// Color glyphs from typf-render-color are rendered in font coordinate space (Y-up),
/// but we need them in bitmap coordinate space (Y-down) for compositing.
fn flip_vertical_rgba(data: &mut [u8], width: u32, height: u32) {
    let row_bytes = (width * 4) as usize;
    for y in 0..(height / 2) {
        let top_start = y as usize * row_bytes;
        let bottom_start = (height - 1 - y) as usize * row_bytes;
        for x in 0..row_bytes {
            data.swap(top_start + x, bottom_start + x);
        }
    }
}

/// Check if a premultiplied RGBA buffer is fully transparent
fn is_fully_transparent(data: &[u8]) -> bool {
    data.chunks_exact(4).all(|px| px[3] == 0)
}

/// Whether preference allows any color/bitmap/SVG sources.
fn allows_color_sources(pref: &GlyphSourcePreference) -> bool {
    pref.effective_order().iter().any(|s| {
        matches!(
            s,
            GlyphSource::Colr0
                | GlyphSource::Colr1
                | GlyphSource::Svg
                | GlyphSource::Sbix
                | GlyphSource::Cbdt
                | GlyphSource::Ebdt
        )
    })
}

/// Build variation location from params
fn build_location(
    font: &Arc<dyn FontRef>,
    variations: &[(String, f32)],
) -> skrifa::instance::Location {
    if variations.is_empty() {
        return skrifa::instance::Location::default();
    }

    let font_data = font.data();
    let font_ref = match skrifa::FontRef::new(font_data) {
        Ok(f) => f,
        Err(_) => return skrifa::instance::Location::default(),
    };

    let axes = font_ref.axes();
    let settings: Vec<(&str, f32)> = variations
        .iter()
        .map(|(tag, value)| (tag.as_str(), *value))
        .collect();

    axes.location(settings)
}

impl Renderer for SkiaRenderer {
    fn name(&self) -> &'static str {
        "skia"
    }

    fn render(
        &self,
        shaped: &ShapingResult,
        font: Arc<dyn FontRef>,
        params: &RenderParams,
    ) -> Result<RenderOutput> {
        let allows_outline = params
            .glyph_sources
            .effective_order()
            .iter()
            .any(|s| matches!(s, GlyphSource::Glyf | GlyphSource::Cff | GlyphSource::Cff2));
        let allows_color = allows_color_sources(&params.glyph_sources);
        if !allows_outline && !allows_color {
            return Err(RenderError::BackendError(
                "skia renderer requires outline or color glyph sources".to_string(),
            )
            .into());
        }

        // Vector mode: delegate to the SVG renderer for path extraction
        if let RenderMode::Vector(vector_format) = params.output {
            if vector_format == VectorFormat::Svg {
                let svg_renderer = SvgRenderer::new();
                return svg_renderer.render(shaped, font, params);
            } else {
                return Err(RenderError::FormatNotSupported(format!(
                    "Skia renderer does not support {:?}",
                    vector_format
                ))
                .into());
            }
        }

        let padding = params.padding as f32;
        let glyph_size = shaped.advance_height;

        // Build variable font location from params.variations
        let location = build_location(&font, &params.variations);

        // Phase 1: Render all glyphs first to get accurate bounds
        // This ensures we don't clip tall glyphs (emoji, Thai marks, Arabic diacritics)
        let mut rendered_glyphs: Vec<(RenderedGlyph, f32, f32)> = Vec::new();
        let mut min_y: f32 = 0.0; // Relative to baseline
        let mut max_y: f32 = 0.0; // Relative to baseline
        let mut last_error: Option<String> = None;

        for glyph in shaped.glyphs.iter() {
            match self.render_glyph(&font, glyph.id, glyph_size, &location, params) {
                Ok(bitmap) => {
                    // bearing_y is distance from baseline to top of glyph (positive = above baseline)
                    // glyph top relative to baseline = glyph.y + bearing_y
                    // glyph bottom relative to baseline = glyph.y + bearing_y - height
                    let glyph_top = glyph.y + bitmap.bearing_y as f32;
                    let glyph_bottom = glyph.y + bitmap.bearing_y as f32 - bitmap.height as f32;

                    max_y = max_y.max(glyph_top);
                    min_y = min_y.min(glyph_bottom);

                    rendered_glyphs.push((
                        RenderedGlyph {
                            bitmap,
                            glyph_x: glyph.x,
                            glyph_y: glyph.y,
                        },
                        glyph_top,
                        glyph_bottom,
                    ));
                },
                Err(e) => {
                    log::warn!("Skia: Failed to render glyph {}: {:?}", glyph.id, e);
                    last_error = Some(e.to_string());
                },
            }
        }

        if rendered_glyphs.is_empty() && !shaped.glyphs.is_empty() {
            if let Some(err) = last_error {
                return Err(RenderError::BackendError(err).into());
            }
            return Err(RenderError::BackendError("no glyphs rendered".into()).into());
        }

        // Phase 2: Calculate canvas dimensions from actual glyph bounds
        let width = (shaped.advance_width + padding * 2.0).ceil() as u32;

        // Height is from highest point above baseline to lowest point below
        // min_y is negative (below baseline), max_y is positive (above baseline)
        let content_height = if rendered_glyphs.is_empty() {
            16.0 // Default minimum for empty text
        } else {
            max_y - min_y // Total height = ascent + descent
        };
        let height = (content_height + padding * 2.0).ceil() as u32;

        // Validate dimensions
        if width == 0 || height == 0 {
            return Err(RenderError::ZeroDimensions { width, height }.into());
        }

        if width > self.max_size || height > self.max_size {
            return Err(RenderError::DimensionsTooLarge {
                width,
                height,
                max: self.max_size,
            }
            .into());
        }

        // Create premultiplied RGBA canvas
        let mut canvas = vec![0u8; (width * height * 4) as usize];

        // Fill background if specified (premultiplied)
        if let Some(bg) = params.background {
            let a = bg.a as u32;
            let r = bg.r as u32 * a / 255;
            let g = bg.g as u32 * a / 255;
            let b = bg.b as u32 * a / 255;
            for pixel in canvas.chunks_exact_mut(4) {
                pixel[0] = r as u8;
                pixel[1] = g as u8;
                pixel[2] = b as u8;
                pixel[3] = a as u8;
            }
        }

        // Baseline position: padding + distance from top to baseline
        // max_y is the highest point above baseline, so baseline is at padding + max_y
        let baseline_y = padding + max_y;

        // Phase 3: Composite pre-rendered glyphs onto canvas
        for (rg, _top, _bottom) in rendered_glyphs {
            let bitmap = &rg.bitmap;

            // Position glyph on canvas
            // X: glyph.x + padding + bearing_x
            // Y: baseline_y + glyph.y - bearing_y (convert from baseline-relative to top-origin)
            let x = (rg.glyph_x + padding) as i32 + bitmap.bearing_x;
            let y = (baseline_y + rg.glyph_y) as i32 - bitmap.bearing_y;

            match &bitmap.data {
                GlyphBitmapData::Mask(mask) => {
                    for gy in 0..bitmap.height {
                        for gx in 0..bitmap.width {
                            let canvas_x = x + gx as i32;
                            let canvas_y = y + gy as i32;

                            if canvas_x < 0
                                || canvas_x >= width as i32
                                || canvas_y < 0
                                || canvas_y >= height as i32
                            {
                                continue;
                            }

                            let canvas_idx =
                                ((canvas_y as u32 * width + canvas_x as u32) * 4) as usize;
                            let glyph_idx = (gy * bitmap.width + gx) as usize;
                            let coverage = mask[glyph_idx] as u32;
                            if coverage == 0 {
                                continue;
                            }

                            // Apply coverage to foreground color, creating premultiplied values
                            let fg = &params.foreground;
                            let src_a = coverage * fg.a as u32 / 255;
                            let src_r = fg.r as u32 * src_a / 255;
                            let src_g = fg.g as u32 * src_a / 255;
                            let src_b = fg.b as u32 * src_a / 255;

                            let dst_a = canvas[canvas_idx + 3] as u32;
                            let inv_a = 255 - src_a;

                            // Premultiplied over: out = src + dst * (1 - src_a/255)
                            canvas[canvas_idx] =
                                (src_r + canvas[canvas_idx] as u32 * inv_a / 255).min(255) as u8;
                            canvas[canvas_idx + 1] =
                                (src_g + canvas[canvas_idx + 1] as u32 * inv_a / 255).min(255) as u8;
                            canvas[canvas_idx + 2] =
                                (src_b + canvas[canvas_idx + 2] as u32 * inv_a / 255).min(255) as u8;
                            canvas[canvas_idx + 3] = (src_a + dst_a * inv_a / 255).min(255) as u8;
                        }
                    }
                },
                GlyphBitmapData::RgbaPremul(rgba) => {
                    for gy in 0..bitmap.height {
                        for gx in 0..bitmap.width {
                            let canvas_x = x + gx as i32;
                            let canvas_y = y + gy as i32;

                            if canvas_x < 0
                                || canvas_x >= width as i32
                                || canvas_y < 0
                                || canvas_y >= height as i32
                            {
                                continue;
                            }

                            let canvas_idx =
                                ((canvas_y as u32 * width + canvas_x as u32) * 4) as usize;
                            let glyph_idx = ((gy * bitmap.width + gx) * 4) as usize;

                            let src_a = rgba[glyph_idx + 3] as u32;
                            if src_a == 0 {
                                continue;
                            }

                            let src_r = rgba[glyph_idx] as u32;
                            let src_g = rgba[glyph_idx + 1] as u32;
                            let src_b = rgba[glyph_idx + 2] as u32;

                            let dst_a = canvas[canvas_idx + 3] as u32;
                            let inv_a = 255 - src_a;

                            // Premultiplied over: out = src + dst * (1 - src_a/255)
                            canvas[canvas_idx] =
                                (src_r + canvas[canvas_idx] as u32 * inv_a / 255).min(255) as u8;
                            canvas[canvas_idx + 1] =
                                (src_g + canvas[canvas_idx + 1] as u32 * inv_a / 255).min(255) as u8;
                            canvas[canvas_idx + 2] =
                                (src_b + canvas[canvas_idx + 2] as u32 * inv_a / 255).min(255) as u8;
                            canvas[canvas_idx + 3] = (src_a + dst_a * inv_a / 255).min(255) as u8;
                        }
                    }
                },
            }
        }

        // Convert premultiplied canvas back to straight RGBA for output
        let mut output = canvas;
        for px in output.chunks_exact_mut(4) {
            let a = px[3];
            if a == 0 {
                px[0] = 0;
                px[1] = 0;
                px[2] = 0;
                continue;
            }
            let a_u = a as u32;
            px[0] = ((px[0] as u32 * 255 + a_u / 2) / a_u).min(255) as u8;
            px[1] = ((px[1] as u32 * 255 + a_u / 2) / a_u).min(255) as u8;
            px[2] = ((px[2] as u32 * 255 + a_u / 2) / a_u).min(255) as u8;
        }

        Ok(RenderOutput::Bitmap(BitmapData {
            width,
            height,
            format: BitmapFormat::Rgba8,
            data: output,
        }))
    }

    fn supports_format(&self, format: &str) -> bool {
        let f = format.to_ascii_lowercase();
        matches!(f.as_str(), "bitmap" | "rgba" | "svg" | "vector")
    }
}

/// A rendered glyph ready for compositing
struct RenderedGlyph {
    bitmap: GlyphBitmap,
    glyph_x: f32,
    glyph_y: f32,
}

/// Stored glyph data for compositing
enum GlyphBitmapData {
    /// Single-channel coverage mask (monochrome outlines)
    Mask(Vec<u8>),
    /// Premultiplied RGBA pixels (color glyphs from COLR/SVG/bitmap)
    RgbaPremul(Vec<u8>),
}

/// A rendered glyph with everything needed for proper positioning
struct GlyphBitmap {
    width: u32,            // Pixel width of the glyph bitmap
    height: u32,           // Pixel height of the glyph bitmap
    data: GlyphBitmapData, // Coverage or color data
    bearing_x: i32,        // Horizontal offset from origin to left edge
    bearing_y: i32,        // Vertical offset from baseline to top edge
}

/// Bridge between skrifa's outline commands and kurbo's path format
///
/// This pen receives drawing commands from skrifa and translates them
/// into kurbo's path representation, handling scaling along the way.
struct PathPen<'a> {
    path: &'a mut kurbo::BezPath,
    scale: f32,
}

impl skrifa::outline::OutlinePen for PathPen<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        // Start a new subpath at this position
        self.path
            .move_to((x as f64 * self.scale as f64, y as f64 * self.scale as f64));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        // Draw a straight line to this point
        self.path
            .line_to((x as f64 * self.scale as f64, y as f64 * self.scale as f64));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        // Draw a quadratic Bézier curve with one control point
        self.path.quad_to(
            (
                cx0 as f64 * self.scale as f64,
                cy0 as f64 * self.scale as f64,
            ),
            (x as f64 * self.scale as f64, y as f64 * self.scale as f64),
        );
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        // Draw a cubic Bézier curve with two control points
        self.path.curve_to(
            (
                cx0 as f64 * self.scale as f64,
                cy0 as f64 * self.scale as f64,
            ),
            (
                cx1 as f64 * self.scale as f64,
                cy1 as f64 * self.scale as f64,
            ),
            (x as f64 * self.scale as f64, y as f64 * self.scale as f64),
        );
    }

    fn close(&mut self) {
        // Close the current subpath, connecting back to the start
        self.path.close_path();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use read_fonts::TableProvider;
    use std::fs;
    use std::path::PathBuf;
    use typf_core::{
        types::{BitmapFormat, Direction},
        Color, GlyphSource, GlyphSourcePreference,
    };

    #[test]
    fn test_renderer_creation() {
        let renderer = SkiaRenderer::new();
        assert_eq!(renderer.name(), "skia");
    }

    #[test]
    fn test_renderer_default() {
        let renderer = SkiaRenderer::default();
        assert_eq!(renderer.name(), "skia");
        assert_eq!(renderer.max_size, 65535);
    }

    #[test]
    fn test_supports_format() {
        let renderer = SkiaRenderer::new();
        assert!(renderer.supports_format("bitmap"));
        assert!(renderer.supports_format("rgba"));
        assert!(renderer.supports_format("svg"));
        assert!(renderer.supports_format("vector"));
        assert!(!renderer.supports_format("pdf"));
        assert!(!renderer.supports_format("unknown"));
    }

    #[test]
    fn fails_when_outlines_denied() {
        let renderer = SkiaRenderer::new();
        let font = load_test_font();

        let glyph_id = font.glyph_id('A').unwrap_or(0);
        let shaped = ShapingResult {
            glyphs: vec![typf_core::types::PositionedGlyph {
                id: glyph_id,
                x: 0.0,
                y: 0.0,
                advance: 64.0,
                cluster: 0,
            }],
            advance_width: 64.0,
            advance_height: 64.0,
            direction: Direction::LeftToRight,
        };

        let params = RenderParams {
            glyph_sources: GlyphSourcePreference::from_parts(
                Vec::new(),
                [
                    GlyphSource::Glyf,
                    GlyphSource::Cff,
                    GlyphSource::Cff2,
                    GlyphSource::Colr0,
                    GlyphSource::Colr1,
                    GlyphSource::Svg,
                    GlyphSource::Sbix,
                    GlyphSource::Cbdt,
                    GlyphSource::Ebdt,
                ],
            ),
            ..RenderParams::default()
        };

        let result = renderer.render(&shaped, font, &params);
        assert!(result.is_err(), "denying all sources should error");
    }

    fn load_test_font() -> Arc<dyn FontRef> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop(); // typf-render-skia
        path.pop(); // backends
        path.push("test-fonts");
        path.push("NotoSans-Regular.ttf");

        let font = typf_fontdb::TypfFontFace::from_file(&path)
            .expect("test font should load for SVG mode");
        Arc::new(font)
    }

    fn color_font_path(name: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop(); // typf-render-skia
        path.pop(); // backends
        path.push("test-fonts");
        path.push(name);
        path
    }

    fn load_color_font(name: &str) -> (Arc<dyn FontRef>, Vec<u8>) {
        let path = color_font_path(name);
        let bytes = fs::read(&path).expect("color font should be present");
        let font = typf_fontdb::TypfFontFace::from_file(&path).expect("color font should load");
        (Arc::new(font), bytes)
    }

    fn first_colr_glyph(font_bytes: &[u8]) -> Option<u32> {
        let font = skrifa::FontRef::new(font_bytes).ok()?;
        let color_glyphs = font.color_glyphs();
        let num = font.maxp().ok()?.num_glyphs() as u32;
        for gid in 0..num {
            let glyph_id = skrifa::GlyphId::new(gid);
            if color_glyphs
                .get_with_format(glyph_id, skrifa::color::ColorGlyphFormat::ColrV1)
                .is_some()
                || color_glyphs
                    .get_with_format(glyph_id, skrifa::color::ColorGlyphFormat::ColrV0)
                    .is_some()
            {
                return Some(glyph_id.to_u32());
            }
        }
        None
    }

    fn first_svg_glyph(font_bytes: &[u8]) -> Option<u32> {
        let font = skrifa::FontRef::new(font_bytes).ok()?;
        let svg_table = font.svg().ok()?;
        let doc_list = svg_table.svg_document_list().ok()?;
        for record in doc_list.document_records() {
            return Some(record.start_glyph_id().to_u32());
        }
        None
    }

    #[test]
    fn test_svg_output_mode_returns_vector() {
        let renderer = SkiaRenderer::new();
        let font = load_test_font();

        let glyph_id = font.glyph_id('A').unwrap_or(0);
        let shaped = ShapingResult {
            glyphs: vec![typf_core::types::PositionedGlyph {
                id: glyph_id,
                x: 0.0,
                y: 0.0,
                advance: 64.0,
                cluster: 0,
            }],
            advance_width: 64.0,
            advance_height: 64.0,
            direction: Direction::LeftToRight,
        };

        let params = RenderParams {
            output: RenderMode::Vector(VectorFormat::Svg),
            ..RenderParams::default()
        };

        let result = renderer.render(&shaped, font, &params).unwrap();

        match result {
            RenderOutput::Vector(vector) => {
                assert_eq!(vector.format, VectorFormat::Svg);
                assert!(vector.data.contains("<svg"));
            },
            other => panic!("expected vector output, got {:?}", other),
        }
    }

    #[test]
    fn renders_colr_glyph_when_outlines_denied() {
        let renderer = SkiaRenderer::new();
        let (font, bytes) = load_color_font("Nabla-Regular-COLR.ttf");
        let glyph_id = first_colr_glyph(&bytes).expect("color glyph should exist");

        let color_probe = render_glyph_with_preference(
            font.data(),
            glyph_id,
            128,
            128,
            48.0,
            0,
            &[],
            &GlyphSourcePreference::from_parts(vec![GlyphSource::Colr1, GlyphSource::Colr0], []),
        )
        .expect("color renderer should succeed directly");
        assert!(
            color_probe
                .0
                .pixmap
                .data()
                .chunks_exact(4)
                .any(|px| px[3] > 0),
            "direct color render produced empty pixmap"
        );

        let shaped = ShapingResult {
            glyphs: vec![typf_core::types::PositionedGlyph {
                id: glyph_id,
                x: 0.0,
                y: 0.0,
                advance: 32.0,
                cluster: 0,
            }],
            advance_width: 32.0,
            advance_height: 32.0,
            direction: Direction::LeftToRight,
        };

        let params = RenderParams {
            foreground: Color::rgba(9, 18, 27, 255),
            glyph_sources: GlyphSourcePreference::from_parts(
                vec![GlyphSource::Colr1, GlyphSource::Colr0],
                [GlyphSource::Glyf, GlyphSource::Cff, GlyphSource::Cff2],
            ),
            padding: 1,
            ..RenderParams::default()
        };

        let result = renderer
            .render(&shaped, font, &params)
            .expect("render should succeed");
        match result {
            RenderOutput::Bitmap(bitmap) => {
                assert_eq!(bitmap.format, BitmapFormat::Rgba8);
                let max_alpha = bitmap
                    .data
                    .chunks_exact(4)
                    .map(|px| px[3])
                    .max()
                    .unwrap_or(0);
                assert!(
                    max_alpha > 0,
                    "color glyph should render opaque pixels (alpha={}, {}x{})",
                    max_alpha,
                    bitmap.width,
                    bitmap.height
                );
            },
            other => panic!("expected bitmap output, got {:?}", other),
        }
    }

    #[test]
    fn renders_svg_glyph_when_outlines_denied() {
        let renderer = SkiaRenderer::new();
        let (font, bytes) = load_color_font("Nabla-Regular-SVG.ttf");
        let glyph_id = first_svg_glyph(&bytes).expect("svg glyph should exist");

        let svg_probe = render_glyph_with_preference(
            font.data(),
            glyph_id,
            128,
            128,
            48.0,
            0,
            &[],
            &GlyphSourcePreference::from_parts(vec![GlyphSource::Svg], []),
        )
        .expect("svg renderer should succeed directly");
        assert!(
            svg_probe
                .0
                .pixmap
                .data()
                .chunks_exact(4)
                .any(|px| px[3] > 0),
            "direct svg render produced empty pixmap"
        );

        let shaped = ShapingResult {
            glyphs: vec![typf_core::types::PositionedGlyph {
                id: glyph_id,
                x: 0.0,
                y: 0.0,
                advance: 48.0,
                cluster: 0,
            }],
            advance_width: 48.0,
            advance_height: 48.0,
            direction: Direction::LeftToRight,
        };

        let params = RenderParams {
            foreground: Color::rgba(200, 50, 10, 255),
            glyph_sources: GlyphSourcePreference::from_parts(
                vec![GlyphSource::Svg],
                [GlyphSource::Glyf, GlyphSource::Cff, GlyphSource::Cff2],
            ),
            padding: 2,
            ..RenderParams::default()
        };

        let result = renderer
            .render(&shaped, font, &params)
            .expect("render should succeed");
        match result {
            RenderOutput::Bitmap(bitmap) => {
                assert_eq!(bitmap.format, BitmapFormat::Rgba8);
                let max_alpha = bitmap
                    .data
                    .chunks_exact(4)
                    .map(|px| px[3])
                    .max()
                    .unwrap_or(0);
                assert!(
                    max_alpha > 0,
                    "svg glyph should render opaque pixels (alpha={}, {}x{})",
                    max_alpha,
                    bitmap.width,
                    bitmap.height
                );
            },
            other => panic!("expected bitmap output, got {:?}", other),
        }
    }
}
