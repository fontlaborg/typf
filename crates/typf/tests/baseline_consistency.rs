//! Cross-renderer baseline contract regression tests

// this_file: crates/typf/tests/baseline_consistency.rs

use std::path::PathBuf;
use std::sync::Arc;

use typf_core::{
    traits::{FontRef, Renderer},
    types::{Direction, PositionedGlyph, RenderOutput, ShapingResult},
    RenderParams,
};
use typf_fontdb::TypfFontFace;
use typf_render_skia::SkiaRenderer;
use typf_render_vello_cpu::VelloCpuRenderer;

fn test_font_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test-fonts")
        .join(name)
}

#[test]
fn test_baseline_height_when_same_font_then_renderers_match() {
    let font_path = test_font_path("NotoSans-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let font_size = 64.0;
    let glyph_id = font.glyph_id('H').unwrap_or(0);

    let shaped = ShapingResult {
        glyphs: vec![PositionedGlyph {
            id: glyph_id,
            x: 0.0,
            y: 0.0,
            advance: 0.0,
            cluster: 0,
        }],
        advance_width: font_size * 2.0,
        advance_height: font_size,
        direction: Direction::LeftToRight,
    };

    let params = RenderParams {
        padding: 4,
        ..Default::default()
    };

    let skia = SkiaRenderer::new();
    let vello_cpu = VelloCpuRenderer::new();

    let skia_out = skia
        .render(&shaped, font.clone(), &params)
        .expect("Skia render");
    let vello_out = vello_cpu
        .render(&shaped, font.clone(), &params)
        .expect("Vello CPU render");

    let RenderOutput::Bitmap(skia_bmp) = skia_out else {
        panic!("Expected Skia bitmap output");
    };
    let RenderOutput::Bitmap(vello_bmp) = vello_out else {
        panic!("Expected Vello CPU bitmap output");
    };

    assert_eq!(
        skia_bmp.height, vello_bmp.height,
        "metrics-first baseline contract: heights should match for same font/size"
    );
}
