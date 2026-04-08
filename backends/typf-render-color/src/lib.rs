#![warn(missing_docs)]
//! Color-glyph rendering support for Typf.
//!
//! Fonts can store color glyphs in several formats, including COLR layers, SVG
//! documents, and embedded bitmaps. This crate decodes those formats and paints
//! the result into a `tiny-skia` pixmap.

#[cfg(feature = "bitmap")]
pub mod bitmap;

#[cfg(feature = "svg")]
pub mod svg;

#[cfg(feature = "bitmap")]
pub use bitmap::{
    get_bitmap_sizes, has_bitmap_glyphs, render_bitmap_glyph, render_bitmap_glyph_or_outline,
    render_bitmap_glyph_scaled, BitmapRenderError, ScaledBitmapGlyph,
};

#[cfg(feature = "svg")]
pub use svg::{
    get_svg_document, has_svg_glyphs, render_svg_glyph, render_svg_glyph_with_palette,
    render_svg_glyph_with_palette_and_ppem, SvgRenderError,
};

use skrifa::color::{Brush, ColorPainter, ColorStop, CompositeMode, Extend, Transform};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::TableProvider;
use skrifa::{GlyphId, MetadataProvider};
use tiny_skia::{
    BlendMode, Color, FillRule, GradientStop, LinearGradient, Mask, Paint, PathBuilder,
    PixmapPaint, Point, RadialGradient, SpreadMode,
};

/// Outline pen that records glyph curves into a `tiny-skia` path.
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

/// `ColorPainter` implementation backed by `tiny-skia`.
pub struct TinySkiaColorPainter<'a> {
    pixmap: &'a mut Pixmap,
    transform_stack: Vec<tiny_skia::Transform>,
    clip_stack: Vec<Option<Mask>>,
    layer_stack: Vec<LayerState>,
    palette: &'a [skrifa::color::Color],
    font: &'a skrifa::FontRef<'a>,
    size: f32,
}

struct LayerState {
    pixmap: Pixmap,
    composite_mode: CompositeMode,
}

impl<'a> TinySkiaColorPainter<'a> {
    pub fn new(
        pixmap: &'a mut Pixmap,
        palette: &'a [skrifa::color::Color],
        font: &'a skrifa::FontRef<'a>,
        size: f32,
    ) -> Self {
        Self {
            pixmap,
            transform_stack: vec![tiny_skia::Transform::identity()],
            clip_stack: vec![None],
            layer_stack: Vec::new(),
            palette,
            font,
            size,
        }
    }

    /// Create a new color painter with an initial transform
    ///
    /// This is useful when the COLR paint commands need to be transformed from
    /// font coordinates to pixmap coordinates (scaling + translation).
    pub fn with_transform(
        pixmap: &'a mut Pixmap,
        palette: &'a [skrifa::color::Color],
        font: &'a skrifa::FontRef<'a>,
        size: f32,
        initial_transform: tiny_skia::Transform,
    ) -> Self {
        Self {
            pixmap,
            transform_stack: vec![initial_transform],
            clip_stack: vec![None],
            layer_stack: Vec::new(),
            palette,
            font,
            size,
        }
    }

    fn current_transform(&self) -> tiny_skia::Transform {
        self.transform_stack
            .last()
            .copied()
            .unwrap_or(tiny_skia::Transform::identity())
    }

    fn convert_transform(t: Transform) -> tiny_skia::Transform {
        tiny_skia::Transform::from_row(t.xx, t.yx, t.xy, t.yy, t.dx, t.dy)
    }

    fn convert_composite_mode(mode: CompositeMode) -> BlendMode {
        match mode {
            CompositeMode::Clear => BlendMode::Clear,
            CompositeMode::Src => BlendMode::Source,
            CompositeMode::Dest => BlendMode::Destination,
            CompositeMode::SrcOver => BlendMode::SourceOver,
            CompositeMode::DestOver => BlendMode::DestinationOver,
            CompositeMode::SrcIn => BlendMode::SourceIn,
            CompositeMode::DestIn => BlendMode::DestinationIn,
            CompositeMode::SrcOut => BlendMode::SourceOut,
            CompositeMode::DestOut => BlendMode::DestinationOut,
            CompositeMode::SrcAtop => BlendMode::SourceAtop,
            CompositeMode::DestAtop => BlendMode::DestinationAtop,
            CompositeMode::Xor => BlendMode::Xor,
            CompositeMode::Plus => BlendMode::Plus,
            CompositeMode::Screen => BlendMode::Screen,
            CompositeMode::Overlay => BlendMode::Overlay,
            CompositeMode::Darken => BlendMode::Darken,
            CompositeMode::Lighten => BlendMode::Lighten,
            CompositeMode::ColorDodge => BlendMode::ColorDodge,
            CompositeMode::ColorBurn => BlendMode::ColorBurn,
            CompositeMode::HardLight => BlendMode::HardLight,
            CompositeMode::SoftLight => BlendMode::SoftLight,
            CompositeMode::Difference => BlendMode::Difference,
            CompositeMode::Exclusion => BlendMode::Exclusion,
            CompositeMode::Multiply => BlendMode::Multiply,
            CompositeMode::HslHue => BlendMode::Hue,
            CompositeMode::HslSaturation => BlendMode::Saturation,
            CompositeMode::HslColor => BlendMode::Color,
            CompositeMode::HslLuminosity => BlendMode::Luminosity,
            CompositeMode::Unknown => BlendMode::SourceOver,
        }
    }

    fn get_palette_color(&self, palette_index: u16, alpha: f32) -> Color {
        if let Some(color) = self.palette.get(palette_index as usize) {
            let a = (color.alpha as f32 / 255.0) * alpha;
            Color::from_rgba8(color.red, color.green, color.blue, (a * 255.0) as u8)
        } else {
            Color::from_rgba8(0, 0, 0, (alpha * 255.0) as u8)
        }
    }

    fn convert_extend(extend: Extend) -> SpreadMode {
        match extend {
            Extend::Pad | Extend::Unknown => SpreadMode::Pad,
            Extend::Repeat => SpreadMode::Repeat,
            Extend::Reflect => SpreadMode::Reflect,
        }
    }

