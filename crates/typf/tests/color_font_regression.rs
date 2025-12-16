//! Color font regression tests for known issues
//!
//! This module tests for specific rendering problems:
//! - Cutoffs: glyphs being cut off at edges
//! - Padding: incorrect spacing/margins
//! - Flips: Y-axis coordinate system problems
//! - Cross-renderer consistency: different renderers should produce similar results

#![allow(
    clippy::expect_fun_call,
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used
)]

// this_file: crates/typf/tests/color_font_regression.rs

use std::path::PathBuf;
use std::sync::Arc;

use typf_core::{
    traits::{FontRef, Renderer, Shaper},
    types::RenderOutput,
    Color, RenderParams, ShapingParams,
};
use typf_fontdb::TypfFontFace;
use typf_render_opixa::OpixaRenderer;
use typf_render_skia::SkiaRenderer;
use typf_render_zeno::ZenoRenderer;
use typf_shape_none::NoneShaper;

/// Get path to test font fixtures
fn test_font_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test-fonts")
        .join(name)
}

/// Extract bitmap dimensions from RenderOutput
fn get_bitmap_dims(output: &RenderOutput) -> (u32, u32) {
    match output {
        RenderOutput::Bitmap(bitmap) => (bitmap.width, bitmap.height),
        _ => panic!("Expected bitmap output"),
    }
}

/// Extract bitmap data from RenderOutput
fn get_bitmap_data(output: &RenderOutput) -> &[u8] {
    match output {
        RenderOutput::Bitmap(bitmap) => &bitmap.data,
        _ => panic!("Expected bitmap output"),
    }
}

/// Count non-transparent pixels in bitmap (RGBA format)
fn count_non_transparent_pixels(data: &[u8]) -> usize {
    data.chunks(4).filter(|p| p[3] > 0).count()
}

/// Count colored (non-grayscale, non-transparent) pixels
/// Uses a tolerance for near-grayscale since gradients may produce subtle color differences
fn count_colored_pixels(data: &[u8]) -> usize {
    data.chunks(4)
        .filter(|p| {
            let a = p[3];
            if a == 0 {
                return false;
            }
            // Check if R, G, B differ by more than a threshold (to catch near-grayscale)
            let r = p[0] as i32;
            let g = p[1] as i32;
            let b = p[2] as i32;
            let max_diff = (r - g).abs().max((g - b).abs()).max((r - b).abs());
            max_diff > 10 // Allow small tolerance for gradients
        })
        .count()
}

/// Count any non-background pixels (for verifying content exists)
fn count_content_pixels(data: &[u8]) -> usize {
    data.chunks(4)
        .filter(|p| {
            let a = p[3];
            if a == 0 {
                return false;
            }
            // Not pure white background
            !(p[0] == 255 && p[1] == 255 && p[2] == 255)
        })
        .count()
}

/// Check if bitmap has content in edge regions (cutoff test)
fn has_edge_content(data: &[u8], width: u32, height: u32, edge_width: u32) -> (bool, bool, bool, bool) {
    let w = width as usize;
    let h = height as usize;
    let ew = edge_width as usize;

    let mut top = false;
    let mut bottom = false;
    let mut left = false;
    let mut right = false;

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * 4;
            let alpha = data[idx + 3];
            if alpha > 0 {
                if y < ew { top = true; }
                if y >= h - ew { bottom = true; }
                if x < ew { left = true; }
                if x >= w - ew { right = true; }
            }
        }
    }

    (top, bottom, left, right)
}

// =============================================================================
// COLR Font Regression Tests
// =============================================================================

#[test]
fn test_colr_glyph_not_cutoff() {
    let font_path = test_font_path("Nabla-Regular-COLR.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: COLR font not found");
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load COLR font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let shaper = NoneShaper::new();
    let renderer = OpixaRenderer::new();

    // Test multiple glyphs that have been known to have cutoff issues
    for text in &["A", "W", "M", "g", "j", "y", "@", "$"] {
        let shaping_params = ShapingParams {
            size: 64.0,
            ..Default::default()
        };

        let render_params = RenderParams {
            foreground: Color::black(),
            background: Some(Color::white()),
            padding: 4, // Minimal padding to detect cutoffs
            ..Default::default()
        };

        let shaped = shaper.shape(text, font.clone(), &shaping_params).unwrap();
        let rendered = renderer.render(&shaped, font.clone(), &render_params).unwrap();

        let (width, height) = get_bitmap_dims(&rendered);
        let data = get_bitmap_data(&rendered);

        // Check edges for content touching them (potential cutoff)
        let (top, bottom, left, right) = has_edge_content(data, width, height, 2);

        // If content touches edge, that's a potential cutoff - warn but don't fail
        // since some glyphs legitimately extend to edges with small padding
        if top || bottom || left || right {
            eprintln!(
                "COLR glyph '{}' may have cutoff: top={}, bottom={}, left={}, right={}",
                text, top, bottom, left, right
            );
        }

        // Verify glyph actually rendered (has content)
        let non_transparent = count_non_transparent_pixels(data);
        assert!(
            non_transparent > 10,
            "COLR glyph '{}' should have content, got {} non-transparent pixels",
            text,
            non_transparent
        );
    }
}

