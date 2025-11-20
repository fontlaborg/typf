//! Attack Unicode processing before it attacks your users
//!
//! Unicode is a minefield of edge cases: invalid sequences, malformed UTF-8,
//! bidirectional text that breaks layouts, normalization bugs that cause
//! text comparisons to fail. This fuzzer bombards the Unicode processor
//! with random data to find crashes in:
//!
//! - UTF-8 parsing and validation
//! - NFC normalization (composing/decomposing characters)
//! - Bidirectional algorithm (RTL/LTR text mixing)
//! - Script detection (Arabic, Hebrew, CJK, etc.)
//! - Text segmentation (word/sentence boundaries)
//!
//! A single Unicode bug can crash your entire app. Fuzz now, ship confidently.

#![no_main]

use libfuzzer_sys::fuzz_target;
use typf_unicode::{UnicodeOptions, UnicodeProcessor};

fuzz_target!(|data: &[u8]| {
    // libFuzzer gives us raw bytes - transform into potentially invalid text
    let text = String::from_utf8_lossy(data);

    // Reject inputs that would cause timeouts or excessive memory usage
    if text.is_empty() || text.len() > 10_000 {
        return;
    }

    let processor = UnicodeProcessor::new();

    // Test with full Unicode processing enabled - this exercises the most code
    let options = UnicodeOptions {
        normalize: true,        // NFC normalization - complex Unicode operations
        detect_scripts: true,   // Script detection - needs character database lookups
        bidi_resolve: true,     // Bidirectional algorithm - notoriously complex
        language: None,         // Auto-detect language from text
    };

    // The Unicode processor should never crash, even with malformed input
    let _ = processor.process(&text, &options);

    // Test with normalization disabled - exercises different code paths
    let options_no_norm = UnicodeOptions {
        normalize: false,       // Skip Unicode normalization
        detect_scripts: true,   // Still do script detection
        bidi_resolve: true,     // Still handle bidirectional text
        language: None,
    };

    let _ = processor.process(&text, &options_no_norm);
});