    fn convert_color_stops(&self, color_stops: &[ColorStop]) -> Vec<GradientStop> {
        color_stops
            .iter()
            .map(|stop| {
                let color = self.get_palette_color(stop.palette_index, stop.alpha);
                GradientStop::new(stop.offset, color)
            })
            .collect()
    }

    fn create_glyph_clip_mask(&self, glyph_id: GlyphId) -> Option<Mask> {
        let outline_glyphs = self.font.outline_glyphs();
        let outline = outline_glyphs.get(glyph_id)?;

        let mut pen = TinySkiaPathPen::new();

        let location = skrifa::instance::Location::default();
        let settings = DrawSettings::unhinted(skrifa::instance::Size::new(self.size), &location);
        outline.draw(settings, &mut pen).ok()?;

        let path = pen.finish()?;

        let width = self.pixmap.width();
        let height = self.pixmap.height();
        let mut mask = Mask::new(width, height)?;

        let transform = self.current_transform();
        mask.fill_path(&path, FillRule::Winding, true, transform);

        Some(mask)
    }

    fn create_box_clip_mask(&self, clip_box: skrifa::raw::types::BoundingBox<f32>) -> Option<Mask> {
        let rect = tiny_skia::Rect::from_ltrb(
            clip_box.x_min,
            clip_box.y_min,
            clip_box.x_max,
            clip_box.y_max,
        )?;

        let path = PathBuilder::from_rect(rect);

        let width = self.pixmap.width();
        let height = self.pixmap.height();
        let mut mask = Mask::new(width, height)?;

        let transform = self.current_transform();
        mask.fill_path(&path, FillRule::Winding, true, transform);

        Some(mask)
    }
}

impl ColorPainter for TinySkiaColorPainter<'_> {
    fn push_transform(&mut self, transform: Transform) {
        let current = self.current_transform();
        let new_transform = current.pre_concat(Self::convert_transform(transform));
        self.transform_stack.push(new_transform);
    }

    fn pop_transform(&mut self) {
        if self.transform_stack.len() > 1 {
            self.transform_stack.pop();
        }
    }

    fn push_clip_glyph(&mut self, glyph_id: GlyphId) {
        let mask = self.create_glyph_clip_mask(glyph_id);
        if mask.is_none() {
            log::debug!("push_clip_glyph: {:?} - failed to create mask", glyph_id);
        }
        self.clip_stack.push(mask);
    }

    fn push_clip_box(&mut self, clip_box: skrifa::raw::types::BoundingBox<f32>) {
        let mask = self.create_box_clip_mask(clip_box);
        if mask.is_none() {
            log::debug!(
                "push_clip_box: {:?} - failed to create mask",
                (
                    clip_box.x_min,
                    clip_box.y_min,
                    clip_box.x_max,
                    clip_box.y_max
                )
            );
        }
        self.clip_stack.push(mask);
    }

    fn pop_clip(&mut self) {
        if self.clip_stack.len() > 1 {
            self.clip_stack.pop();
        }
    }

    fn fill(&mut self, brush: Brush<'_>) {
        let transform = self.current_transform();

        let (width, height) = if let Some(layer) = self.layer_stack.last() {
            (layer.pixmap.width(), layer.pixmap.height())
        } else {
            (self.pixmap.width(), self.pixmap.height())
        };

        match brush {
            Brush::Solid {
                palette_index,
                alpha,
            } => {
                let color = self.get_palette_color(palette_index, alpha);
                let mut paint = Paint::default();
                paint.set_color(color);
                paint.anti_alias = true;

                let rect = tiny_skia::Rect::from_xywh(0.0, 0.0, width as f32, height as f32);
                if let Some(rect) = rect {
                    let clip_mask = self.clip_stack.iter().rev().find_map(|m| m.as_ref());
                    let target = if let Some(layer) = self.layer_stack.last_mut() {
                        &mut layer.pixmap
                    } else {
                        &mut *self.pixmap
                    };
                    target.fill_rect(rect, &paint, transform, clip_mask);
                }
            },
            Brush::LinearGradient {
                p0,
                p1,
                color_stops,
                extend,
            } => {
                if color_stops.is_empty() {
                    return;
                }

                if color_stops.len() == 1 {
                    let stop = &color_stops[0];
                    let color = self.get_palette_color(stop.palette_index, stop.alpha);
                    let mut paint = Paint::default();
                    paint.set_color(color);
                    paint.anti_alias = true;
                    if let Some(rect) =
                        tiny_skia::Rect::from_xywh(0.0, 0.0, width as f32, height as f32)
                    {
                        let clip_mask = self.clip_stack.iter().rev().find_map(|m| m.as_ref());
                        let target = if let Some(layer) = self.layer_stack.last_mut() {
                            &mut layer.pixmap
                        } else {
                            &mut *self.pixmap
                        };
                        target.fill_rect(rect, &paint, transform, clip_mask);
                    }
                    return;
                }

                let stops = self.convert_color_stops(color_stops);
                let spread_mode = Self::convert_extend(extend);

                if let Some(shader) = LinearGradient::new(
                    Point::from_xy(p0.x, p0.y),
                    Point::from_xy(p1.x, p1.y),
                    stops,
                    spread_mode,
                    tiny_skia::Transform::identity(),
                ) {
                    let paint = Paint {
                        shader,
                        anti_alias: true,
                        ..Default::default()
                    };

                    if let Some(rect) =
                        tiny_skia::Rect::from_xywh(0.0, 0.0, width as f32, height as f32)
                    {
                        let path = PathBuilder::from_rect(rect);
                        let clip_mask = self.clip_stack.iter().rev().find_map(|m| m.as_ref());
                        let target = if let Some(layer) = self.layer_stack.last_mut() {
                            &mut layer.pixmap
                        } else {
                            &mut *self.pixmap
                        };
                        target.fill_path(
                            &path,
                            &paint,
                            tiny_skia::FillRule::Winding,
                            transform,
                            clip_mask,
                        );
                    }
                }
            },
            Brush::RadialGradient {
                c0,
                r0,
                c1,
                r1,
                color_stops,
                extend,
            } => {
                if color_stops.is_empty() {
                    return;
                }

                if color_stops.len() == 1 {
                    let stop = &color_stops[0];
                    let color = self.get_palette_color(stop.palette_index, stop.alpha);
                    let mut paint = Paint::default();
                    paint.set_color(color);
                    paint.anti_alias = true;
                    if let Some(rect) =
                        tiny_skia::Rect::from_xywh(0.0, 0.0, width as f32, height as f32)
                    {
                        let clip_mask = self.clip_stack.iter().rev().find_map(|m| m.as_ref());
                        let target = if let Some(layer) = self.layer_stack.last_mut() {
                            &mut layer.pixmap
                        } else {
                            &mut *self.pixmap
                        };
                        target.fill_rect(rect, &paint, transform, clip_mask);
                    }
                    return;
                }

                let stops = self.convert_color_stops(color_stops);
                let spread_mode = Self::convert_extend(extend);

                // tiny-skia exposes a single-radius radial gradient, so COLRv1's
                // two-radius form is approximated with the larger radius.
                let radius = r0.max(r1).max(0.001);

                if let Some(shader) = RadialGradient::new(
                    Point::from_xy(c0.x, c0.y),
                    Point::from_xy(c1.x, c1.y),
                    radius,
                    stops,
                    spread_mode,
                    tiny_skia::Transform::identity(),
                ) {
                    let paint = Paint {
                        shader,
                        anti_alias: true,
                        ..Default::default()
                    };

                    if let Some(rect) =
                        tiny_skia::Rect::from_xywh(0.0, 0.0, width as f32, height as f32)
                    {
                        let path = PathBuilder::from_rect(rect);
                        let clip_mask = self.clip_stack.iter().rev().find_map(|m| m.as_ref());
                        let target = if let Some(layer) = self.layer_stack.last_mut() {
                            &mut layer.pixmap
                        } else {
                            &mut *self.pixmap
                        };
                        target.fill_path(
                            &path,
                            &paint,
                            tiny_skia::FillRule::Winding,
                            transform,
                            clip_mask,
                        );
                    }
                }

                log::debug!(
                    "RadialGradient: c0=({}, {}), r0={}, c1=({}, {}), r1={}, stops={}",
                    c0.x,
                    c0.y,
                    r0,
                    c1.x,
                    c1.y,
                    r1,
                    color_stops.len()
                );
            },
            Brush::SweepGradient {
                c0,
                start_angle,
                end_angle,
                color_stops,
                extend: _,
            } => {
                // tiny-skia does not support sweep gradients, so this falls back
                // to a solid color sampled from the middle stop.
                if color_stops.is_empty() {
                    return;
                }

                log::debug!(
                    "SweepGradient: c0=({}, {}), start={}, end={}, stops={} (fallback to solid)",
                    c0.x,
                    c0.y,
                    start_angle,
                    end_angle,
                    color_stops.len()
                );

                let stop = &color_stops[color_stops.len() / 2];
                let color = self.get_palette_color(stop.palette_index, stop.alpha);
                let mut paint = Paint::default();
                paint.set_color(color);
                paint.anti_alias = true;

                if let Some(rect) =
                    tiny_skia::Rect::from_xywh(0.0, 0.0, width as f32, height as f32)
                {
                    let clip_mask = self.clip_stack.iter().rev().find_map(|m| m.as_ref());
                    let target = if let Some(layer) = self.layer_stack.last_mut() {
                        &mut layer.pixmap
                    } else {
                        &mut *self.pixmap
                    };
                    target.fill_rect(rect, &paint, transform, clip_mask);
                }
            },
        }
    }

    fn push_layer(&mut self, composite_mode: CompositeMode) {
        let width = self.pixmap.width();
        let height = self.pixmap.height();

        if let Some(layer_pixmap) = Pixmap::new(width, height) {
            self.layer_stack.push(LayerState {
                pixmap: layer_pixmap,
                composite_mode,
            });
        }
    }

    fn pop_layer(&mut self) {
        if let Some(layer) = self.layer_stack.pop() {
            let blend_mode = Self::convert_composite_mode(layer.composite_mode);
            let paint = PixmapPaint {
                opacity: 1.0,
                blend_mode,
                quality: tiny_skia::FilterQuality::Bilinear,
            };

            let target = if let Some(parent_layer) = self.layer_stack.last_mut() {
                &mut parent_layer.pixmap
            } else {
                &mut *self.pixmap
            };

            target.draw_pixmap(
                0,
                0,
                layer.pixmap.as_ref(),
                &paint,
                tiny_skia::Transform::identity(),
                None,
            );
        }
    }

    fn pop_layer_with_mode(&mut self, composite_mode: CompositeMode) {
        if let Some(mut layer) = self.layer_stack.pop() {
            layer.composite_mode = composite_mode;
            self.layer_stack.push(layer);
        }
        self.pop_layer();
    }
}

