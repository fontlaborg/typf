//! Visual regression tests using SSIM image comparison.
//!
//! These tests compare rendered output across different renderers using
//! the Structural Similarity Index (SSIM) to detect visual differences.
//!
//! **Test Coverage (21 tests):**
//! - Cross-renderer: All 6 pairs of 4 renderers (Opixa, Skia, Zeno, Vello-CPU)
//! - Idempotency: Opixa, Skia, Zeno, Vello-CPU (same renderer → identical output)
//! - Font sizes: Small (12pt), Large (96pt)
//! - RTL Arabic: NotoNaskhArabic with 3 renderer pairs
//! - Variable fonts: Kalnia with default instance
//! - Color fonts: COLR and SVG (Nabla) with Skia vs Zeno
//!
//! SSIM scores:
//! - 1.0 = Identical images
//! - 0.95+ = Visually indistinguishable
//! - 0.90+ = Very similar (acceptable for most use cases)
//! - 0.80+ = Similar but noticeable differences
//! - Below 0.80 = Significant differences

// this_file: crates/typf/tests/visual_regression.rs

use std::path::PathBuf;
use std::sync::Arc;

use image::GrayImage;
use image_compare::Algorithm;
use typf_core::{
    traits::{FontRef, Renderer},
    types::{BitmapData, BitmapFormat, Direction, PositionedGlyph, RenderOutput, ShapingResult},
    Color, RenderParams,
};
use typf_fontdb::TypfFontFace;
use typf_render_opixa::OpixaRenderer;
use typf_render_skia::SkiaRenderer;
use typf_render_vello_cpu::VelloCpuRenderer;
use typf_render_zeno::ZenoRenderer;

/// Minimum SSIM score for tests to pass.
/// 0.90 allows for minor antialiasing differences between renderers.
const MIN_SSIM_THRESHOLD: f64 = 0.90;

/// Higher threshold for same-renderer consistency tests.
const HIGH_SSIM_THRESHOLD: f64 = 0.99;

fn test_font_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test-fonts")
        .join(name)
}

/// Convert typf BitmapData to image_compare compatible grayscale buffer.
fn bitmap_to_grayscale(bitmap: &BitmapData) -> Option<GrayImage> {
    let width = bitmap.width;
    let height = bitmap.height;

    if width == 0 || height == 0 {
        return None;
    }

    let grayscale: Vec<u8> = match bitmap.format {
        BitmapFormat::Gray8 => bitmap.data.clone(),
        BitmapFormat::Rgba8 => {
            // Convert RGBA to grayscale using luminance formula
            bitmap
                .data
                .chunks(4)
                .map(|rgba| {
                    let r = rgba[0] as f32;
                    let g = rgba[1] as f32;
                    let b = rgba[2] as f32;
                    (0.299 * r + 0.587 * g + 0.114 * b) as u8
                })
                .collect()
        },
        _ => return None,
    };

    GrayImage::from_raw(width, height, grayscale)
}

/// Calculate SSIM between two bitmaps.
fn calculate_ssim(bitmap_a: &BitmapData, bitmap_b: &BitmapData) -> Option<f64> {
    let img_a = bitmap_to_grayscale(bitmap_a)?;
    let img_b = bitmap_to_grayscale(bitmap_b)?;

    // Images must be same size for SSIM
    if img_a.dimensions() != img_b.dimensions() {
        return None;
    }

    let result = image_compare::gray_similarity_structure(&Algorithm::MSSIMSimple, &img_a, &img_b);
    match result {
        Ok(similarity) => Some(similarity.score),
        Err(_) => None,
    }
}

/// Create a simple shaped result for testing (LTR).
fn create_test_shaped_result(font: &Arc<dyn FontRef>, text: &str, font_size: f32) -> ShapingResult {
    create_test_shaped_result_with_direction(font, text, font_size, Direction::LeftToRight)
}

