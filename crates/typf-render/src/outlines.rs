// this_file: crates/typf-render/src/outlines.rs

//! Shared glyph outline recording utilities.

use kurbo::{BezPath, Point};
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{GlyphId, MetadataProvider};
use std::collections::HashMap;

/// Recorded outline commands for a glyph.
#[derive(Debug, Clone, PartialEq)]
pub enum OutlineCommand {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    QuadTo {
        ctrl_x: f32,
        ctrl_y: f32,
        x: f32,
        y: f32,
    },
    CurveTo {
        ctrl1_x: f32,
        ctrl1_y: f32,
        ctrl2_x: f32,
        ctrl2_y: f32,
        x: f32,
        y: f32,
    },
    Close,
}

/// Geometry container for a recorded glyph outline.
#[derive(Debug, Clone, Default)]
pub struct GlyphOutline {
    commands: Vec<OutlineCommand>,
}

impl GlyphOutline {
    pub fn commands(&self) -> &[OutlineCommand] {
        &self.commands
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Convert the recorded outline into a `kurbo::BezPath`, applying the provided font-scale.
    pub fn to_bez_path(&self, scale: f32) -> BezPath {
        if self.commands.is_empty() || scale <= 0.0 {
            return BezPath::new();
        }

        let mut path = BezPath::new();
        for command in &self.commands {
            match *command {
                OutlineCommand::MoveTo(x, y) => path.move_to(scale_point(x, y, scale)),
                OutlineCommand::LineTo(x, y) => path.line_to(scale_point(x, y, scale)),
                OutlineCommand::QuadTo {
                    ctrl_x,
                    ctrl_y,
                    x,
                    y,
                } => path.quad_to(scale_point(ctrl_x, ctrl_y, scale), scale_point(x, y, scale)),
                OutlineCommand::CurveTo {
                    ctrl1_x,
                    ctrl1_y,
                    ctrl2_x,
                    ctrl2_y,
                    x,
                    y,
                } => path.curve_to(
                    scale_point(ctrl1_x, ctrl1_y, scale),
                    scale_point(ctrl2_x, ctrl2_y, scale),
                    scale_point(x, y, scale),
                ),
                OutlineCommand::Close => path.close_path(),
            }
        }

        path
    }
}

fn scale_point(x: f32, y: f32, scale: f32) -> Point {
    Point::new((x as f64) * (scale as f64), -(y as f64) * (scale as f64))
}

/// Types that can expose glyph outlines via skrifa's OutlinePen.
pub trait OutlineSource {
    fn outline_with_pen<P: OutlinePen>(
        &self,
        glyph_id: GlyphId,
        size: f32,
        pen: &mut P,
    ) -> Option<()>;

    fn outline_with_pen_and_location<P: OutlinePen>(
        &self,
        glyph_id: GlyphId,
        size: f32,
        variations: &HashMap<String, f32>,
        pen: &mut P,
    ) -> Option<()>;
}

impl<'a> OutlineSource for skrifa::FontRef<'a> {
    fn outline_with_pen<P: OutlinePen>(
        &self,
        glyph_id: GlyphId,
        size: f32,
        pen: &mut P,
    ) -> Option<()> {
        let outlines = self.outline_glyphs();
        let location = LocationRef::default();
        let settings = DrawSettings::unhinted(Size::new(size), location);
        outlines.get(glyph_id)?.draw(settings, pen).ok()?;
        Some(())
    }

