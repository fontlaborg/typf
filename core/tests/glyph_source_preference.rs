use typf_core::{GlyphSource, GlyphSourcePreference};

#[test]
fn default_order_prefers_outlines_then_color_then_bitmaps() {
    let pref = GlyphSourcePreference::default();

    assert_eq!(
        pref.prefer,
        vec![
            GlyphSource::Glyf,
            GlyphSource::Cff2,
            GlyphSource::Cff,
            GlyphSource::Colr1,
            GlyphSource::Colr0,
            GlyphSource::Svg,
            GlyphSource::Sbix,
            GlyphSource::Cbdt,
            GlyphSource::Ebdt,
        ]
    );
    assert!(pref.deny.is_empty());
}

#[test]
fn from_parts_deduplicates_and_respects_denies() {
    let pref = GlyphSourcePreference::from_parts(
        vec![
            GlyphSource::Svg,
            GlyphSource::Glyf,
            GlyphSource::Svg,
            GlyphSource::Colr0,
        ],
        [GlyphSource::Svg, GlyphSource::Cbdt],
    );

    assert_eq!(
        pref.prefer,
        vec![GlyphSource::Glyf, GlyphSource::Colr0],
        "deny list should strip excluded sources while preserving order"
    );
    assert!(pref.deny.contains(&GlyphSource::Svg));
    assert!(pref.deny.contains(&GlyphSource::Cbdt));
}

#[test]
fn empty_prefer_falls_back_to_default_minus_denies() {
    let pref =
        GlyphSourcePreference::from_parts(Vec::new(), [GlyphSource::Colr1, GlyphSource::Svg]);

    assert_eq!(
        pref.prefer,
        vec![
            GlyphSource::Glyf,
            GlyphSource::Cff2,
            GlyphSource::Cff,
            GlyphSource::Colr0,
            GlyphSource::Sbix,
            GlyphSource::Cbdt,
            GlyphSource::Ebdt,
        ],
        "default order should be reused with denied sources removed"
    );
    assert!(pref.deny.contains(&GlyphSource::Colr1));
    assert!(pref.deny.contains(&GlyphSource::Svg));
}
