use super::*;

#[test]
fn test_empty_text() {
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions::default();
    let result = processor.process("", &options).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_simple_latin() {
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        detect_scripts: true,
        ..Default::default()
    };
    let result = processor.process("Hello World", &options).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].direction, Direction::LeftToRight);
}

#[test]
fn test_word_segmentation() {
    let processor = UnicodeProcessor::new();
    let words = processor.segment_words("Hello, World! Test 123").unwrap();
    assert!(words.contains(&"Hello".to_string()));
    assert!(words.contains(&"World".to_string()));
    assert!(words.contains(&"Test".to_string()));
    assert!(words.contains(&"123".to_string()));
}

#[test]
fn test_line_breaks() {
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions::default();
    let result = processor
        .process("Line 1\nLine 2\nLine 3", &options)
        .unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_arabic_rtl() {
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        detect_scripts: true,
        bidi_resolve: true,
        ..Default::default()
    };
    // "Hello" in Arabic
    let result = processor.process("مرحبا", &options).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].direction, Direction::RightToLeft);
    assert_eq!(result[0].script, Script::Arabic);
}

#[test]
fn test_devanagari() {
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        detect_scripts: true,
        ..Default::default()
    };
    // "Namaste" in Devanagari
    let result = processor.process("नमस्ते", &options).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].script, Script::Devanagari);
    assert_eq!(result[0].direction, Direction::LeftToRight);
}

#[test]
fn test_mixed_scripts() {
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        detect_scripts: true,
        ..Default::default()
    };
    // English + Arabic
    let result = processor.process("Hello مرحبا", &options).unwrap();
    // Should detect script changes
    assert!(result.len() >= 1);
}

#[test]
fn test_hebrew_rtl() {
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        detect_scripts: true,
        bidi_resolve: true,
        ..Default::default()
    };
    // "Shalom" in Hebrew
    let result = processor.process("שלום", &options).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].direction, Direction::RightToLeft);
    assert_eq!(result[0].script, Script::Hebrew);
}

#[test]
fn test_chinese_cjk() {
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        detect_scripts: true,
        ..Default::default()
    };
    // "Hello" in Chinese
    let result = processor.process("你好", &options).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].script, Script::Han);
    assert_eq!(result[0].direction, Direction::LeftToRight);
}

#[test]
fn test_thai() {
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        detect_scripts: true,
        ..Default::default()
    };
    // "Hello" in Thai
    let result = processor.process("สวัสดี", &options).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].script, Script::Thai);
}

#[test]
fn test_nfc_normalization() {
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        normalize: true,
        ..Default::default()
    };

    // Test with decomposed character (é as e + combining acute)
    let decomposed = "e\u{0301}"; // e + combining acute accent
    let result = processor.process(decomposed, &options).unwrap();

    // After NFC normalization, should be composed
    assert_eq!(result[0].text, "é"); // Single precomposed character
}

#[test]
fn test_normalization_with_scripts() {
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        normalize: true,
        detect_scripts: true,
        ..Default::default()
    };

    // Test normalization with combining marks
    let text = "café"; // Last e might be decomposed
    let result = processor.process(text, &options).unwrap();

    // Should normalize and detect Latin script
    assert!(result.len() > 0);
    // Normalized text should be valid
    assert!(result[0].text.len() > 0);
}

#[test]
fn test_no_normalization() {
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        normalize: false,
        ..Default::default()
    };

    // Test without normalization
    let text = "hello";
    let result = processor.process(text, &options).unwrap();

    // Text should remain unchanged
    assert_eq!(result[0].text, "hello");
}

#[test]
fn test_bidi_mixed_text() {
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        detect_scripts: true,
        bidi_resolve: true,
        ..Default::default()
    };

    // Mixed LTR (English) and RTL (Arabic)
    let text = "Hello مرحبا World";
    let result = processor.process(text, &options).unwrap();

    // Should create multiple runs with appropriate directions
    assert!(result.len() >= 1);

    // Find RTL run (Arabic)
    let has_rtl = result.iter().any(|r| r.direction == Direction::RightToLeft);
    assert!(has_rtl, "Should detect RTL direction for Arabic text");
}