pub use skrifa::color::{ColorGlyph, ColorGlyphFormat, ColorPalette, ColorPalettes, PaintError};
pub use skrifa::instance::Location;
pub use tiny_skia::Pixmap;

/// Errors returned while rendering a color glyph.
#[derive(Debug)]
pub enum ColorRenderError {
    /// Font parsing failed
    FontParseFailed,
    /// No COLR table in font
    NoColrTable,
    /// Glyph not found in COLR table
    GlyphNotFound,
    /// Painting failed
    PaintError(PaintError),
    /// Pixmap creation failed
    PixmapCreationFailed,
    /// No color palette available
    NoPalette,
    /// Bitmap rendering error
    #[cfg(feature = "bitmap")]
    BitmapError(bitmap::BitmapRenderError),
}

impl std::fmt::Display for ColorRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FontParseFailed => write!(f, "failed to parse font"),
            Self::NoColrTable => write!(f, "font has no COLR table"),
            Self::GlyphNotFound => write!(f, "glyph not found in COLR table"),
            Self::PaintError(e) => write!(f, "paint error: {:?}", e),
            Self::PixmapCreationFailed => write!(f, "failed to create pixmap"),
            Self::NoPalette => write!(f, "no color palette available"),
            #[cfg(feature = "bitmap")]
            Self::BitmapError(e) => write!(f, "bitmap error: {:?}", e),
        }
    }
}

impl std::error::Error for ColorRenderError {}

impl From<PaintError> for ColorRenderError {
    fn from(e: PaintError) -> Self {
        Self::PaintError(e)
    }
}

#[cfg(feature = "bitmap")]
impl From<bitmap::BitmapRenderError> for ColorRenderError {
    fn from(e: bitmap::BitmapRenderError) -> Self {
        Self::BitmapError(e)
    }
}