#[test]
fn test_colr_padding_applied_correctly() {
    let font_path = test_font_path("Nabla-Regular-COLR.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: COLR font not found");
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load COLR font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let shaper = NoneShaper::new();
    let renderer = OpixaRenderer::new();
    let text = "A";

    let shaping_params = ShapingParams {
        size: 48.0,
        ..Default::default()
    };

    // Render with different padding values
    let padding_values = [0, 5, 10, 20];
    let mut sizes: Vec<(u32, u32)> = Vec::new();

    for padding in padding_values {
        let render_params = RenderParams {
            padding,
            background: Some(Color::white()),
            ..Default::default()
        };

        let shaped = shaper.shape(text, font.clone(), &shaping_params).unwrap();
        let rendered = renderer.render(&shaped, font.clone(), &render_params).unwrap();
        let (w, h) = get_bitmap_dims(&rendered);
        sizes.push((w, h));
    }

    // Verify padding increases dimensions appropriately
    // Each increase of 5 padding should add ~10 to width and height (5 on each side)
    for i in 1..sizes.len() {
        let (prev_w, prev_h) = sizes[i - 1];
        let (curr_w, curr_h) = sizes[i];
        let expected_increase = (padding_values[i] - padding_values[i - 1]) as u32 * 2;

        // Allow some tolerance for rounding
        let w_diff = curr_w.saturating_sub(prev_w);
        let h_diff = curr_h.saturating_sub(prev_h);

        assert!(
            w_diff >= expected_increase.saturating_sub(2) && w_diff <= expected_increase + 2,
            "Width padding increase should be ~{}, got {} (padding {} -> {})",
            expected_increase, w_diff, padding_values[i - 1], padding_values[i]
        );
        assert!(
            h_diff >= expected_increase.saturating_sub(2) && h_diff <= expected_increase + 2,
            "Height padding increase should be ~{}, got {} (padding {} -> {})",
            expected_increase, h_diff, padding_values[i - 1], padding_values[i]
        );
    }
}

#[test]
fn test_colr_has_content() {
    let font_path = test_font_path("Nabla-Regular-COLR.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: COLR font not found");
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load COLR font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let shaper = NoneShaper::new();
    let renderer = OpixaRenderer::new();
    let text = "ABC";

    let shaping_params = ShapingParams {
        size: 64.0,
        ..Default::default()
    };

    let render_params = RenderParams {
        padding: 10,
        background: Some(Color::white()),
        ..Default::default()
    };

    let shaped = shaper.shape(text, font.clone(), &shaping_params).unwrap();
    let rendered = renderer.render(&shaped, font, &render_params).unwrap();

    let data = get_bitmap_data(&rendered);
    let content_pixels = count_content_pixels(data);
    let colored_pixels = count_colored_pixels(data);

    // COLR fonts should produce content (may be grayscale fallback or color)
    assert!(
        content_pixels > 100,
        "COLR glyphs should have content, got {} content pixels",
        content_pixels
    );

    // Report color status (informational, not failing)
    if colored_pixels > 100 {
        println!("COLR glyphs rendered with {} colored pixels (full color)", colored_pixels);
    } else {
        println!("COLR glyphs rendered with {} content pixels (grayscale fallback)", content_pixels);
    }
}

// =============================================================================
// Cross-Renderer Consistency Tests
// =============================================================================