/// Create a shaped result with specified direction.
fn create_test_shaped_result_with_direction(
    font: &Arc<dyn FontRef>,
    text: &str,
    font_size: f32,
    direction: Direction,
) -> ShapingResult {
    let mut x = 0.0;
    let glyphs: Vec<PositionedGlyph> = text
        .chars()
        .enumerate()
        .filter_map(|(i, ch)| {
            let glyph_id = font.glyph_id(ch)?;
            let advance = font.advance_width(glyph_id) * font_size / font.units_per_em() as f32;
            let glyph = PositionedGlyph {
                id: glyph_id,
                x,
                y: 0.0,
                advance,
                cluster: i as u32,
            };
            x += advance;
            Some(glyph)
        })
        .collect();

    let advance_width = x;

    ShapingResult {
        glyphs,
        advance_width,
        advance_height: font_size,
        direction,
    }
}

/// Render with a given renderer and return bitmap.
fn render_text<R: Renderer>(
    renderer: &R,
    font: Arc<dyn FontRef>,
    text: &str,
    font_size: f32,
) -> Option<BitmapData> {
    let shaped = create_test_shaped_result(&font, text, font_size);
    let params = RenderParams {
        foreground: Color::black(),
        background: Some(Color::white()),
        padding: 8,
        ..Default::default()
    };

    let output = renderer.render(&shaped, font, &params).ok()?;
    match output {
        RenderOutput::Bitmap(bmp) => Some(bmp),
        _ => None,
    }
}

// =============================================================================
// Cross-Renderer Visual Comparison Tests
// =============================================================================