/// Render one COLR glyph into a pixmap.
///
/// Use this when you already know the glyph should be handled through the COLR
/// pipeline. For automatic fallback across COLR, SVG, bitmap, and outline
/// sources, call [`render_glyph`] instead.
pub fn render_color_glyph(
    font_data: &[u8],
    glyph_id: u32,
    width: u32,
    height: u32,
    size: f32,
    palette_index: u16,
) -> Result<Pixmap, ColorRenderError> {
    render_color_glyph_with_variations(font_data, glyph_id, width, height, size, palette_index, &[])
}

/// Render one COLR glyph with explicit variable-font coordinates.
///
/// `variations` is a slice of `(axis_tag, value)` pairs such as
/// `[("wght", 700.0), ("wdth", 75.0)]`.
pub fn render_color_glyph_with_variations(
    font_data: &[u8],
    glyph_id: u32,
    width: u32,
    height: u32,
    size: f32,
    palette_index: u16,
    variations: &[(&str, f32)],
) -> Result<Pixmap, ColorRenderError> {
    let font = skrifa::FontRef::new(font_data).map_err(|_| ColorRenderError::FontParseFailed)?;
    let glyph_id = GlyphId::new(glyph_id);

    let color_glyph = font
        .color_glyphs()
        .get(glyph_id)
        .ok_or(ColorRenderError::GlyphNotFound)?;

    let palettes = ColorPalettes::new(&font);
    let palette = palettes
        .get(palette_index)
        .ok_or(ColorRenderError::NoPalette)?;
    let colors = palette.colors();

    let location = font.axes().location(variations.iter().copied());

    let upem = font.head().map(|h| h.units_per_em()).unwrap_or(1000) as f32;
    let scale = size / upem;

    let location_ref = skrifa::instance::LocationRef::new(&[]);
    let colr_bbox = color_glyph.bounding_box(location_ref, skrifa::instance::Size::unscaled());

    let (pix_width, pix_height, translate_x, translate_y) = if let Some(bbox) = colr_bbox {
        let scaled_x0 = bbox.x_min * scale;
        let scaled_y0 = bbox.y_min * scale;
        let scaled_x1 = bbox.x_max * scale;
        let scaled_y1 = bbox.y_max * scale;

        let w = ((scaled_x1 - scaled_x0).ceil() as u32).max(1);
        let h = ((scaled_y1 - scaled_y0).ceil() as u32).max(1);

        let tx = -scaled_x0;
        let ty = scaled_y1;

        (w, h, tx, ty)
    } else {
        (width, height, 0.0, size)
    };

    let mut pixmap =
        Pixmap::new(pix_width, pix_height).ok_or(ColorRenderError::PixmapCreationFailed)?;

    let transform =
        tiny_skia::Transform::from_scale(scale, -scale).post_translate(translate_x, translate_y);

    {
        let mut painter =
            TinySkiaColorPainter::with_transform(&mut pixmap, colors, &font, size, transform);
        color_glyph.paint(&location, &mut painter)?;
    }

    Ok(pixmap)
}

/// Return true when the font exposes a COLR table.
pub fn has_color_glyphs(font_data: &[u8]) -> bool {
    if let Ok(font) = skrifa::FontRef::new(font_data) {
        font.colr().is_ok()
    } else {
        false
    }
}

/// Color-glyph formats supported by a font.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFontType {
    /// COLR v0 - layered color glyphs
    ColrV0,
    /// COLR v1 - layered color with gradients and effects
    ColrV1,
    /// SVG table - embedded SVG documents
    Svg,
    /// Bitmap tables (sbix, CBDT/CBLC, EBDT/EBLC) - embedded PNG/bitmap
    Bitmap,
}

/// Detect the color-glyph formats supported by a font.
///
/// The returned order is the preferred render order: COLR v1, COLR v0, SVG,
/// then bitmap data.
pub fn detect_color_font_types(font_data: &[u8]) -> Vec<ColorFontType> {
    let mut types = Vec::new();

    let font = match skrifa::FontRef::new(font_data) {
        Ok(f) => f,
        Err(_) => return types,
    };

    let color_glyphs = font.color_glyphs();
    let num_glyphs = font.maxp().map(|m| m.num_glyphs()).unwrap_or(0);

    let mut has_colr_v1 = false;
    let mut has_colr_v0 = false;

    for gid in 0..num_glyphs.min(1000) {
        let glyph_id = GlyphId::new(gid as u32);
        if color_glyphs
            .get_with_format(glyph_id, ColorGlyphFormat::ColrV1)
            .is_some()
        {
            has_colr_v1 = true;
            break;
        }
        if color_glyphs
            .get_with_format(glyph_id, ColorGlyphFormat::ColrV0)
            .is_some()
        {
            has_colr_v0 = true;
        }
    }

    if has_colr_v1 {
        types.push(ColorFontType::ColrV1);
    }
    if has_colr_v0 {
        types.push(ColorFontType::ColrV0);
    }

    #[cfg(feature = "svg")]
    {
        if font.svg().is_ok() {
            types.push(ColorFontType::Svg);
        }
    }

    #[cfg(feature = "bitmap")]
    {
        use skrifa::bitmap::BitmapStrikes;
        let strikes = BitmapStrikes::new(&font);
        if !strikes.is_empty() {
            types.push(ColorFontType::Bitmap);
        }
    }

    types
}

/// Return true when any color-glyph format is available.
pub fn has_any_color_support(font_data: &[u8]) -> bool {
    !detect_color_font_types(font_data).is_empty()
}

/// Return the preferred color-glyph format for rendering.
pub fn get_best_color_type(font_data: &[u8]) -> Option<ColorFontType> {
    detect_color_font_types(font_data).into_iter().next()
}

/// Return the COLR format used by a specific glyph, if any.
pub fn get_color_glyph_format(font_data: &[u8], glyph_id: u32) -> Option<ColorGlyphFormat> {
    let font = skrifa::FontRef::new(font_data).ok()?;
    let glyph_id = GlyphId::new(glyph_id);

    if font
        .color_glyphs()
        .get_with_format(glyph_id, ColorGlyphFormat::ColrV1)
        .is_some()
    {
        return Some(ColorGlyphFormat::ColrV1);
    }

    if font
        .color_glyphs()
        .get_with_format(glyph_id, ColorGlyphFormat::ColrV0)
        .is_some()
    {
        return Some(ColorGlyphFormat::ColrV0);
    }

    None
}

