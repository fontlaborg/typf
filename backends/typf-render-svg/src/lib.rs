//! SVG Renderer: where glyphs become scalable vector paths
//!
//! Unlike raster renderers that produce pixels, the SVG renderer extracts
//! glyph outlines directly from the font and emits perfect vector paths.
//! The result scales infinitely without quality loss.
//!
//! ## How it works
//!
//! 1. Takes shaped glyph positions from any shaper
//! 2. Extracts outline curves from the font using skrifa
//! 3. Converts curves to SVG path commands
//! 4. Returns complete SVG document as RenderOutput::Vector
//!
//! ## Canvas Sizing
//!
//! Uses two-phase rendering to ensure proper viewBox dimensions:
//! - Phase 1: Extract all glyph paths, track actual bounds
//! - Phase 2: Generate SVG with accurate viewBox from bounds

use skrifa::MetadataProvider;
use std::fmt::Write as FmtWrite;
use std::sync::Arc;
use typf_core::{
    error::{RenderError, Result},
    traits::{FontRef, Renderer},
    types::{BitmapData, BitmapFormat, RenderOutput, ShapingResult, VectorData, VectorFormat},
    GlyphSource, GlyphSourcePreference, RenderParams,
};
use typf_export::png::encode_bitmap_to_png;
use typf_render_color::render_glyph_with_preference;

/// SVG vector renderer
///
/// Produces scalable vector graphics from shaped text by extracting
/// glyph outlines directly from the font file.
#[derive(Debug, Default)]
pub struct SvgRenderer {
    /// SVG canvas padding
    padding: f32,
}

impl SvgRenderer {
    /// Create a new SVG renderer with default padding
    pub fn new() -> Self {
        Self { padding: 10.0 }
    }