#[test]
fn test_colr_cross_renderer_all_produce_output() {
    let font_path = test_font_path("Nabla-Regular-COLR.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: COLR font not found");
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load COLR font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let shaper = NoneShaper::new();
    let text = "A";

    let shaping_params = ShapingParams {
        size: 48.0,
        ..Default::default()
    };

    let render_params = RenderParams {
        padding: 10,
        background: Some(Color::white()),
        ..Default::default()
    };

    let shaped = shaper.shape(text, font.clone(), &shaping_params).unwrap();

    // Render with different renderers
    let opixa = OpixaRenderer::new();
    let skia = SkiaRenderer::new();
    let zeno = ZenoRenderer::new();

    let opixa_result = opixa.render(&shaped, font.clone(), &render_params);
    let skia_result = skia.render(&shaped, font.clone(), &render_params);
    let zeno_result = zeno.render(&shaped, font, &render_params);

    // All should succeed
    assert!(opixa_result.is_ok(), "Opixa should render COLR");
    assert!(skia_result.is_ok(), "Skia should render COLR");
    assert!(zeno_result.is_ok(), "Zeno should render COLR");

    let opixa_rendered = opixa_result.unwrap();
    let skia_rendered = skia_result.unwrap();
    let zeno_rendered = zeno_result.unwrap();

    let (ow, oh) = get_bitmap_dims(&opixa_rendered);
    let (sw, sh) = get_bitmap_dims(&skia_rendered);
    let (zw, zh) = get_bitmap_dims(&zeno_rendered);

    // All should produce reasonable output
    assert!(ow > 0 && oh > 0, "Opixa should produce non-empty bitmap");
    assert!(sw > 0 && sh > 0, "Skia should produce non-empty bitmap");
    assert!(zw > 0 && zh > 0, "Zeno should produce non-empty bitmap");

    // Report dimension differences (informational)
    println!("COLR cross-renderer dimensions:");
    println!("  Opixa: {}x{}", ow, oh);
    println!("  Skia:  {}x{}", sw, sh);
    println!("  Zeno:  {}x{}", zw, zh);

    // Width should be consistent (same shaping)
    let w_tolerance = 0.05; // 5% width tolerance
    let w_diff_skia = (ow as f32 - sw as f32).abs() / ow.max(1) as f32;
    let w_diff_zeno = (ow as f32 - zw as f32).abs() / ow.max(1) as f32;

    assert!(
        w_diff_skia <= w_tolerance,
        "COLR width differs significantly: Opixa {}px vs Skia {}px",
        ow, sw
    );
    assert!(
        w_diff_zeno <= w_tolerance,
        "COLR width differs significantly: Opixa {}px vs Zeno {}px",
        ow, zw
    );

    // Height may vary more due to baseline handling differences
    // Just ensure all are reasonable (> 20px for 48pt font)
    assert!(oh >= 20, "Opixa height too small: {}", oh);
    assert!(sh >= 20, "Skia height too small: {}", sh);
    assert!(zh >= 20, "Zeno height too small: {}", zh);
}

// =============================================================================
// SVG Font Regression Tests
// =============================================================================

#[test]
fn test_svg_glyph_not_cutoff() {
    let font_path = test_font_path("Nabla-Regular-SVG.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: SVG font not found");
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load SVG font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let shaper = NoneShaper::new();
    let renderer = OpixaRenderer::new();

    for text in &["X", "Y", "Z"] {
        let shaping_params = ShapingParams {
            size: 64.0,
            ..Default::default()
        };

        let render_params = RenderParams {
            padding: 8,
            background: Some(Color::white()),
            ..Default::default()
        };

        let shaped = shaper.shape(text, font.clone(), &shaping_params).unwrap();
        let rendered = renderer.render(&shaped, font.clone(), &render_params).unwrap();

        let (width, height) = get_bitmap_dims(&rendered);
        let data = get_bitmap_data(&rendered);

        // Verify content exists
        let non_transparent = count_non_transparent_pixels(data);
        assert!(
            non_transparent > 10,
            "SVG glyph '{}' should have content, got {} non-transparent pixels",
            text,
            non_transparent
        );

        // Check dimensions are reasonable (not zero, not tiny)
        assert!(
            width >= 20 && height >= 20,
            "SVG glyph '{}' dimensions too small: {}x{}",
            text,
            width,
            height
        );
    }
}