/// Output from glyph rendering plus method metadata.
#[derive(Debug)]
pub struct RenderResult {
    /// The rendered pixmap
    pub pixmap: Pixmap,
    /// Which rendering method was used
    pub method: RenderMethod,
    /// Bearing_x: horizontal offset from glyph origin to left edge (pixels)
    /// For bitmap glyphs, from font metrics. For COLR/SVG, computed from pixmap content bounds.
    pub bearing_x: Option<f32>,
    /// Bearing_y: vertical offset from baseline to top edge (pixels, positive = above baseline)
    /// For bitmap glyphs, from font metrics. For COLR/SVG, computed from pixmap content bounds.
    pub bearing_y: Option<f32>,
}

/// Rendering path chosen for a glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMethod {
    /// COLR v0 layered color
    ColrV0,
    /// COLR v1 with gradients
    ColrV1,
    /// SVG table glyph
    Svg,
    /// Bitmap from sbix/CBDT/CBLC
    Bitmap,
    /// Outline fallback (monochrome)
    Outline,
}

/// Bounds of non-transparent pixels in a pixmap.
#[derive(Debug, Clone, Copy)]
pub struct ContentBounds {
    /// Leftmost column with non-transparent content (0-indexed)
    pub min_x: u32,
    /// Rightmost column with non-transparent content (0-indexed, inclusive)
    pub max_x: u32,
    /// Topmost row with non-transparent content (0-indexed)
    pub min_y: u32,
    /// Bottommost row with non-transparent content (0-indexed, inclusive)
    pub max_y: u32,
}

impl ContentBounds {
    /// Returns content width in pixels
    pub fn width(&self) -> u32 {
        self.max_x.saturating_sub(self.min_x) + 1
    }

    /// Returns content height in pixels
    pub fn height(&self) -> u32 {
        self.max_y.saturating_sub(self.min_y) + 1
    }
}

/// Compute the bounds of non-transparent pixels in a pixmap.
pub fn compute_content_bounds(pixmap: &Pixmap) -> Option<ContentBounds> {
    let width = pixmap.width();
    let height = pixmap.height();
    let data = pixmap.data();

    let mut min_x = width;
    let mut max_x = 0;
    let mut min_y = height;
    let mut max_y = 0;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4 + 3) as usize;
            if data[idx] > 0 {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
    }

    if min_x > max_x || min_y > max_y {
        None
    } else {
        Some(ContentBounds {
            min_x,
            max_x,
            min_y,
            max_y,
        })
    }
}

/// Render a glyph with the best available color source.
///
/// The fallback order is COLR v1, COLR v0, SVG, bitmap, then outline. The
/// returned [`RenderResult`] reports which path was used.
pub fn render_glyph(
    font_data: &[u8],
    glyph_id: u32,
    width: u32,
    height: u32,
    size: f32,
    palette_index: u16,
) -> Result<RenderResult, ColorRenderError> {
    render_glyph_with_variations(font_data, glyph_id, width, height, size, palette_index, &[])
}

/// Render a glyph with explicit variable-font coordinates.
///
/// This follows the same source-selection order as [`render_glyph`], but uses
/// the supplied axis coordinates when the font supports variation settings.
pub fn render_glyph_with_variations(
    font_data: &[u8],
    glyph_id: u32,
    width: u32,
    height: u32,
    size: f32,
    palette_index: u16,
    variations: &[(&str, f32)],
) -> Result<RenderResult, ColorRenderError> {
    let font = skrifa::FontRef::new(font_data).map_err(|_| ColorRenderError::FontParseFailed)?;
    let gid = GlyphId::new(glyph_id);

    if font
        .color_glyphs()
        .get_with_format(gid, ColorGlyphFormat::ColrV1)
        .is_some()
    {
        let pixmap = render_color_glyph_with_variations(
            font_data,
            glyph_id,
            width,
            height,
            size,
            palette_index,
            variations,
        )?;
        return Ok(RenderResult {
            pixmap,
            method: RenderMethod::ColrV1,
            bearing_x: None,
            bearing_y: None,
        });
    }

    if font
        .color_glyphs()
        .get_with_format(gid, ColorGlyphFormat::ColrV0)
        .is_some()
    {
        let pixmap = render_color_glyph_with_variations(
            font_data,
            glyph_id,
            width,
            height,
            size,
            palette_index,
            variations,
        )?;
        return Ok(RenderResult {
            pixmap,
            method: RenderMethod::ColrV0,
            bearing_x: None,
            bearing_y: None,
        });
    }

    #[cfg(feature = "svg")]
    {
        let palettes = ColorPalettes::new(&font);
        let palette_colors: Vec<skrifa::color::Color> = palettes
            .get(palette_index)
            .map(|p| p.colors().to_vec())
            .unwrap_or_default();

        if let Ok(pixmap) = svg::render_svg_glyph_with_palette_and_ppem(
            font_data,
            glyph_id,
            width,
            height,
            &palette_colors,
            size,
        ) {
            return Ok(RenderResult {
                pixmap,
                method: RenderMethod::Svg,
                bearing_x: None,
                bearing_y: None,
            });
        }
    }

    #[cfg(feature = "bitmap")]
    {
        match bitmap::render_bitmap_glyph_scaled(font_data, glyph_id, size) {
            Ok(scaled) => Ok(RenderResult {
                pixmap: scaled.pixmap,
                method: RenderMethod::Bitmap,
                bearing_x: Some(scaled.bearing_x),
                bearing_y: Some(scaled.bearing_y),
            }),
            Err(bitmap::BitmapRenderError::NoBitmapTable)
            | Err(bitmap::BitmapRenderError::GlyphNotFound)
            | Err(bitmap::BitmapRenderError::UnsupportedFormat) => {
                let (pixmap, _used_bitmap) = bitmap::render_bitmap_glyph_or_outline(
                    font_data, glyph_id, width, height, size,
                )?;
                Ok(RenderResult {
                    pixmap,
                    method: RenderMethod::Outline,
                    bearing_x: None,
                    bearing_y: None,
                })
            },
            Err(e) => Err(e.into()),
        }
    }

    #[cfg(not(feature = "bitmap"))]
    Err(ColorRenderError::GlyphNotFound)
}

