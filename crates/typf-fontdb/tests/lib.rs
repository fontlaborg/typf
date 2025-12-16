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