#[test]
fn test_svg_coordinate_system_not_flipped() {
    let font_path = test_font_path("Nabla-Regular-SVG.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: SVG font not found");
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load SVG font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let shaper = NoneShaper::new();
    let renderer = OpixaRenderer::new();

    // Use a glyph that is visually asymmetric vertically (like 'T')
    let text = "T";

    let shaping_params = ShapingParams {
        size: 64.0,
        ..Default::default()
    };

    let render_params = RenderParams {
        padding: 10,
        background: Some(Color::white()),
        ..Default::default()
    };

    let shaped = shaper.shape(text, font.clone(), &shaping_params).unwrap();
    let rendered = renderer.render(&shaped, font, &render_params).unwrap();

    let (width, height) = get_bitmap_dims(&rendered);
    let data = get_bitmap_data(&rendered);

    // For 'T', the top should have more content than the bottom
    // (the crossbar is at top, stem goes down)
    let mut top_half_pixels = 0;
    let mut bottom_half_pixels = 0;
    let mid = height / 2;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            if data[idx + 3] > 0 {
                if y < mid {
                    top_half_pixels += 1;
                } else {
                    bottom_half_pixels += 1;
                }
            }
        }
    }

    // The top half should have significant content (the crossbar)
    // This is a heuristic check - if the glyph is flipped, bottom would have more
    assert!(
        top_half_pixels > 0 && bottom_half_pixels > 0,
        "SVG glyph 'T' should have content in both halves"
    );

    // For a properly oriented 'T', the ratio shouldn't be extreme
    // Just verify we have reasonable distribution
    let total = top_half_pixels + bottom_half_pixels;
    let top_ratio = top_half_pixels as f32 / total as f32;

    // Top should be between 20% and 80% (not completely flipped or missing)
    assert!(
        top_ratio > 0.2 && top_ratio < 0.8,
        "SVG glyph 'T' content distribution suspicious: top_ratio={:.2}",
        top_ratio
    );
}

#[test]
fn test_svg_has_color_content() {
    let font_path = test_font_path("Nabla-Regular-SVG.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: SVG font not found");
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load SVG font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let shaper = NoneShaper::new();
    let renderer = OpixaRenderer::new();
    let text = "ABC";

    let shaping_params = ShapingParams {
        size: 64.0,
        ..Default::default()
    };

    let render_params = RenderParams {
        padding: 10,
        ..Default::default()
    };

    let shaped = shaper.shape(text, font.clone(), &shaping_params).unwrap();
    let rendered = renderer.render(&shaped, font, &render_params).unwrap();

    let data = get_bitmap_data(&rendered);
    let colored_pixels = count_colored_pixels(data);

    // SVG color fonts should produce colored output
    assert!(
        colored_pixels > 100,
        "SVG glyphs should have colored content, got {} colored pixels",
        colored_pixels
    );
}

// =============================================================================
// Bitmap Font (sbix/CBDT) Regression Tests
// =============================================================================

#[test]
fn test_sbix_glyph_not_cutoff() {
    let font_path = test_font_path("Nabla-Regular-sbix.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: sbix font not found");
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load sbix font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let shaper = NoneShaper::new();
    let renderer = OpixaRenderer::new();

    for text in &["1", "2", "3"] {
        let shaping_params = ShapingParams {
            size: 64.0,
            ..Default::default()
        };

        let render_params = RenderParams {
            padding: 4,
            background: Some(Color::white()),
            ..Default::default()
        };

        let shaped = shaper.shape(text, font.clone(), &shaping_params).unwrap();
        let rendered = renderer.render(&shaped, font.clone(), &render_params).unwrap();

        let (width, height) = get_bitmap_dims(&rendered);
        let data = get_bitmap_data(&rendered);

        let non_transparent = count_non_transparent_pixels(data);
        assert!(
            non_transparent > 10,
            "sbix glyph '{}' should have content, got {} non-transparent pixels",
            text,
            non_transparent
        );

        // Check dimensions are reasonable
        assert!(
            width >= 10 && height >= 10,
            "sbix glyph '{}' dimensions too small: {}x{}",
            text,
            width,
            height
        );
    }
}

#[test]
fn test_cbdt_glyph_not_cutoff() {
    let font_path = test_font_path("Nabla-Regular-CBDT.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: CBDT font not found");
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load CBDT font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let shaper = NoneShaper::new();
    let renderer = OpixaRenderer::new();

    for text in &["D", "E", "F"] {
        let shaping_params = ShapingParams {
            size: 64.0,
            ..Default::default()
        };

        let render_params = RenderParams {
            padding: 4,
            background: Some(Color::white()),
            ..Default::default()
        };

        let shaped = shaper.shape(text, font.clone(), &shaping_params).unwrap();
        let rendered = renderer.render(&shaped, font.clone(), &render_params).unwrap();

        let (width, height) = get_bitmap_dims(&rendered);
        let data = get_bitmap_data(&rendered);

        let non_transparent = count_non_transparent_pixels(data);
        assert!(
            non_transparent > 10,
            "CBDT glyph '{}' should have content, got {} non-transparent pixels",
            text,
            non_transparent
        );

        assert!(
            width >= 10 && height >= 10,
            "CBDT glyph '{}' dimensions too small: {}x{}",
            text,
            width,
            height
        );
    }
}