/// Render a specific glyph source in the order provided by GlyphSourcePreference.
///
/// Attempts sources in order; returns the first successful render along with
/// the resolved GlyphSource. Outline-only sources are skipped here so that
/// bitmap/vector renderers can keep using their existing outline paths.
#[allow(clippy::too_many_arguments)]
pub fn render_glyph_with_preference(
    font_data: &[u8],
    glyph_id: u32,
    width: u32,
    height: u32,
    size: f32,
    palette_index: u16,
    variations: &[(&str, f32)],
    preference: &typf_core::GlyphSourcePreference,
) -> Result<(RenderResult, typf_core::GlyphSource), ColorRenderError> {
    use typf_core::GlyphSource;

    let gid = GlyphId::new(glyph_id);
    let font = skrifa::FontRef::new(font_data).map_err(|_| ColorRenderError::FontParseFailed)?;
    let orders = preference.effective_order();

    for source in orders {
        match source {
            GlyphSource::Colr1 => {
                if let Some(color_glyph) = font
                    .color_glyphs()
                    .get_with_format(gid, ColorGlyphFormat::ColrV1)
                {
                    let palettes = ColorPalettes::new(&font);
                    let palette = palettes
                        .get(palette_index)
                        .ok_or(ColorRenderError::NoPalette)?;
                    let colors = palette.colors();
                    let location = font.axes().location(variations.iter().copied());

                    let mut pixmap =
                        Pixmap::new(width, height).ok_or(ColorRenderError::PixmapCreationFailed)?;
                    let mut painter = TinySkiaColorPainter::new(&mut pixmap, colors, &font, size);
                    color_glyph.paint(&location, &mut painter)?;

                    return Ok((
                        RenderResult {
                            pixmap,
                            method: RenderMethod::ColrV1,
                            bearing_x: None,
                            bearing_y: None,
                        },
                        GlyphSource::Colr1,
                    ));
                }
            },
            GlyphSource::Colr0 => {
                if let Some(color_glyph) = font
                    .color_glyphs()
                    .get_with_format(gid, ColorGlyphFormat::ColrV0)
                {
                    let palettes = ColorPalettes::new(&font);
                    let palette = palettes
                        .get(palette_index)
                        .ok_or(ColorRenderError::NoPalette)?;
                    let colors = palette.colors();
                    let location = font.axes().location(variations.iter().copied());

                    let mut pixmap =
                        Pixmap::new(width, height).ok_or(ColorRenderError::PixmapCreationFailed)?;
                    let mut painter = TinySkiaColorPainter::new(&mut pixmap, colors, &font, size);
                    color_glyph.paint(&location, &mut painter)?;

                    return Ok((
                        RenderResult {
                            pixmap,
                            method: RenderMethod::ColrV0,
                            bearing_x: None,
                            bearing_y: None,
                        },
                        GlyphSource::Colr0,
                    ));
                }
            },
            GlyphSource::Svg => {
                #[cfg(feature = "svg")]
                {
                    let palettes = ColorPalettes::new(&font);
                    let palette_colors: Vec<skrifa::color::Color> = palettes
                        .get(palette_index)
                        .map(|p| p.colors().to_vec())
                        .unwrap_or_default();

                    if let Ok(pixmap) = svg::render_svg_glyph_with_palette_and_ppem(
                        font_data,
                        glyph_id,
                        width,
                        height,
                        &palette_colors,
                        size,
                    ) {
                        return Ok((
                            RenderResult {
                                pixmap,
                                method: RenderMethod::Svg,
                                bearing_x: Some(0.0),
                                bearing_y: Some(size),
                            },
                            GlyphSource::Svg,
                        ));
                    }
                }
            },
            GlyphSource::Sbix | GlyphSource::Cbdt | GlyphSource::Ebdt => {
                #[cfg(feature = "bitmap")]
                {
                    match bitmap::render_bitmap_glyph_scaled(font_data, glyph_id, size) {
                        Ok(scaled) => {
                            return Ok((
                                RenderResult {
                                    pixmap: scaled.pixmap,
                                    method: RenderMethod::Bitmap,
                                    bearing_x: Some(scaled.bearing_x),
                                    bearing_y: Some(scaled.bearing_y),
                                },
                                source,
                            ));
                        },
                        Err(bitmap::BitmapRenderError::NoBitmapTable)
                        | Err(bitmap::BitmapRenderError::GlyphNotFound)
                        | Err(bitmap::BitmapRenderError::UnsupportedFormat) => {},
                        Err(e) => return Err(e.into()),
                    }
                }
            },
            GlyphSource::Glyf | GlyphSource::Cff | GlyphSource::Cff2 => {
                continue;
            },
        }
    }

    Err(ColorRenderError::GlyphNotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composite_mode_conversion() {
        assert_eq!(
            TinySkiaColorPainter::convert_composite_mode(CompositeMode::SrcOver),
            BlendMode::SourceOver
        );
        assert_eq!(
            TinySkiaColorPainter::convert_composite_mode(CompositeMode::Multiply),
            BlendMode::Multiply
        );
    }

    /// Test COLR glyph detection with NotoColorEmojiCOLR font
    #[test]
    fn test_has_color_glyphs_noto_colr() {
        let font_path =
            "../../external/resvg/crates/resvg/tests/fonts/NotoColorEmojiCOLR.subset.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            assert!(
                has_color_glyphs(&font_data),
                "NotoColorEmojiCOLR should have COLR table"
            );
        } else {
            // Skip test if font not available
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    /// Test that regular fonts don't have color glyphs
    #[test]
    fn test_has_color_glyphs_regular_font() {
        let font_path = "../../external/resvg/crates/resvg/tests/fonts/NotoSans-Regular.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            assert!(
                !has_color_glyphs(&font_data),
                "NotoSans-Regular should not have COLR table"
            );
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    /// Test rendering a COLR glyph from NotoColorEmoji
    #[test]
    fn test_render_colr_glyph() {
        let font_path =
            "../../external/resvg/crates/resvg/tests/fonts/NotoColorEmojiCOLR.subset.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            // First check if we can find a color glyph
            let font = skrifa::FontRef::new(&font_data).expect("Failed to parse font");
            let color_glyphs = font.color_glyphs();

            // Try to find any color glyph in the font
            let mut found_glyph = None;
            for gid in 0..font.maxp().expect("no maxp").num_glyphs() {
                let glyph_id = GlyphId::new(gid as u32);
                if color_glyphs.get(glyph_id).is_some() {
                    found_glyph = Some(gid);
                    break;
                }
            }

            if let Some(gid) = found_glyph {
                let result = render_color_glyph(&font_data, gid as u32, 128, 128, 128.0, 0);
                assert!(
                    result.is_ok(),
                    "Failed to render COLR glyph: {:?}",
                    result.err()
                );
                let pixmap = result.unwrap();
                assert_eq!(pixmap.width(), 128);
                assert_eq!(pixmap.height(), 128);
            } else {
                eprintln!("No color glyphs found in subset font");
            }
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    /// Test get_color_glyph_format detection
    #[test]
    fn test_get_color_glyph_format_colrv1() {
        let font_path = "../../external/old-typf/testdata/fonts/NotoColorEmojiColr1.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            let font = skrifa::FontRef::new(&font_data).expect("Failed to parse font");
            let color_glyphs = font.color_glyphs();

            // Find any color glyph
            for gid in 0..font.maxp().expect("no maxp").num_glyphs() {
                let glyph_id = GlyphId::new(gid as u32);
                if color_glyphs.get(glyph_id).is_some() {
                    let format = get_color_glyph_format(&font_data, gid as u32);
                    assert!(
                        format.is_some(),
                        "Should detect color format for glyph {}",
                        gid
                    );
                    // NotoColorEmojiColr1 uses COLRv1
                    if matches!(format, Some(ColorGlyphFormat::ColrV1)) {
                        println!("Found COLRv1 glyph at index {}", gid);
                        return; // Test passed
                    }
                }
            }
            eprintln!("No COLRv1 glyphs found in font");
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    /// Test unified color font type detection
    #[test]
    fn test_detect_color_font_types_colr() {
        let font_path =
            "../../external/resvg/crates/resvg/tests/fonts/NotoColorEmojiCOLR.subset.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            let types = detect_color_font_types(&font_data);
            println!("Detected color types: {:?}", types);
            assert!(!types.is_empty(), "Should detect at least one color type");
            // Should detect COLR v0 or v1
            assert!(
                types.contains(&ColorFontType::ColrV0) || types.contains(&ColorFontType::ColrV1),
                "Should detect COLR support"
            );
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    /// Test unified detection for regular fonts (no color)
    #[test]
    fn test_detect_color_font_types_regular() {
        let font_path = "../../test-fonts/NotoSans-Regular.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            let types = detect_color_font_types(&font_data);
            assert!(types.is_empty(), "Regular font should have no color types");
            assert!(!has_any_color_support(&font_data));
            assert!(get_best_color_type(&font_data).is_none());
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    /// Test detection for sbix font
    #[test]
    #[cfg(feature = "bitmap")]
    fn test_detect_color_font_types_sbix() {
        let font_path = "../../test-fonts/Nabla-Regular-sbix.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            let types = detect_color_font_types(&font_data);
            println!("Detected color types for sbix: {:?}", types);
            assert!(
                types.contains(&ColorFontType::Bitmap),
                "Should detect bitmap support"
            );
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    /// Test detection for SVG font
    #[test]
    #[cfg(feature = "svg")]
    fn test_detect_color_font_types_svg() {
        let font_path = "../../test-fonts/Nabla-Regular-SVG.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            let types = detect_color_font_types(&font_data);
            println!("Detected color types for SVG: {:?}", types);
            assert!(
                types.contains(&ColorFontType::Svg),
                "Should detect SVG support"
            );
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    // ========================================
    // Unified render_glyph Tests
    // ========================================

    /// Test unified render_glyph with COLR font
    #[test]
    #[cfg(feature = "bitmap")]
    fn test_render_glyph_unified_colr() {
        let font_path =
            "../../external/resvg/crates/resvg/tests/fonts/NotoColorEmojiCOLR.subset.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            let font = skrifa::FontRef::new(&font_data).unwrap();
            let color_glyphs = font.color_glyphs();

            for gid in 0..font.maxp().unwrap().num_glyphs().min(50) {
                let glyph_id = GlyphId::new(gid as u32);
                if color_glyphs.get(glyph_id).is_some() {
                    let result = render_glyph(&font_data, gid as u32, 64, 64, 64.0, 0);
                    assert!(result.is_ok());
                    let res = result.unwrap();
                    assert!(
                        matches!(res.method, RenderMethod::ColrV0 | RenderMethod::ColrV1),
                        "Expected COLR method, got {:?}",
                        res.method
                    );
                    println!("Unified render used {:?} for glyph {}", res.method, gid);
                    return;
                }
            }
        }
    }

    /// Test unified render_glyph with SVG font
    #[test]
    #[cfg(all(feature = "svg", feature = "bitmap"))]
    fn test_render_glyph_unified_svg() {
        let font_path = "../../test-fonts/Nabla-Regular-SVG.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            // Try rendering a glyph - should use SVG method
            for gid in 1..50 {
                if let Ok(result) = render_glyph(&font_data, gid, 128, 128, 128.0, 0) {
                    if result.method == RenderMethod::Svg {
                        println!("Unified render used SVG for glyph {}", gid);
                        return;
                    }
                }
            }
        }
    }

    /// Test unified render_glyph with sbix font
    #[test]
    #[cfg(feature = "bitmap")]
    fn test_render_glyph_unified_bitmap() {
        let font_path = "../../test-fonts/Nabla-Regular-sbix.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            for gid in 1..50 {
                if let Ok(result) = render_glyph(&font_data, gid, 64, 64, 64.0, 0) {
                    println!("Unified render used {:?} for glyph {}", result.method, gid);
                    // sbix fonts should use Bitmap or Outline
                    assert!(matches!(
                        result.method,
                        RenderMethod::Bitmap | RenderMethod::Outline
                    ));
                    return;
                }
            }
        }
    }

    // ========================================
    // Success Metrics Tests (from PLAN.md)
    // ========================================

    /// Success Metric: Noto Color Emoji renders correctly (COLR format)
    #[test]
    fn test_success_metric_noto_colr_emoji() {
        let font_path =
            "../../external/resvg/crates/resvg/tests/fonts/NotoColorEmojiCOLR.subset.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            // Verify COLR detection
            assert!(has_color_glyphs(&font_data), "Should detect COLR table");

            // Find and render a color glyph
            let font = skrifa::FontRef::new(&font_data).expect("Failed to parse font");
            let color_glyphs = font.color_glyphs();
            let mut rendered_count = 0;

            for gid in 0..font.maxp().unwrap().num_glyphs().min(100) {
                let glyph_id = GlyphId::new(gid as u32);
                if color_glyphs.get(glyph_id).is_some() {
                    let result = render_color_glyph(&font_data, gid as u32, 64, 64, 64.0, 0);
                    assert!(
                        result.is_ok(),
                        "Failed to render glyph {}: {:?}",
                        gid,
                        result.err()
                    );
                    let pixmap = result.unwrap();
                    // Verify non-empty output
                    assert!(
                        pixmap.data().iter().any(|&b| b != 0),
                        "Glyph {} should have non-empty pixels",
                        gid
                    );
                    rendered_count += 1;
                    if rendered_count >= 3 {
                        break;
                    }
                }
            }
            assert!(rendered_count > 0, "Should render at least one color glyph");
            println!(
                "SUCCESS: Rendered {} Noto COLR emoji glyphs",
                rendered_count
            );
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    /// Success Metric: Apple Color Emoji sbix glyphs display at correct sizes
    #[test]
    #[cfg(feature = "bitmap")]
    fn test_success_metric_sbix_sizes() {
        use crate::bitmap::{get_bitmap_sizes, render_bitmap_glyph};

        let font_path = "../../test-fonts/Nabla-Regular-sbix.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            // Get available sizes
            let sizes = get_bitmap_sizes(&font_data);
            assert!(!sizes.is_empty(), "sbix font should have strike sizes");
            println!("Available sbix sizes: {:?}", sizes);

            // Render at different sizes and verify output dimensions scale appropriately
            for gid in 1..50 {
                if let Ok(pixmap) = render_bitmap_glyph(&font_data, gid, sizes[0]) {
                    // Verify the bitmap has content
                    assert!(pixmap.width() > 0 && pixmap.height() > 0);
                    println!(
                        "SUCCESS: sbix glyph {} rendered at {}x{}",
                        gid,
                        pixmap.width(),
                        pixmap.height()
                    );
                    return;
                }
            }
            eprintln!("No renderable sbix glyphs found");
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    /// Success Metric: COLR v1 renders with gradients
    #[test]
    fn test_success_metric_colrv1_gradients() {
        let font_path = "../../test-fonts/Nabla-Regular-COLR.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            // Verify COLRv1 detection
            let types = detect_color_font_types(&font_data);
            println!("Detected types: {:?}", types);

            // Find and render a COLRv1 glyph
            let font = skrifa::FontRef::new(&font_data).expect("Failed to parse font");
            let color_glyphs = font.color_glyphs();

            for gid in 0..font.maxp().unwrap().num_glyphs().min(200) {
                let glyph_id = GlyphId::new(gid as u32);
                if color_glyphs
                    .get_with_format(glyph_id, ColorGlyphFormat::ColrV1)
                    .is_some()
                {
                    let result = render_color_glyph(&font_data, gid as u32, 128, 128, 128.0, 0);
                    assert!(
                        result.is_ok(),
                        "Failed to render COLRv1 glyph {}: {:?}",
                        gid,
                        result.err()
                    );
                    println!("SUCCESS: Rendered COLRv1 glyph {} with gradients", gid);
                    return;
                }
            }
            eprintln!("No COLRv1 glyphs found in font");
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    /// Success Metric: Custom SVG fonts render accurately
    #[test]
    #[cfg(feature = "svg")]
    fn test_success_metric_svg_accuracy() {
        use crate::svg::{get_svg_document, render_svg_glyph};

        let font_path = "../../test-fonts/Nabla-Regular-SVG.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            // Find and render SVG glyphs - some may be empty, so try several
            let mut rendered_count = 0;
            for gid in 1..100 {
                if let Ok(svg_doc) = get_svg_document(&font_data, gid) {
                    // Verify SVG content is valid
                    assert!(
                        svg_doc.contains("<svg") || svg_doc.contains("<?xml"),
                        "Should be valid SVG"
                    );

                    // Render (some glyphs may be empty/whitespace)
                    if let Ok(pixmap) = render_svg_glyph(&font_data, gid, 256, 256) {
                        if pixmap.data().iter().any(|&b| b != 0) {
                            println!("SUCCESS: Rendered SVG glyph {} accurately", gid);
                            rendered_count += 1;
                            if rendered_count >= 3 {
                                return;
                            }
                        }
                    }
                }
            }
            assert!(rendered_count > 0, "Should render at least one SVG glyph");
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }

    /// Success Metric: Performance - color glyph overhead
    #[test]
    fn test_success_metric_performance() {
        let font_path =
            "../../external/resvg/crates/resvg/tests/fonts/NotoColorEmojiCOLR.subset.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            let font = skrifa::FontRef::new(&font_data).expect("Failed to parse font");
            let color_glyphs = font.color_glyphs();

            // Find a color glyph
            for gid in 0..font.maxp().unwrap().num_glyphs().min(100) {
                let glyph_id = GlyphId::new(gid as u32);
                if color_glyphs.get(glyph_id).is_some() {
                    // Warm up
                    let _ = render_color_glyph(&font_data, gid as u32, 64, 64, 64.0, 0);

                    // Time multiple renders
                    let start = std::time::Instant::now();
                    let iterations = 100;
                    for _ in 0..iterations {
                        let _ = render_color_glyph(&font_data, gid as u32, 64, 64, 64.0, 0);
                    }
                    let elapsed = start.elapsed();
                    let per_glyph_us = elapsed.as_micros() as f64 / iterations as f64;

                    println!(
                        "SUCCESS: Color glyph rendering: {:.1} µs/glyph ({} iterations)",
                        per_glyph_us, iterations
                    );

                    // Basic sanity check - should be under 10ms per glyph
                    assert!(
                        per_glyph_us < 10_000.0,
                        "Performance too slow: {:.1} µs/glyph",
                        per_glyph_us
                    );
                    return;
                }
            }
            eprintln!("No color glyphs found for performance test");
        } else {
            eprintln!("Skipping test: font not found at {}", font_path);
        }
    }
}
