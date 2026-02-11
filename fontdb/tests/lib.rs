// this_file: crates/typf-fontdb/tests/lib.rs

use std::path::PathBuf;
use std::sync::Arc;

use read_fonts::{FontRef as ReadFontRef, TableProvider};
use typf_core::traits::FontRef;
use typf_fontdb::TypfFontFace;

fn repo_test_font_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-fonts")
        .join(name)
}

#[test]
fn test_metrics_when_loading_real_font_then_matches_read_fonts_tables() {
    let font_path = repo_test_font_path("NotoSans-Regular.ttf");
    let font = Arc::new(TypfFontFace::from_file(&font_path).expect("load test font"));
    let font_ref: Arc<dyn FontRef> = font;

    let metrics = font_ref
        .metrics()
        .expect("TypfFontFace should expose metrics");

    let data = std::fs::read(&font_path).expect("read test font bytes");
    let read_font = ReadFontRef::from_index(&data, 0).expect("parse test font");
    let expected_units_per_em = read_font
        .head()
        .map(|head| head.units_per_em())
        .unwrap_or(1000);

    let (expected_ascent, expected_descent, expected_line_gap) = read_font
        .os2()
        .ok()
        .map(|os2| {
            (
                os2.s_typo_ascender(),
                os2.s_typo_descender(),
                os2.s_typo_line_gap(),
            )
        })
        .or_else(|| {
            read_font.hhea().ok().map(|hhea| {
                (
                    hhea.ascender().to_i16(),
                    hhea.descender().to_i16(),
                    hhea.line_gap().to_i16(),
                )
            })
        })
        .unwrap_or((0, 0, 0));

    assert_eq!(
        metrics.units_per_em, expected_units_per_em,
        "units_per_em should match head.units_per_em"
    );
    assert_eq!(
        metrics.ascent, expected_ascent,
        "ascent should match OS/2 or hhea"
    );
    assert_eq!(
        metrics.descent, expected_descent,
        "descent should match OS/2 or hhea"
    );
    assert_eq!(
        metrics.line_gap, expected_line_gap,
        "line_gap should match OS/2 or hhea"
    );
}

#[test]
fn test_variation_axes_when_loading_variable_font_then_returns_axes() {
    let font_path = repo_test_font_path("Kalnia[wdth,wght].ttf");
    let font = Arc::new(TypfFontFace::from_file(&font_path).expect("load variable font"));
    let font_ref: Arc<dyn FontRef> = font;

    let axes = font_ref
        .variation_axes()
        .expect("Variable font should have axes");

    // Kalnia has wdth and wght axes
    assert_eq!(axes.len(), 2, "Kalnia should have 2 axes");

    // Check we got the right axis tags
    let tags: Vec<&str> = axes.iter().map(|a| a.tag.as_str()).collect();
    assert!(tags.contains(&"wdth"), "Should contain wdth axis");
    assert!(tags.contains(&"wght"), "Should contain wght axis");

    // Check wght axis has reasonable values
    let wght = axes.iter().find(|a| a.tag == "wght").unwrap();
    assert!(wght.min_value >= 100.0, "wght min should be >= 100");
    assert!(wght.max_value <= 900.0, "wght max should be <= 900");
    assert!(wght.default_value >= wght.min_value, "default >= min");
    assert!(wght.default_value <= wght.max_value, "default <= max");
}

#[test]
fn test_variation_axes_when_loading_static_font_then_returns_none() {
    let font_path = repo_test_font_path("NotoSans-Regular.ttf");
    let font = Arc::new(TypfFontFace::from_file(&font_path).expect("load static font"));
    let font_ref: Arc<dyn FontRef> = font;

    let axes = font_ref.variation_axes();
    assert!(axes.is_none(), "Static font should not have variation axes");
}

#[test]
fn test_is_variable_when_loading_variable_font_then_returns_true() {
    let font_path = repo_test_font_path("Kalnia[wdth,wght].ttf");
    let font = Arc::new(TypfFontFace::from_file(&font_path).expect("load variable font"));
    let font_ref: Arc<dyn FontRef> = font;

    assert!(font_ref.is_variable(), "Kalnia should be variable");
}

#[test]
fn test_is_variable_when_loading_static_font_then_returns_false() {
    let font_path = repo_test_font_path("NotoSans-Regular.ttf");
    let font = Arc::new(TypfFontFace::from_file(&font_path).expect("load static font"));
    let font_ref: Arc<dyn FontRef> = font;

    assert!(!font_ref.is_variable(), "NotoSans should not be variable");
}