#[test]
fn test_bidi_pure_rtl() {
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        detect_scripts: true,
        bidi_resolve: true,
        ..Default::default()
    };

    // Pure RTL text
    let text = "مرحبا بالعالم";
    let result = processor.process(text, &options).unwrap();

    // Should be all RTL
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].direction, Direction::RightToLeft);
}

#[test]
fn test_bidi_arabic_latin_emoji() {
    // Mixed Arabic + Latin + Emoji - tests correct byte-to-char indexing
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        detect_scripts: true,
        bidi_resolve: true,
        ..Default::default()
    };

    // Arabic (RTL) + English (LTR) + Emoji (neutral)
    let text = "مرحبا Hello 😀";
    let result = processor.process(text, &options).unwrap();

    // Should have runs with correct directions
    assert!(!result.is_empty());

    // Arabic part should be RTL
    let arabic_runs: Vec<_> = result
        .iter()
        .filter(|r| r.script == Script::Arabic)
        .collect();
    assert!(!arabic_runs.is_empty(), "Should detect Arabic script");
    for run in arabic_runs {
        assert_eq!(
            run.direction,
            Direction::RightToLeft,
            "Arabic should be RTL"
        );
    }
}

#[test]
fn test_bidi_hebrew_with_numbers() {
    // Hebrew with embedded numbers - common real-world case
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        detect_scripts: true,
        bidi_resolve: true,
        ..Default::default()
    };

    // Hebrew with number (numbers are weak RTL in Hebrew context)
    let text = "שלום 123 עולם";
    let result = processor.process(text, &options).unwrap();

    // Should have Hebrew runs marked as RTL
    let hebrew_runs: Vec<_> = result
        .iter()
        .filter(|r| r.script == Script::Hebrew)
        .collect();
    assert!(!hebrew_runs.is_empty(), "Should detect Hebrew script");
    for run in hebrew_runs {
        assert_eq!(
            run.direction,
            Direction::RightToLeft,
            "Hebrew should be RTL"
        );
    }
}

#[test]
fn test_bidi_thai_marks() {
    // Thai with combining marks - tests multi-byte char handling
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        detect_scripts: true,
        bidi_resolve: true,
        ..Default::default()
    };

    // Thai text with vowel marks and tone marks
    let text = "สวัสดี"; // "Hello" in Thai (contains combining marks)
    let result = processor.process(text, &options).unwrap();

    assert!(!result.is_empty());
    // Thai is LTR
    assert_eq!(result[0].direction, Direction::LeftToRight);
    assert_eq!(result[0].script, Script::Thai);
}

#[test]
fn test_bidi_multibyte_boundary() {
    // Test that byte/char boundary handling is correct
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        detect_scripts: true,
        bidi_resolve: true,
        ..Default::default()
    };

    // Mix of single-byte (ASCII) and multi-byte (Arabic) characters
    // This tests the byte-to-char conversion at boundaries
    let text = "A مرحبا B";
    let result = processor.process(text, &options).unwrap();

    // Should not panic and should detect correct directions
    assert!(!result.is_empty());

    // Find Arabic run and verify it's RTL
    let has_rtl = result.iter().any(|r| r.direction == Direction::RightToLeft);
    assert!(has_rtl, "Should detect RTL for Arabic content");
}

#[test]
fn test_line_breaking_simple() {
    let processor = UnicodeProcessor::new();
    let text = "Hello world! This is a test.";
    let breaks = processor.segment_lines(text).unwrap();

    // Should have line break opportunities (at least start and end)
    assert!(breaks.len() >= 2);
    assert_eq!(breaks[0], 0); // Start
    assert_eq!(*breaks.last().unwrap(), text.len()); // End
}

#[test]
fn test_line_breaking_multiline() {
    let processor = UnicodeProcessor::new();
    let text = "Line 1\nLine 2\nLine 3";
    let breaks = processor.segment_lines(text).unwrap();

    // Should have breaks at newlines
    assert!(breaks.len() > 3);
    assert!(breaks.contains(&0));
    assert!(breaks.contains(&text.len()));
}

#[test]
fn test_line_breaking_long_text() {
    let processor = UnicodeProcessor::new();
    let text =
        "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs.";
    let breaks = processor.segment_lines(text).unwrap();

    // Should have multiple break opportunities (spaces, punctuation)
    assert!(breaks.len() > 10);
}
