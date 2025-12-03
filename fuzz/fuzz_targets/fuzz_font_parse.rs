//! Fuzz font parsing to catch crashes from malformed font files
//!
//! This fuzzer tests the font loading and parsing code paths for robustness.
//! By feeding arbitrary bytes as font data, we ensure that malformed fonts
//! cannot cause panics, memory corruption, or infinite loops.
//!
//! What gets fuzzed:
//! - Font format detection (TTF, OTF, TTC, OTC, WOFF, WOFF2)
//! - Table parsing (cmap, glyf, CFF, head, hhea, etc.)
//! - Font metrics extraction
//! - Glyph ID lookup
//! - Invalid font data handling

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Skip very small inputs that can't be valid fonts
    if data.len() < 12 {
        return;
    }

    // Skip very large inputs to avoid timeouts
    if data.len() > 10_000_000 {
        return;
    }

    // Test read-fonts parsing (low-level font access)
    test_read_fonts(data);

    // Test skrifa parsing (higher-level API)
    test_skrifa(data);

    // Test typf-fontdb loading
    test_fontdb(data);
});

/// Test read-fonts low-level parsing
fn test_read_fonts(data: &[u8]) {
    use read_fonts::FontRef;

    // Try to parse as a font
    if let Ok(font) = FontRef::new(data) {
        // Exercise table access - these should not panic
        let _ = font.head();
        let _ = font.hhea();
        let _ = font.maxp();
        let _ = font.os2();
        let _ = font.post();
        let _ = font.name();
        let _ = font.cmap();

        // Try glyph-related tables
        let _ = font.glyf();
        let _ = font.loca();
        let _ = font.cff();
        let _ = font.cff2();
        let _ = font.colr();
        let _ = font.cpal();
        let _ = font.svg();
        let _ = font.sbix();
        let _ = font.cbdt();
        let _ = font.cblc();
        let _ = font.ebdt();
        let _ = font.eblc();

        // Try OpenType feature tables
        let _ = font.gdef();
        let _ = font.gpos();
        let _ = font.gsub();

        // Try variable font tables
        let _ = font.fvar();
        let _ = font.gvar();
        let _ = font.avar();
        let _ = font.stat();
        let _ = font.hvar();
        let _ = font.vvar();
        let _ = font.mvar();

        // Try to enumerate table tags
        if let Ok(table_directory) = font.table_directory() {
            for record in table_directory.table_records() {
                let _ = record.tag();
                let _ = record.offset();
                let _ = record.length();
            }
        }
    }

    // Try parsing as a font collection (TTC/OTC)
    if let Ok(collection) = read_fonts::FileRef::new(data) {
        match collection {
            read_fonts::FileRef::Font(_) => {}
            read_fonts::FileRef::Collection(ttc) => {
                // Exercise collection API
                let _ = ttc.len();
                for i in 0..ttc.len().min(10) {
                    let _ = ttc.get(i);
                }
            }
        }
    }
}

/// Test skrifa higher-level parsing
fn test_skrifa(data: &[u8]) {
    use skrifa::FontRef;

    if let Ok(font) = FontRef::new(data) {
        // Get basic font info
        let _ = font.head();
        let _ = font.hhea();
        let _ = font.maxp();

        // Try charmap operations
        if let Ok(charmap) = font.charmap() {
            // Test some common codepoints
            let _ = charmap.map('A');
            let _ = charmap.map('a');
            let _ = charmap.map('0');
            let _ = charmap.map(' ');
            let _ = charmap.map('🙂');
            let _ = charmap.map('你');
            let _ = charmap.map('م');
        }

        // Try glyph metrics
        if let Ok(glyph_metrics) = font.glyph_metrics(skrifa::instance::Size::unscaled(), &[]) {
            for gid in 0..font.maxp().map(|m| m.num_glyphs()).unwrap_or(0).min(100) {
                let _ = glyph_metrics.advance_width(skrifa::GlyphId::new(gid));
                let _ = glyph_metrics.left_side_bearing(skrifa::GlyphId::new(gid));
            }
        }

        // Try font metrics
        if let Ok(metrics) = font.metrics(skrifa::instance::Size::unscaled(), &[]) {
            let _ = metrics.units_per_em;
            let _ = metrics.ascent;
            let _ = metrics.descent;
            let _ = metrics.leading;
            let _ = metrics.cap_height;
            let _ = metrics.x_height;
        }

        // Try outline extraction for a few glyphs
        if let Ok(outlines) = font.outline_glyphs() {
            for gid in 0..5u16 {
                let _ = outlines.get(skrifa::GlyphId::new(gid));
            }
        }

        // Try color glyph access
        if let Some(colr) = font.colr() {
            for gid in 0..5u16 {
                let _ = colr.base_glyph(skrifa::GlyphId::new(gid));
            }
        }
    }
}

/// Test typf-fontdb font loading
fn test_fontdb(data: &[u8]) {
    use typf_fontdb::TypfFontFace;

    // Try loading as a font
    if let Ok(face) = TypfFontFace::from_data(data.to_vec()) {
        use typf_core::traits::FontRef;

        // Exercise FontRef trait
        let _ = face.data();
        let _ = face.units_per_em();
        let _ = face.glyph_count();

        // Test glyph lookups
        let _ = face.glyph_id('A');
        let _ = face.glyph_id('a');
        let _ = face.glyph_id(' ');
        let _ = face.glyph_id('你');

        // Test advance widths
        for gid in 0..10u32 {
            let _ = face.advance_width(gid);
        }
    }

    // Try loading with specific face index
    for index in 0..3u32 {
        let _ = TypfFontFace::from_data_with_index(data.to_vec(), index);
    }
}