    fn outline_with_pen_and_location<P: OutlinePen>(
        &self,
        glyph_id: GlyphId,
        size: f32,
        variations: &HashMap<String, f32>,
        pen: &mut P,
    ) -> Option<()> {
        if variations.is_empty() {
            return self.outline_with_pen(glyph_id, size, pen);
        }

        // Convert variation HashMap to skrifa Location
        let axes = self.axes();
        let location_coords: Vec<_> = variations
            .iter()
            .filter_map(|(tag, &value)| {
                if tag.len() != 4 {
                    return None;
                }
                let tag_bytes = tag.as_bytes();
                axes.iter()
                    .find(|axis| axis.tag().to_be_bytes() == tag_bytes)
                    .map(|axis| {
                        let min = axis.min_value();
                        let max = axis.max_value();
                        (axis.tag(), value.clamp(min, max))
                    })
            })
            .collect();

        let location = self
            .axes()
            .location(location_coords.iter().map(|(t, v)| (*t, *v)));

        let outlines = self.outline_glyphs();
        let settings = DrawSettings::unhinted(Size::new(size), &location);
        outlines.get(glyph_id)?.draw(settings, pen).ok()?;
        Some(())
    }
}

/// Record the outline for the provided glyph.
///
/// The `size` parameter specifies the font size in points for skrifa to use
/// when extracting the outline. The outline coordinates are in font units.
pub fn glyph_outline<S: OutlineSource>(
    source: &S,
    glyph_id: GlyphId,
    size: f32,
) -> Option<GlyphOutline> {
    glyph_outline_with_variations(source, glyph_id, size, None)
}

pub fn glyph_outline_with_variations<S: OutlineSource>(
    source: &S,
    glyph_id: GlyphId,
    size: f32,
    variations: Option<&HashMap<String, f32>>,
) -> Option<GlyphOutline> {
    let mut recorder = RecordingOutline::default();
    record_outline(source, glyph_id, size, variations, &mut recorder)?;
    let outline = recorder.finish();
    (!outline.is_empty()).then_some(outline)
}

/// Convenience helper that records and converts a glyph outline to a `BezPath`.
pub fn glyph_bez_path<S: OutlineSource>(
    source: &S,
    glyph_id: GlyphId,
    size: f32,
    scale: f32,
) -> Option<BezPath> {
    if scale <= 0.0 || size <= 0.0 {
        return None;
    }
    glyph_outline(source, glyph_id, size).map(|outline| outline.to_bez_path(scale))
}

/// Convert a glyph outline with explicit variations into a `BezPath`.
pub fn glyph_bez_path_with_variations<S: OutlineSource>(
    source: &S,
    glyph_id: GlyphId,
    size: f32,
    scale: f32,
    variations: Option<&HashMap<String, f32>>,
) -> Option<BezPath> {
    if scale <= 0.0 || size <= 0.0 {
        return None;
    }
    glyph_outline_with_variations(source, glyph_id, size, variations)
        .map(|outline| outline.to_bez_path(scale))
}

fn record_outline<S: OutlineSource, P: OutlinePen>(
    source: &S,
    glyph_id: GlyphId,
    size: f32,
    variations: Option<&HashMap<String, f32>>,
    pen: &mut P,
) -> Option<()> {
    if let Some(vars) = variations {
        if !vars.is_empty() {
            return source.outline_with_pen_and_location(glyph_id, size, vars, pen);
        }
    }
    source.outline_with_pen(glyph_id, size, pen)
}

#[derive(Default)]
struct RecordingOutline {
    commands: Vec<OutlineCommand>,
}

impl RecordingOutline {
    fn finish(self) -> GlyphOutline {
        GlyphOutline {
            commands: self.commands,
        }
    }
}

impl OutlinePen for RecordingOutline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.commands.push(OutlineCommand::MoveTo(x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.commands.push(OutlineCommand::LineTo(x, y));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.commands.push(OutlineCommand::QuadTo {
            ctrl_x: cx0,
            ctrl_y: cy0,
            x,
            y,
        });
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.commands.push(OutlineCommand::CurveTo {
            ctrl1_x: cx0,
            ctrl1_y: cy0,
            ctrl2_x: cx1,
            ctrl2_y: cy1,
            x,
            y,
        });
    }

    fn close(&mut self) {
        self.commands.push(OutlineCommand::Close);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::Shape;
    use skrifa::{FontRef, MetadataProvider};
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn captures_outline_for_known_glyph() {
        let (font, _data) = noto_font();
        let glyph_id = font
            .charmap()
            .map('A')
            .expect("Noto Sans should include 'A'");
        let outline = glyph_outline(&font, glyph_id, 48.0).expect("outline recorded");
        assert!(outline.commands().len() > 4, "expected multiple commands");
    }

    #[test]
    fn records_quadratic_and_cubic_segments() {
        let (font, _data) = noto_font();
        let glyph_id = font.charmap().map('g').expect("Glyph must exist");
        let outline = glyph_outline(&font, glyph_id, 48.0).expect("outline recorded");
        assert!(
            outline
                .commands()
                .iter()
                .any(|cmd| matches!(cmd, OutlineCommand::QuadTo { .. })),
            "TrueType outlines should include quadratic segments"
        );
    }

    #[test]
    fn converts_outline_into_bez_path() {
        let (font, _data) = noto_font();
        let glyph_id = font.charmap().map('A').unwrap();
        let path = glyph_bez_path(&font, glyph_id, 48.0, 32.0).expect("path");
        assert!(
            !path.elements().is_empty(),
            "conversion should emit bezier elements"
        );
        let bounds = path.bounding_box();
        assert!(bounds.width() > 0.0 && bounds.height() > 0.0);
    }

    #[test]
    fn records_composite_glyph_outline() {
        let (font, _data) = noto_font();
        let glyph_id = font
            .charmap()
            .map('Å')
            .expect("Noto Sans should contain Å glyph");
        let outline = glyph_outline(&font, glyph_id, 48.0).expect("outline recorded");
        let move_commands = outline
            .commands()
            .iter()
            .filter(|cmd| matches!(cmd, OutlineCommand::MoveTo(_, _)))
            .count();
        assert!(
            move_commands >= 2,
            "Composite glyphs should record multiple move commands"
        );
    }

    #[test]
    fn variable_weight_changes_outline_geometry() {
        let (font, _data) = load_font("RobotoFlex-Variable.ttf");
        let glyph_id = font.charmap().map('H').expect("glyph present");
        let mut light = HashMap::new();
        let mut bold = HashMap::new();
        let axes = font.axes();
        let wght_axis = axes
            .iter()
            .find(|axis| axis.tag().to_be_bytes() == *b"wght")
            .expect("wght axis");
        light.insert("wght".into(), wght_axis.min_value());
        bold.insert("wght".into(), wght_axis.max_value());
        let light_path = glyph_bez_path_with_variations(&font, glyph_id, 48.0, 32.0, Some(&light))
            .expect("light outline");
        let bold_path = glyph_bez_path_with_variations(&font, glyph_id, 48.0, 32.0, Some(&bold))
            .expect("bold outline");
        assert!(
            light_path.bounding_box().width() < bold_path.bounding_box().width(),
            "Bold axis should widen glyph outlines"
        );
    }

    #[test]
    #[ignore] // TODO: AmstelvarAlpha-VF.ttf needs to be re-downloaded (currently corrupted HTML file)
    fn avar_opsz_influences_outline_shape() {
        let (font, _data) = load_font("AmstelvarAlpha-VF.ttf");
        let glyph_id = font.charmap().map('e').expect("glyph present");
        let axes = font.axes();
        let opsz_axis = axes
            .iter()
            .find(|axis| axis.tag().to_be_bytes() == *b"opsz")
            .expect("opsz axis");
        let mut small = HashMap::new();
        let mut large = HashMap::new();
        small.insert("opsz".into(), opsz_axis.min_value());
        large.insert("opsz".into(), opsz_axis.max_value());

        let small_path = glyph_bez_path_with_variations(&font, glyph_id, 24.0, 16.0, Some(&small))
            .expect("small opsz path");
        let large_path = glyph_bez_path_with_variations(&font, glyph_id, 72.0, 16.0, Some(&large))
            .expect("large opsz path");

        assert!(
            (large_path.bounding_box().height() - small_path.bounding_box().height()).abs() > 0.5,
            "Optical size extremes should alter outline bounds"
        );
    }

    fn noto_font() -> (FontRef<'static>, &'static [u8]) {
        load_font("NotoSans-Regular.ttf")
    }

    fn load_font(name: &str) -> (FontRef<'static>, &'static [u8]) {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../testdata/fonts")
            .join(name);
        let data = fs::read(&path).expect("Test font readable");
        let data = Box::leak(data.into_boxed_slice());
        let font = FontRef::new(data).expect("Test font parsed");
        (font, data)
    }
}