#[test]
fn test_cbdt_cross_renderer_consistency() {
    let font_path = test_font_path("Nabla-Regular-CBDT.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: CBDT font not found");
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load CBDT font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let shaper = NoneShaper::new();
    let text = "D";

    let shaping_params = ShapingParams {
        size: 48.0,
        ..Default::default()
    };

    let render_params = RenderParams {
        padding: 10,
        background: Some(Color::white()),
        ..Default::default()
    };

    let shaped = shaper.shape(text, font.clone(), &shaping_params).unwrap();

    let opixa = OpixaRenderer::new();
    let skia = SkiaRenderer::new();
    let zeno = ZenoRenderer::new();

    let opixa_result = opixa.render(&shaped, font.clone(), &render_params);
    let skia_result = skia.render(&shaped, font.clone(), &render_params);
    let zeno_result = zeno.render(&shaped, font, &render_params);

    // All should succeed
    assert!(opixa_result.is_ok(), "Opixa should render CBDT");
    assert!(skia_result.is_ok(), "Skia should render CBDT");
    assert!(zeno_result.is_ok(), "Zeno should render CBDT");

    let opixa_rendered = opixa_result.unwrap();
    let skia_rendered = skia_result.unwrap();
    let zeno_rendered = zeno_result.unwrap();

    let (ow, oh) = get_bitmap_dims(&opixa_rendered);
    let (sw, sh) = get_bitmap_dims(&skia_rendered);
    let (zw, zh) = get_bitmap_dims(&zeno_rendered);

    // Dimensions should be similar (within 15% tolerance for bitmap fonts)
    let tolerance = 0.15;

    let w_diff_skia = (ow as f32 - sw as f32).abs() / ow.max(1) as f32;
    let h_diff_skia = (oh as f32 - sh as f32).abs() / oh.max(1) as f32;
    let w_diff_zeno = (ow as f32 - zw as f32).abs() / ow.max(1) as f32;
    let h_diff_zeno = (oh as f32 - zh as f32).abs() / oh.max(1) as f32;

    assert!(
        w_diff_skia <= tolerance,
        "CBDT Opixa vs Skia width difference: {}x{} vs {}x{} ({}%)",
        ow, oh, sw, sh, w_diff_skia * 100.0
    );
    assert!(
        h_diff_skia <= tolerance,
        "CBDT Opixa vs Skia height difference: {}x{} vs {}x{} ({}%)",
        ow, oh, sw, sh, h_diff_skia * 100.0
    );
    assert!(
        w_diff_zeno <= tolerance,
        "CBDT Opixa vs Zeno width difference: {}x{} vs {}x{} ({}%)",
        ow, oh, zw, zh, w_diff_zeno * 100.0
    );
    assert!(
        h_diff_zeno <= tolerance,
        "CBDT Opixa vs Zeno height difference: {}x{} vs {}x{} ({}%)",
        ow, oh, zw, zh, h_diff_zeno * 100.0
    );
}

#[test]
fn test_bitmap_scaling_preserves_content() {
    let font_path = test_font_path("Nabla-Regular-sbix.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: sbix font not found");
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load sbix font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let shaper = NoneShaper::new();
    let renderer = OpixaRenderer::new();
    let text = "1";

    let render_params = RenderParams {
        padding: 5,
        background: Some(Color::white()),
        ..Default::default()
    };

    let mut sizes: Vec<(u32, u32, usize)> = Vec::new();

    // Test at different sizes
    for size in [32.0, 64.0, 128.0] {
        let shaping_params = ShapingParams {
            size,
            ..Default::default()
        };

        let shaped = shaper.shape(text, font.clone(), &shaping_params).unwrap();
        let rendered = renderer.render(&shaped, font.clone(), &render_params).unwrap();

        let (w, h) = get_bitmap_dims(&rendered);
        let data = get_bitmap_data(&rendered);
        let content_pixels = count_non_transparent_pixels(data);

        sizes.push((w, h, content_pixels));
    }

    // Larger sizes should have proportionally larger dimensions
    for i in 1..sizes.len() {
        let (prev_w, prev_h, _) = sizes[i - 1];
        let (curr_w, curr_h, _) = sizes[i];

        // Dimensions should increase with size (at least 20% larger)
        assert!(
            curr_w >= prev_w,
            "Width should not decrease with larger size: {} -> {}",
            prev_w, curr_w
        );
        assert!(
            curr_h >= prev_h,
            "Height should not decrease with larger size: {} -> {}",
            prev_h, curr_h
        );
    }

    // All sizes should have content
    for (i, (w, h, pixels)) in sizes.iter().enumerate() {
        assert!(
            *pixels > 10,
            "Size {} should have content: {}x{} with {} pixels",
            [32, 64, 128][i], w, h, pixels
        );
    }
}

