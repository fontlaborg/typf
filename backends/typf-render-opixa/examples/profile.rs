use read_fonts::TableProvider;
use skrifa::{FontRef as SkrifaFontRef, MetadataProvider};
use std::fs;
use std::sync::Arc;
use typf_core::{
    traits::{FontRef, Renderer},
    types::{Direction, PositionedGlyph, ShapingResult},
    Color, RenderParams,
};
use typf_render_opixa::OpixaRenderer;

struct SimpleFont {
    data: Vec<u8>,
}

impl SimpleFont {
    fn new(path: &str) -> Self {
        let data = fs::read(path).expect(&format!("Failed to read font file at {}", path));
        Self { data }
    }
}

impl FontRef for SimpleFont {
    fn data(&self) -> &[u8] {
        &self.data
    }

    fn units_per_em(&self) -> u16 {
        let font = SkrifaFontRef::new(&self.data).unwrap();
        font.head().unwrap().units_per_em()
    }

    fn glyph_id(&self, ch: char) -> Option<u32> {
        let font = SkrifaFontRef::new(&self.data).unwrap();
        font.charmap().map(ch).map(|g| g.to_u32())
    }

    fn advance_width(&self, glyph_id: u32) -> f32 {
        let font = SkrifaFontRef::new(&self.data).unwrap();
        let gid = skrifa::GlyphId::new(glyph_id);
        font.hmtx().unwrap().advance(gid).unwrap_or(0) as f32
    }
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let font_path =
        std::path::Path::new(manifest_dir).join("../../typf-tester/fonts/NotoSans-Regular.ttf");

    let font = Arc::new(SimpleFont::new(font_path.to_str().unwrap()));

    // Create a shaping result with some glyphs
    // "Hello World" repeated to have more work
    let text = "Hello World ".repeat(10);
    let mut glyphs = Vec::new();
    let mut x = 0.0;
    for (i, ch) in text.chars().enumerate() {
        if let Some(gid) = font.glyph_id(ch) {
            let advance = font.advance_width(gid);
            glyphs.push(PositionedGlyph {
                id: gid,
                x,
                y: 0.0,
                advance,
                cluster: i as u32,
            });
            x += advance;
        }
    }

    let shaped = ShapingResult {
        glyphs,
        advance_width: x,
        advance_height: font.units_per_em() as f32,
        direction: Direction::LeftToRight,
    };

    let params = RenderParams {
        foreground: Color::black(),
        background: Some(Color::white()),
        padding: 0,
        antialias: true,
        ..Default::default()
    };

    let renderer = OpixaRenderer::default();

    println!("Profiling rendering of {} glyphs...", shaped.glyphs.len());

    // Render multiple times for profiling
    for _ in 0..1000 {
        let _ = renderer.render(&shaped, font.clone() as Arc<dyn FontRef>, &params);
    }

    println!("Done.");
}