#[test]
fn test_ssim_opixa_vs_skia_latin_text() {
    let font_path = test_font_path("NotoSans-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let opixa = OpixaRenderer::new();
    let skia = SkiaRenderer::new();

    let opixa_bmp = render_text(&opixa, font.clone(), "Hello", 48.0).expect("Opixa render");
    let skia_bmp = render_text(&skia, font.clone(), "Hello", 48.0).expect("Skia render");

    let ssim = calculate_ssim(&opixa_bmp, &skia_bmp);
    assert!(
        ssim.is_some(),
        "SSIM calculation failed (dimension mismatch?)"
    );

    let score = ssim.unwrap();
    assert!(
        score >= MIN_SSIM_THRESHOLD,
        "Opixa vs Skia SSIM {:.4} < {:.2} threshold",
        score,
        MIN_SSIM_THRESHOLD
    );

    eprintln!("Opixa vs Skia SSIM: {:.4}", score);
}

#[test]
fn test_ssim_opixa_vs_zeno_latin_text() {
    let font_path = test_font_path("NotoSans-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let opixa = OpixaRenderer::new();
    let zeno = ZenoRenderer::new();

    let opixa_bmp = render_text(&opixa, font.clone(), "Hello", 48.0).expect("Opixa render");
    let zeno_bmp = render_text(&zeno, font.clone(), "Hello", 48.0).expect("Zeno render");

    let ssim = calculate_ssim(&opixa_bmp, &zeno_bmp);
    assert!(
        ssim.is_some(),
        "SSIM calculation failed (dimension mismatch?)"
    );

    let score = ssim.unwrap();
    assert!(
        score >= MIN_SSIM_THRESHOLD,
        "Opixa vs Zeno SSIM {:.4} < {:.2} threshold",
        score,
        MIN_SSIM_THRESHOLD
    );

    eprintln!("Opixa vs Zeno SSIM: {:.4}", score);
}

#[test]
fn test_ssim_skia_vs_zeno_latin_text() {
    let font_path = test_font_path("NotoSans-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let skia = SkiaRenderer::new();
    let zeno = ZenoRenderer::new();

    let skia_bmp = render_text(&skia, font.clone(), "Hello", 48.0).expect("Skia render");
    let zeno_bmp = render_text(&zeno, font.clone(), "Hello", 48.0).expect("Zeno render");

    let ssim = calculate_ssim(&skia_bmp, &zeno_bmp);
    assert!(
        ssim.is_some(),
        "SSIM calculation failed (dimension mismatch?)"
    );

    let score = ssim.unwrap();
    // Skia and Zeno have different rasterization algorithms; allow 0.85 threshold
    const SKIA_ZENO_THRESHOLD: f64 = 0.85;
    assert!(
        score >= SKIA_ZENO_THRESHOLD,
        "Skia vs Zeno SSIM {:.4} < {:.2} threshold",
        score,
        SKIA_ZENO_THRESHOLD
    );

    eprintln!("Skia vs Zeno SSIM: {:.4}", score);
}

#[test]
fn test_ssim_skia_vs_vello_cpu_latin_text() {
    let font_path = test_font_path("NotoSans-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let skia = SkiaRenderer::new();
    let vello_cpu = VelloCpuRenderer::new();

    let skia_bmp = render_text(&skia, font.clone(), "Hello", 48.0).expect("Skia render");
    let vello_bmp = render_text(&vello_cpu, font.clone(), "Hello", 48.0).expect("Vello CPU render");

    let ssim = calculate_ssim(&skia_bmp, &vello_bmp);
    assert!(
        ssim.is_some(),
        "SSIM calculation failed (dimension mismatch?)"
    );

    let score = ssim.unwrap();
    assert!(
        score >= MIN_SSIM_THRESHOLD,
        "Skia vs Vello-CPU SSIM {:.4} < {:.2} threshold",
        score,
        MIN_SSIM_THRESHOLD
    );

    eprintln!("Skia vs Vello-CPU SSIM: {:.4}", score);
}

#[test]
fn test_ssim_opixa_vs_vello_cpu_latin_text() {
    let font_path = test_font_path("NotoSans-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let opixa = OpixaRenderer::new();
    let vello_cpu = VelloCpuRenderer::new();

    let opixa_bmp = render_text(&opixa, font.clone(), "Hello", 48.0).expect("Opixa render");
    let vello_bmp = render_text(&vello_cpu, font.clone(), "Hello", 48.0).expect("Vello CPU render");

    let ssim = calculate_ssim(&opixa_bmp, &vello_bmp);
    assert!(
        ssim.is_some(),
        "SSIM calculation failed (dimension mismatch?)"
    );

    let score = ssim.unwrap();
    assert!(
        score >= MIN_SSIM_THRESHOLD,
        "Opixa vs Vello-CPU SSIM {:.4} < {:.2} threshold",
        score,
        MIN_SSIM_THRESHOLD
    );

    eprintln!("Opixa vs Vello-CPU SSIM: {:.4}", score);
}

#[test]
fn test_ssim_zeno_vs_vello_cpu_latin_text() {
    let font_path = test_font_path("NotoSans-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let zeno = ZenoRenderer::new();
    let vello_cpu = VelloCpuRenderer::new();

    let zeno_bmp = render_text(&zeno, font.clone(), "Hello", 48.0).expect("Zeno render");
    let vello_bmp = render_text(&vello_cpu, font.clone(), "Hello", 48.0).expect("Vello CPU render");

    let ssim = calculate_ssim(&zeno_bmp, &vello_bmp);
    assert!(
        ssim.is_some(),
        "SSIM calculation failed (dimension mismatch?)"
    );

    let score = ssim.unwrap();
    assert!(
        score >= MIN_SSIM_THRESHOLD,
        "Zeno vs Vello-CPU SSIM {:.4} < {:.2} threshold",
        score,
        MIN_SSIM_THRESHOLD
    );

    eprintln!("Zeno vs Vello-CPU SSIM: {:.4}", score);
}

// =============================================================================
// Same-Renderer Consistency Tests
// =============================================================================

#[test]
fn test_ssim_opixa_idempotent() {
    let font_path = test_font_path("NotoSans-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let opixa = OpixaRenderer::new();

    let bmp1 = render_text(&opixa, font.clone(), "Test", 32.0).expect("First render");
    let bmp2 = render_text(&opixa, font.clone(), "Test", 32.0).expect("Second render");

    let ssim = calculate_ssim(&bmp1, &bmp2);
    assert!(ssim.is_some(), "SSIM calculation failed");

    let score = ssim.unwrap();
    assert!(
        score >= HIGH_SSIM_THRESHOLD,
        "Same renderer should produce identical output: SSIM {:.4}",
        score
    );
}

#[test]
fn test_ssim_vello_cpu_idempotent() {
    let font_path = test_font_path("NotoSans-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let vello_cpu = VelloCpuRenderer::new();

    let bmp1 = render_text(&vello_cpu, font.clone(), "Test", 32.0).expect("First render");
    let bmp2 = render_text(&vello_cpu, font.clone(), "Test", 32.0).expect("Second render");

    let ssim = calculate_ssim(&bmp1, &bmp2);
    assert!(ssim.is_some(), "SSIM calculation failed");

    let score = ssim.unwrap();
    assert!(
        score >= HIGH_SSIM_THRESHOLD,
        "Vello-CPU same renderer should produce identical output: SSIM {:.4}",
        score
    );
}

#[test]
fn test_ssim_skia_idempotent() {
    let font_path = test_font_path("NotoSans-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let skia = SkiaRenderer::new();

    let bmp1 = render_text(&skia, font.clone(), "Test", 32.0).expect("First render");
    let bmp2 = render_text(&skia, font.clone(), "Test", 32.0).expect("Second render");

    let ssim = calculate_ssim(&bmp1, &bmp2);
    assert!(ssim.is_some(), "SSIM calculation failed");

    let score = ssim.unwrap();
    assert!(
        score >= HIGH_SSIM_THRESHOLD,
        "Skia same renderer should produce identical output: SSIM {:.4}",
        score
    );
}

#[test]
fn test_ssim_zeno_idempotent() {
    let font_path = test_font_path("NotoSans-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let zeno = ZenoRenderer::new();

    let bmp1 = render_text(&zeno, font.clone(), "Test", 32.0).expect("First render");
    let bmp2 = render_text(&zeno, font.clone(), "Test", 32.0).expect("Second render");

    let ssim = calculate_ssim(&bmp1, &bmp2);
    assert!(ssim.is_some(), "SSIM calculation failed");

    let score = ssim.unwrap();
    assert!(
        score >= HIGH_SSIM_THRESHOLD,
        "Zeno same renderer should produce identical output: SSIM {:.4}",
        score
    );
}

// =============================================================================
// Different Font Size Tests
// =============================================================================

#[test]
fn test_ssim_renderers_small_text() {
    let font_path = test_font_path("NotoSans-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let opixa = OpixaRenderer::new();
    let skia = SkiaRenderer::new();

    // Small text is harder to match due to antialiasing
    let opixa_bmp = render_text(&opixa, font.clone(), "Small", 12.0).expect("Opixa render");
    let skia_bmp = render_text(&skia, font.clone(), "Small", 12.0).expect("Skia render");

    let ssim = calculate_ssim(&opixa_bmp, &skia_bmp);
    if let Some(score) = ssim {
        eprintln!("Small text (12pt) Opixa vs Skia SSIM: {:.4}", score);
        // Lower threshold for small text due to antialiasing differences
        assert!(
            score >= 0.85,
            "Small text SSIM {:.4} below 0.85 threshold",
            score
        );
    }
}

#[test]
fn test_ssim_renderers_large_text() {
    let font_path = test_font_path("NotoSans-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let opixa = OpixaRenderer::new();
    let skia = SkiaRenderer::new();

    // Large text should match better
    let opixa_bmp = render_text(&opixa, font.clone(), "Large", 96.0).expect("Opixa render");
    let skia_bmp = render_text(&skia, font.clone(), "Large", 96.0).expect("Skia render");

    let ssim = calculate_ssim(&opixa_bmp, &skia_bmp);
    if let Some(score) = ssim {
        eprintln!("Large text (96pt) Opixa vs Skia SSIM: {:.4}", score);
        assert!(
            score >= MIN_SSIM_THRESHOLD,
            "Large text SSIM {:.4} below threshold",
            score
        );
    }
}

// =============================================================================
// RTL Arabic Text Tests
// =============================================================================

/// Render RTL text with direction hint.
fn render_rtl_text<R: Renderer>(
    renderer: &R,
    font: Arc<dyn FontRef>,
    text: &str,
    font_size: f32,
) -> Option<BitmapData> {
    let shaped =
        create_test_shaped_result_with_direction(&font, text, font_size, Direction::RightToLeft);
    let params = RenderParams {
        foreground: Color::black(),
        background: Some(Color::white()),
        padding: 8,
        ..Default::default()
    };

    let output = renderer.render(&shaped, font, &params).ok()?;
    match output {
        RenderOutput::Bitmap(bmp) => Some(bmp),
        _ => None,
    }
}

#[test]
fn test_ssim_opixa_vs_skia_arabic_text() {
    let font_path = test_font_path("NotoNaskhArabic-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: Arabic font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load Arabic font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let opixa = OpixaRenderer::new();
    let skia = SkiaRenderer::new();

    // Arabic text (مرحبا = "Hello")
    // Note: Without proper shaping, individual glyphs are used
    let opixa_bmp =
        render_rtl_text(&opixa, font.clone(), "مرحبا", 48.0).expect("Opixa Arabic render");
    let skia_bmp = render_rtl_text(&skia, font.clone(), "مرحبا", 48.0).expect("Skia Arabic render");

    let ssim = calculate_ssim(&opixa_bmp, &skia_bmp);
    assert!(
        ssim.is_some(),
        "SSIM calculation failed for Arabic text (dimension mismatch?)"
    );

    let score = ssim.unwrap();
    eprintln!("Arabic text Opixa vs Skia SSIM: {:.4}", score);
    assert!(
        score >= MIN_SSIM_THRESHOLD,
        "Arabic text SSIM {:.4} < {:.2} threshold",
        score,
        MIN_SSIM_THRESHOLD
    );
}

#[test]
fn test_ssim_opixa_vs_zeno_arabic_text() {
    let font_path = test_font_path("NotoNaskhArabic-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: Arabic font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load Arabic font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let opixa = OpixaRenderer::new();
    let zeno = ZenoRenderer::new();

    let opixa_bmp =
        render_rtl_text(&opixa, font.clone(), "مرحبا", 48.0).expect("Opixa Arabic render");
    let zeno_bmp = render_rtl_text(&zeno, font.clone(), "مرحبا", 48.0).expect("Zeno Arabic render");

    let ssim = calculate_ssim(&opixa_bmp, &zeno_bmp);
    assert!(ssim.is_some(), "SSIM calculation failed for Arabic text");

    let score = ssim.unwrap();
    eprintln!("Arabic text Opixa vs Zeno SSIM: {:.4}", score);
    // Lower threshold for complex script rendering differences (Zeno vs Opixa)
    assert!(
        score >= 0.80,
        "Arabic text SSIM {:.4} < 0.80 threshold",
        score
    );
}

#[test]
fn test_ssim_skia_vs_vello_cpu_arabic_text() {
    let font_path = test_font_path("NotoNaskhArabic-Regular.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: Arabic font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load Arabic font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let skia = SkiaRenderer::new();
    let vello_cpu = VelloCpuRenderer::new();

    let skia_bmp = render_rtl_text(&skia, font.clone(), "مرحبا", 48.0).expect("Skia Arabic render");
    let vello_bmp =
        render_rtl_text(&vello_cpu, font.clone(), "مرحبا", 48.0).expect("Vello-CPU Arabic render");

    let ssim = calculate_ssim(&skia_bmp, &vello_bmp);
    assert!(ssim.is_some(), "SSIM calculation failed for Arabic text");

    let score = ssim.unwrap();
    eprintln!("Arabic text Skia vs Vello-CPU SSIM: {:.4}", score);
    // Lower threshold for complex script + renderer differences
    assert!(
        score >= 0.80,
        "Arabic text SSIM {:.4} < 0.80 threshold",
        score
    );
}

// =============================================================================
// Variable Font Tests
// =============================================================================

#[test]
fn test_ssim_variable_font_opixa_vs_skia() {
    let font_path = test_font_path("Kalnia[wdth,wght].ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: Variable font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load variable font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let opixa = OpixaRenderer::new();
    let skia = SkiaRenderer::new();

    // Default instance (no variations applied)
    let opixa_bmp =
        render_text(&opixa, font.clone(), "Variable", 48.0).expect("Opixa variable render");
    let skia_bmp =
        render_text(&skia, font.clone(), "Variable", 48.0).expect("Skia variable render");

    let ssim = calculate_ssim(&opixa_bmp, &skia_bmp);
    assert!(ssim.is_some(), "SSIM calculation failed for variable font");

    let score = ssim.unwrap();
    eprintln!("Variable font Opixa vs Skia SSIM: {:.4}", score);
    // Lower threshold for variable fonts (outline interpolation differences)
    assert!(
        score >= 0.80,
        "Variable font SSIM {:.4} < 0.80 threshold",
        score
    );
}

#[test]
fn test_ssim_variable_font_idempotent() {
    let font_path = test_font_path("Kalnia[wdth,wght].ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: Variable font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load variable font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let opixa = OpixaRenderer::new();

    let bmp1 = render_text(&opixa, font.clone(), "Test", 32.0).expect("First render");
    let bmp2 = render_text(&opixa, font.clone(), "Test", 32.0).expect("Second render");

    let ssim = calculate_ssim(&bmp1, &bmp2);
    assert!(ssim.is_some(), "SSIM calculation failed");

    let score = ssim.unwrap();
    assert!(
        score >= HIGH_SSIM_THRESHOLD,
        "Variable font same renderer should produce identical output: SSIM {:.4}",
        score
    );
}

// =============================================================================
// Color Font Tests (COLR/SVG)
// =============================================================================

#[test]
fn test_ssim_colr_font_skia_vs_zeno() {
    let font_path = test_font_path("Nabla-Regular-COLR.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: COLR font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load COLR font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let skia = SkiaRenderer::new();
    let zeno = ZenoRenderer::new();

    // Nabla is a display font - use simple ASCII that maps to glyphs
    let skia_bmp = render_text(&skia, font.clone(), "ABC", 64.0).expect("Skia COLR render");
    let zeno_bmp = render_text(&zeno, font.clone(), "ABC", 64.0).expect("Zeno COLR render");

    let ssim = calculate_ssim(&skia_bmp, &zeno_bmp);
    assert!(ssim.is_some(), "SSIM calculation failed for COLR font");

    let score = ssim.unwrap();
    eprintln!("COLR font Skia vs Zeno SSIM: {:.4}", score);
    // Color fonts may have more variation due to compositing differences
    assert!(
        score >= 0.75,
        "COLR font SSIM {:.4} < 0.75 threshold",
        score
    );
}

#[test]
fn test_ssim_colr_font_skia_idempotent() {
    let font_path = test_font_path("Nabla-Regular-COLR.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: COLR font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load COLR font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let skia = SkiaRenderer::new();

    let bmp1 = render_text(&skia, font.clone(), "XYZ", 48.0).expect("First COLR render");
    let bmp2 = render_text(&skia, font.clone(), "XYZ", 48.0).expect("Second COLR render");

    let ssim = calculate_ssim(&bmp1, &bmp2);
    assert!(ssim.is_some(), "SSIM calculation failed");

    let score = ssim.unwrap();
    assert!(
        score >= HIGH_SSIM_THRESHOLD,
        "COLR font same renderer should produce identical output: SSIM {:.4}",
        score
    );
}

#[test]
fn test_ssim_svg_font_skia_vs_zeno() {
    let font_path = test_font_path("Nabla-Regular-SVG.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: SVG font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load SVG font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let skia = SkiaRenderer::new();
    let zeno = ZenoRenderer::new();

    let skia_bmp = render_text(&skia, font.clone(), "ABC", 64.0).expect("Skia SVG render");
    let zeno_bmp = render_text(&zeno, font.clone(), "ABC", 64.0).expect("Zeno SVG render");

    let ssim = calculate_ssim(&skia_bmp, &zeno_bmp);
    assert!(ssim.is_some(), "SSIM calculation failed for SVG font");

    let score = ssim.unwrap();
    eprintln!("SVG font Skia vs Zeno SSIM: {:.4}", score);
    // SVG fonts may have significant renderer differences
    assert!(score >= 0.70, "SVG font SSIM {:.4} < 0.70 threshold", score);
}

#[test]
fn test_ssim_svg_font_skia_idempotent() {
    let font_path = test_font_path("Nabla-Regular-SVG.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: SVG font not found at {:?}", font_path);
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load SVG font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let skia = SkiaRenderer::new();

    let bmp1 = render_text(&skia, font.clone(), "XYZ", 48.0).expect("First SVG render");
    let bmp2 = render_text(&skia, font.clone(), "XYZ", 48.0).expect("Second SVG render");

    let ssim = calculate_ssim(&bmp1, &bmp2);
    assert!(ssim.is_some(), "SSIM calculation failed");

    let score = ssim.unwrap();
    assert!(
        score >= HIGH_SSIM_THRESHOLD,
        "SVG font same renderer should produce identical output: SSIM {:.4}",
        score
    );
}