    /// Set the padding around the SVG canvas
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Extract glyph outline as SVG path string with bounds
    ///
    /// Returns (path_string, min_y, max_y) where min_y/max_y are in scaled
    /// SVG coordinates (y-flipped, relative to glyph origin).
    fn extract_glyph_path_with_bounds(
        &self,
        font: &Arc<dyn FontRef>,
        glyph_id: u32,
        scale: f32,
        location: &skrifa::instance::Location,
    ) -> Result<GlyphPath> {
        let font_data = font.data();
        let font_ref = skrifa::FontRef::new(font_data).map_err(|_| RenderError::InvalidFont)?;

        let outlines = font_ref.outline_glyphs();
        // Use GlyphId::new to support full u32 range (>65k glyph IDs)
        let glyph_id = skrifa::GlyphId::new(glyph_id);

        let glyph = match outlines.get(glyph_id) {
            Some(g) => g,
            None => {
                return Ok(GlyphPath {
                    path: String::new(),
                    min_y_svg: 0.0,
                    max_y_svg: 0.0,
                    bounds: None,
                })
            },
        };

        let mut path_builder = SvgPathBuilder::new(scale);

        let size = skrifa::instance::Size::new(font.units_per_em() as f32);
        // Use provided location for variable font support
        let settings = skrifa::outline::DrawSettings::unhinted(size, location.coords());

        glyph
            .draw(settings, &mut path_builder)
            .map_err(|_| RenderError::OutlineExtractionFailed)?;

        let (path, min_y_svg, max_y_svg, bounds) = path_builder.finish_with_bounds();
        Ok(GlyphPath {
            path,
            min_y_svg,
            max_y_svg,
            bounds,
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

    /// Render a color glyph to an RGBA PNG (base64 encoded) when COLR/SVG/bitmap data exists
    fn render_color_image(
        &self,
        font: &Arc<dyn FontRef>,
        glyph_id: u32,
        bounds: &GlyphBounds,
        glyph_size: f32,
        params: &RenderParams,
        source: GlyphSource,
    ) -> Option<ColorImage> {
        let width = (bounds.max_x - bounds.min_x).ceil().max(1.0) as u32;
        let height = (bounds.max_y - bounds.min_y).ceil().max(1.0) as u32;

        if width == 0 || height == 0 {
            return None;
        }

        let variations: Vec<(&str, f32)> = params
            .variations
            .iter()
            .map(|(tag, value)| (tag.as_str(), *value))
            .collect();

        let preference = GlyphSourcePreference::from_parts(vec![source], []);

        let (render_result, _) = render_glyph_with_preference(
            font.data(),
            glyph_id,
            width,
            height,
            glyph_size,
            params.color_palette,
            &variations,
            &preference,
        )
        .ok()?;

        let bitmap = BitmapData {
            width,
            height,
            format: BitmapFormat::Rgba8,
            data: render_result.pixmap.data().to_vec(),
        };

        let png_bytes = encode_bitmap_to_png(&bitmap).ok()?;
        let data_base64 = base64_encode(&png_bytes);

        Some(ColorImage {
            data_base64,
            width,
            height,
        })
    }
}

/// Extracted glyph path with vertical bounds
#[derive(Clone, Copy, Debug)]
struct GlyphBounds {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

struct GlyphPath {
    path: String,
    min_y_svg: f32, // In SVG coords (y-flipped), relative to glyph origin
    max_y_svg: f32,
    bounds: Option<GlyphBounds>,
}

enum GlyphRenderKind {
    Path(String),
    ColorImage {
        data_base64: String,
        width: u32,
        height: u32,
    },
}

struct ColorImage {
    data_base64: String,
    width: u32,
    height: u32,
}

struct PreparedGlyph {
    x: f32,
    y: f32,
    bounds: GlyphBounds,
    kind: GlyphRenderKind,
}

impl Renderer for SvgRenderer {
    fn name(&self) -> &'static str {
        "svg"
    }

    fn render(
        &self,
        shaped: &ShapingResult,
        font: Arc<dyn FontRef>,
        params: &RenderParams,
    ) -> Result<RenderOutput> {
        log::debug!(
            "SvgRenderer: Rendering {} glyphs as vector paths",
            shaped.glyphs.len()
        );

        let padding = params.padding as f32;
        let foreground = params.foreground;
        let scale = shaped.advance_height / font.units_per_em() as f32;
        let glyph_size = shaped.advance_height;

        // Build variable font location from params.variations
        let location = Self::build_location(&font, &params.variations);

        // Phase 1: Extract all glyph paths and compute actual bounds
        // min_y/max_y are in SVG coordinates relative to baseline (y=0)
        let mut prepared_glyphs: Vec<PreparedGlyph> = Vec::new();
        let mut min_y: f32 = 0.0; // Below baseline (positive in SVG coords)
        let mut max_y: f32 = 0.0; // Above baseline (negative in SVG coords, but we track magnitude)
        let source_order = params.glyph_sources.effective_order();

        for glyph in &shaped.glyphs {
            let glyph_path =
                self.extract_glyph_path_with_bounds(&font, glyph.id, scale, &location)?;

            let bounds = match glyph_path.bounds {
                Some(b) => b,
                None => continue,
            };

            // Glyph bounds relative to baseline at this position
            // glyph.y is the vertical offset from baseline (usually 0 for base glyphs)
            let glyph_min_y = glyph_path.min_y_svg + glyph.y;
            let glyph_max_y = glyph_path.max_y_svg + glyph.y;

            min_y = min_y.min(glyph_min_y);
            max_y = max_y.max(glyph_max_y);

            let mut chosen_kind: Option<GlyphRenderKind> = None;
            for source in &source_order {
                match source {
                    GlyphSource::Glyf | GlyphSource::Cff | GlyphSource::Cff2 => {
                        if !glyph_path.path.is_empty() {
                            chosen_kind = Some(GlyphRenderKind::Path(glyph_path.path.clone()));
                            break;
                        }
                    },
                    GlyphSource::Colr1
                    | GlyphSource::Colr0
                    | GlyphSource::Svg
                    | GlyphSource::Sbix
                    | GlyphSource::Cbdt
                    | GlyphSource::Ebdt => {
                        if let Some(img) = self.render_color_image(
                            &font, glyph.id, &bounds, glyph_size, params, *source,
                        ) {
                            chosen_kind = Some(GlyphRenderKind::ColorImage {
                                data_base64: img.data_base64,
                                width: img.width,
                                height: img.height,
                            });
                            break;
                        }
                    },
                }
            }

            let Some(kind) = chosen_kind else {
                continue;
            };

            prepared_glyphs.push(PreparedGlyph {
                x: glyph.x,
                y: glyph.y,
                bounds,
                kind,
            });
        }

        // Phase 2: Calculate viewBox from actual content bounds
        let width = shaped.advance_width + padding * 2.0;

        // In SVG coords: min_y is topmost (most negative), max_y is bottommost (most positive)
        // Content height spans from min_y to max_y
        let content_height = if prepared_glyphs.is_empty() {
            shaped.advance_height // Fallback for empty text
        } else {
            max_y - min_y
        };
        let height = content_height + padding * 2.0;

        // Baseline position: distance from top of viewBox to baseline
        // min_y is the topmost point (most negative in SVG), so baseline is at:
        // padding + |min_y| = padding - min_y (since min_y is typically negative for ascenders)
        let baseline_y = padding - min_y;

        let mut svg = String::new();

        // SVG header
        writeln!(&mut svg, r#"<?xml version="1.0" encoding="UTF-8"?>"#)
            .map_err(|_| RenderError::PathBuildingFailed)?;

        writeln!(
            &mut svg,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {:.2} {:.2}" width="{:.0}" height="{:.0}">"#,
            width, height, width, height
        )
        .map_err(|_| RenderError::PathBuildingFailed)?;

        // Phase 3: Render each glyph with correct positioning
        for glyph in &prepared_glyphs {
            match &glyph.kind {
                GlyphRenderKind::Path(path) => {
                    let x = padding + glyph.x;
                    let y = baseline_y + glyph.y;

                    writeln!(
                        &mut svg,
                        r#"  <path d="{}" fill="rgb({},{},{})" fill-opacity="{:.2}" transform="translate({:.2},{:.2})"/>"#,
                        path,
                        foreground.r,
                        foreground.g,
                        foreground.b,
                        foreground.a as f32 / 255.0,
                        x,
                        y
                    )
                    .map_err(|_| RenderError::PathBuildingFailed)?;
                },
                GlyphRenderKind::ColorImage {
                    data_base64,
                    width,
                    height,
                } => {
                    let x = padding + glyph.x + glyph.bounds.min_x;
                    let y = baseline_y + glyph.y - glyph.bounds.max_y;

                    writeln!(
                        &mut svg,
                        r#"  <image x="{:.2}" y="{:.2}" width="{}" height="{}" href="data:image/png;base64,{}" />"#,
                        x,
                        y,
                        width,
                        height,
                        data_base64
                    )
                    .map_err(|_| RenderError::PathBuildingFailed)?;
                },
            }
        }

        // SVG footer
        writeln!(&mut svg, "</svg>").map_err(|_| RenderError::PathBuildingFailed)?;

        Ok(RenderOutput::Vector(VectorData {
            format: VectorFormat::Svg,
            data: svg,
        }))
    }

    fn supports_format(&self, format: &str) -> bool {
        matches!(format.to_lowercase().as_str(), "svg" | "vector")
    }
}

/// SVG path builder implementing skrifa's OutlinePen
///
/// Tracks vertical bounds while building the path for proper viewBox sizing.
struct SvgPathBuilder {
    commands: String,
    scale: f32,
    min_y_svg: f32,
    max_y_svg: f32,
    min_x_raw: f32,
    max_x_raw: f32,
    min_y_raw: f32,
    max_y_raw: f32,
    has_points: bool,
}

impl SvgPathBuilder {
    fn new(scale: f32) -> Self {
        Self {
            commands: String::new(),
            scale,
            min_y_svg: 0.0,
            max_y_svg: 0.0,
            min_x_raw: 0.0,
            max_x_raw: 0.0,
            min_y_raw: 0.0,
            max_y_raw: 0.0,
            has_points: false,
        }
    }

    /// Track a point for bounds calculation (raw coords are y-up)
    fn track_point(&mut self, x_raw: f32, y_raw: f32) {
        let y_svg = -y_raw;

        if !self.has_points {
            self.min_x_raw = x_raw;
            self.max_x_raw = x_raw;
            self.min_y_raw = y_raw;
            self.max_y_raw = y_raw;
            self.min_y_svg = y_svg;
            self.max_y_svg = y_svg;
            self.has_points = true;
        } else {
            self.min_x_raw = self.min_x_raw.min(x_raw);
            self.max_x_raw = self.max_x_raw.max(x_raw);
            self.min_y_raw = self.min_y_raw.min(y_raw);
            self.max_y_raw = self.max_y_raw.max(y_raw);
            self.min_y_svg = self.min_y_svg.min(y_svg);
            self.max_y_svg = self.max_y_svg.max(y_svg);
        }
    }

    fn finish_with_bounds(self) -> (String, f32, f32, Option<GlyphBounds>) {
        let bounds = if self.has_points {
            Some(GlyphBounds {
                min_x: self.min_x_raw,
                max_x: self.max_x_raw,
                min_y: self.min_y_raw,
                max_y: self.max_y_raw,
            })
        } else {
            None
        };

        (self.commands, self.min_y_svg, self.max_y_svg, bounds)
    }
}

impl skrifa::outline::OutlinePen for SvgPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let x = x * self.scale;
        let y_raw = y * self.scale;
        let y_svg = -y_raw; // Flip Y for SVG coordinate system
        self.track_point(x, y_raw);
        let _ = write!(&mut self.commands, "M{:.2},{:.2}", x, y_svg);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let x = x * self.scale;
        let y_raw = y * self.scale;
        let y_svg = -y_raw;
        self.track_point(x, y_raw);
        let _ = write!(&mut self.commands, "L{:.2},{:.2}", x, y_svg);
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let cx = cx * self.scale;
        let cy_raw = cy * self.scale;
        let cy_svg = -cy_raw;
        let x = x * self.scale;
        let y_raw = y * self.scale;
        let y_svg = -y_raw;
        // Track control point and endpoint
        self.track_point(cx, cy_raw);
        self.track_point(x, y_raw);
        let _ = write!(
            &mut self.commands,
            "Q{:.2},{:.2} {:.2},{:.2}",
            cx, cy_svg, x, y_svg
        );
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let cx0 = cx0 * self.scale;
        let cy0_raw = cy0 * self.scale;
        let cy0_svg = -cy0_raw;
        let cx1 = cx1 * self.scale;
        let cy1_raw = cy1 * self.scale;
        let cy1_svg = -cy1_raw;
        let x = x * self.scale;
        let y_raw = y * self.scale;
        let y_svg = -y_raw;
        // Track all control points and endpoint
        self.track_point(cx0, cy0_raw);
        self.track_point(cx1, cy1_raw);
        self.track_point(x, y_raw);
        let _ = write!(
            &mut self.commands,
            "C{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}",
            cx0, cy0_svg, cx1, cy1_svg, x, y_svg
        );
    }

    fn close(&mut self) {
        self.commands.push('Z');
    }
}

/// Simple base64 encoding (copied from typf-export to avoid extra dependencies)
fn base64_encode(data: &[u8]) -> String {
    use std::fmt::Write;

    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &byte) in chunk.iter().enumerate() {
            buf[i] = byte;
        }

        let b1 = (buf[0] >> 2) as usize;
        let b2 = (((buf[0] & 0x03) << 4) | (buf[1] >> 4)) as usize;
        let b3 = (((buf[1] & 0x0f) << 2) | (buf[2] >> 6)) as usize;
        let b4 = (buf[2] & 0x3f) as usize;

        let _ = write!(&mut result, "{}", TABLE[b1] as char);
        let _ = write!(&mut result, "{}", TABLE[b2] as char);

        if chunk.len() > 1 {
            let _ = write!(&mut result, "{}", TABLE[b3] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            let _ = write!(&mut result, "{}", TABLE[b4] as char);
        } else {
            result.push('=');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use typf_core::{
        types::{Direction, PositionedGlyph},
        GlyphSource, GlyphSourcePreference, RenderMode,
    };

    fn load_font(name: &str) -> Option<Arc<dyn FontRef>> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop(); // typf-render-svg
        path.pop(); // backends
        path.push("test-fonts");
        path.push(name);
        if !path.exists() {
            eprintln!("skipping svg renderer color tests; missing {:?}", path);
            return None;
        }

        let font = typf_fontdb::TypfFontFace::from_file(&path)
            .unwrap_or_else(|_| panic!("missing test font at {:?}", path));
        Some(Arc::new(font))
    }

    fn shaped_for_char(font: &Arc<dyn FontRef>, ch: char, size: f32) -> ShapingResult {
        let gid = font.glyph_id(ch).unwrap_or(0);
        ShapingResult {
            glyphs: vec![PositionedGlyph {
                id: gid,
                x: 0.0,
                y: 0.0,
                advance: size,
                cluster: 0,
            }],
            advance_width: size,
            advance_height: size,
            direction: Direction::LeftToRight,
        }
    }

    #[test]
    fn default_prefers_outlines_over_color() {
        let renderer = SvgRenderer::new();
        let Some(font) = load_font("Nabla-Regular-COLR.ttf") else {
            return;
        };
        let shaped = shaped_for_char(&font, 'A', 64.0);

        let params = RenderParams {
            output: RenderMode::Vector(VectorFormat::Svg),
            ..RenderParams::default()
        };

        let output = renderer.render(&shaped, font, &params).unwrap();
        let svg = match output {
            RenderOutput::Vector(v) => v.data,
            other => panic!("expected vector output, got {:?}", other),
        };

        assert!(
            svg.contains("<path"),
            "outline path should be used when outlines are preferred"
        );
        assert!(
            !svg.contains("<image"),
            "color glyphs should be skipped when outlines are preferred"
        );
    }

    #[test]
    fn prefers_color_when_requested() {
        let renderer = SvgRenderer::new();
        let Some(font) = load_font("Nabla-Regular-COLR.ttf") else {
            return;
        };
        let shaped = shaped_for_char(&font, 'A', 64.0);

        let params = RenderParams {
            output: RenderMode::Vector(VectorFormat::Svg),
            glyph_sources: GlyphSourcePreference::from_parts(
                vec![GlyphSource::Colr1, GlyphSource::Glyf],
                [],
            ),
            ..RenderParams::default()
        };

        let output = renderer.render(&shaped, font, &params).unwrap();
        let svg = match output {
            RenderOutput::Vector(v) => v.data,
            other => panic!("expected vector output, got {:?}", other),
        };

        assert!(
            svg.contains("<image"),
            "color glyph should be embedded when COLR is preferred"
        );
    }

    #[test]
    fn test_renderer_creation() {
        let renderer = SvgRenderer::new();
        assert_eq!(renderer.name(), "svg");
    }

    #[test]
    fn test_renderer_with_padding() {
        let renderer = SvgRenderer::new().with_padding(20.0);
        assert_eq!(renderer.padding, 20.0);
    }

    #[test]
    fn test_supports_format() {
        let renderer = SvgRenderer::new();
        assert!(renderer.supports_format("svg"));
        assert!(renderer.supports_format("SVG"));
        assert!(renderer.supports_format("vector"));
        assert!(!renderer.supports_format("png"));
    }
}