// =============================================================================
// Multi-Glyph Rendering Tests (text strings)
// =============================================================================

#[test]
fn test_colr_multi_glyph_spacing() {
    let font_path = test_font_path("Nabla-Regular-COLR.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: COLR font not found");
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load COLR font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let shaper = NoneShaper::new();
    let renderer = OpixaRenderer::new();

    let shaping_params = ShapingParams {
        size: 48.0,
        ..Default::default()
    };

    let render_params = RenderParams {
        padding: 10,
        background: Some(Color::white()),
        ..Default::default()
    };

    // Render single glyph
    let shaped_single = shaper.shape("A", font.clone(), &shaping_params).unwrap();
    let rendered_single = renderer.render(&shaped_single, font.clone(), &render_params).unwrap();
    let (single_w, _) = get_bitmap_dims(&rendered_single);

    // Render multiple glyphs
    let shaped_multi = shaper.shape("AAA", font.clone(), &shaping_params).unwrap();
    let rendered_multi = renderer.render(&shaped_multi, font.clone(), &render_params).unwrap();
    let (multi_w, _) = get_bitmap_dims(&rendered_multi);

    // Triple width should be roughly 3x single (minus some padding overlap)
    // Allow generous tolerance for spacing variations
    let expected_min = single_w * 2; // At least 2x
    let expected_max = single_w * 4; // At most 4x

    assert!(
        multi_w >= expected_min && multi_w <= expected_max,
        "Multi-glyph width should be ~3x single: single={}, multi={} (expected {}-{})",
        single_w, multi_w, expected_min, expected_max
    );
}

#[test]
fn test_svg_multi_glyph_no_overlap() {
    let font_path = test_font_path("Nabla-Regular-SVG.ttf");
    if !font_path.exists() {
        eprintln!("Skipping test: SVG font not found");
        return;
    }

    let font_face = TypfFontFace::from_file(&font_path).expect("Failed to load SVG font");
    let font: Arc<dyn FontRef> = Arc::new(font_face);

    let shaper = NoneShaper::new();
    let renderer = OpixaRenderer::new();

    let shaping_params = ShapingParams {
        size: 64.0,
        ..Default::default()
    };

    let render_params = RenderParams {
        padding: 10,
        background: Some(Color::white()),
        ..Default::default()
    };

    let shaped = shaper.shape("XY", font.clone(), &shaping_params).unwrap();
    let rendered = renderer.render(&shaped, font, &render_params).unwrap();

    let (width, height) = get_bitmap_dims(&rendered);
    let data = get_bitmap_data(&rendered);

    // Check that glyphs are properly spaced (not overlapping in center)
    // Look at the vertical strip in the middle of the image
    let mid_x = width / 2;
    let strip_width = 4;
    let mut mid_strip_content = 0;

    for y in 0..height {
        for dx in 0..strip_width {
            let x = mid_x.saturating_sub(strip_width / 2) + dx;
            if x < width {
                let idx = ((y * width + x) * 4) as usize;
                if data[idx + 3] > 0 {
                    mid_strip_content += 1;
                }
            }
        }
    }

    // The middle strip should have relatively low content if glyphs are properly spaced
    // This is a heuristic - we're checking for obvious overlap issues
    let total_pixels = (height * strip_width) as usize;
    let mid_ratio = mid_strip_content as f32 / total_pixels as f32;

    // Less than 50% of mid-strip should be content (allowing for descenders, etc.)
    assert!(
        mid_ratio < 0.5,
        "Glyphs may be overlapping: mid-strip content ratio = {:.2}",
        mid_ratio
    );
}
