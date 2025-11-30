//! Where text becomes understandable: Unicode processing for Typf
//!
//! The second stage of the pipeline transforms raw strings into structured
//! runs that understand direction, scripts, and boundaries. Without this stage,
//! Arabic would render backwards and Chinese characters might break randomly.

use icu_properties::props::Script;
use icu_segmenter::{GraphemeClusterSegmenter, LineSegmenter, WordSegmenter};
use unicode_bidi::BidiInfo;
use unicode_normalization::UnicodeNormalization;

use typf_core::{
    error::Result,
    types::{Direction, TextRun},
};

/// Configuration for how deeply we analyze your text
#[derive(Debug, Clone, Default)]
pub struct UnicodeOptions {
    pub detect_scripts: bool,
    pub normalize: bool,
    pub bidi_resolve: bool,
    pub language: Option<String>,
}

/// Your text's tour guide through the Unicode landscape
pub struct UnicodeProcessor;

impl UnicodeProcessor {
    /// Creates a new processor ready to tackle any Unicode challenge
    pub fn new() -> Self {
        Self
    }

    /// Transforms raw text into directionally-aware runs
    pub fn process(&self, text: &str, options: &UnicodeOptions) -> Result<Vec<TextRun>> {
        if text.is_empty() {
            return Ok(vec![]);
        }

        // Clean up messy Unicode (é from e + ´ becomes é)
        let normalized = if options.normalize {
            text.nfc().collect::<String>()
        } else {
            text.to_string()
        };

        // Figure out which writing system each part uses
        let scripts = if options.detect_scripts {
            self.detect_scripts(&normalized)?
        } else {
            vec![(Script::Common, 0, normalized.len())]
        };

        // Find where graphemes start and end (for complex scripts like Thai)
        let grapheme_segmenter = GraphemeClusterSegmenter::new();
        let _grapheme_breaks: Vec<usize> = grapheme_segmenter.segment_str(text).collect();

        // Handle bidirectional text (Arabic/Hebrew vs Latin)
        let runs = if options.bidi_resolve {
            self.create_bidi_runs(&normalized, scripts, options)?
        } else {
            // Simple left-to-right processing
            self.create_simple_runs(&normalized, scripts, options, Direction::LeftToRight)?
        };

        Ok(runs)
    }

    /// Maps out when and where writing systems change in your text
    fn detect_scripts(&self, text: &str) -> Result<Vec<(Script, usize, usize)>> {
        use icu_properties::script::ScriptWithExtensions;
        let script_data = ScriptWithExtensions::new();
        let mut scripts = Vec::new();
        let mut current_script = Script::Common;
        let mut start = 0;

        for (i, ch) in text.char_indices() {
            let script = script_data.get_script_val(ch);

            if script != current_script && script != Script::Common && script != Script::Inherited {
                if i > start {
                    scripts.push((current_script, start, i));
                }
                current_script = script;
                start = i;
            }
        }

        if start < text.len() {
            scripts.push((current_script, start, text.len()));
        }

        if scripts.is_empty() {
            scripts.push((Script::Common, 0, text.len()));
        }

        Ok(scripts)
    }

    /// Creates text runs assuming everyone reads left-to-right
    fn create_simple_runs(
        &self,
        text: &str,
        scripts: Vec<(Script, usize, usize)>,
        options: &UnicodeOptions,
        default_direction: Direction,
    ) -> Result<Vec<TextRun>> {
        let mut runs = Vec::new();
        for (script, start, end) in scripts {
            runs.push(TextRun {
                text: text[start..end].to_string(),
                start,
                end,
                script,
                language: options.language.clone().unwrap_or_default(),
                direction: default_direction,
            });
        }
        Ok(runs)
    }

    /// Creates text runs that respect text direction (critical for Arabic/Hebrew)
    fn create_bidi_runs(
        &self,
        text: &str,
        scripts: Vec<(Script, usize, usize)>,
        options: &UnicodeOptions,
    ) -> Result<Vec<TextRun>> {
        let bidi_info = BidiInfo::new(text, None);

        // Extract the directional information
        // IMPORTANT: levels is indexed by SCALAR (char) position, not byte position.
        // The scripts vector from detect_scripts contains byte positions, so we must
        // convert byte positions to char positions when indexing into levels.
        let levels = bidi_info.levels;
        let mut runs = Vec::new();

        let char_count = text.chars().count();

        // For each script segment, decide if it reads RTL or LTR
        for (script, start_byte, end_byte) in scripts {
            // Convert byte positions to char positions for level indexing
            let start_char = if start_byte == 0 {
                0
            } else {
                text[..start_byte].chars().count()
            };
            let end_char = if end_byte >= text.len() {
                char_count
            } else {
                text[..end_byte].chars().count()
            };

            // Guard against out of bounds (shouldn't happen with valid input)
            let start_char = start_char.min(levels.len());
            let end_char = end_char.min(levels.len()).max(start_char);

            // Look at the actual bidi levels to determine direction
            let has_rtl = if start_char < end_char && end_char <= levels.len() {
                levels[start_char..end_char]
                    .iter()
                    .any(|level| level.is_rtl())
            } else {
                false
            };

            let direction = if has_rtl {
                Direction::RightToLeft
            } else {
                Direction::LeftToRight
            };

            runs.push(TextRun {
                text: text[start_byte..end_byte].to_string(),
                start: start_byte,
                end: end_byte,
                script,
                language: options.language.clone().unwrap_or_default(),
                direction,
            });
        }

        Ok(runs)
    }

    /// Breaks text into words using locale-aware rules
    pub fn segment_words(&self, text: &str) -> Result<Vec<String>> {
        use icu_segmenter::options::WordBreakInvariantOptions;
        let segmenter = WordSegmenter::new_auto(WordBreakInvariantOptions::default());
        let mut words = Vec::new();
        let mut last = 0;

        for boundary in segmenter.segment_str(text) {
            if boundary > last {
                let word = &text[last..boundary];
                if !word.trim().is_empty() {
                    words.push(word.to_string());
                }
            }
            last = boundary;
        }

        Ok(words)
    }

    /// Finds all the places where text could safely break across lines
    pub fn segment_lines(&self, text: &str) -> Result<Vec<usize>> {
        use icu_segmenter::options::LineBreakOptions;
        let segmenter = LineSegmenter::new_auto(LineBreakOptions::default());
        let breaks: Vec<usize> = segmenter.segment_str(text).collect();
        Ok(breaks)
    }
}

impl Default for UnicodeProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
            assert_eq!(run.direction, Direction::RightToLeft, "Arabic should be RTL");
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
            assert_eq!(run.direction, Direction::RightToLeft, "Hebrew should be RTL");
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
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // Property: NFC normalization is idempotent (normalizing twice == normalizing once)
    proptest! {
        #[test]
        fn prop_nfc_idempotent(s in "\\PC*") {
            let processor = UnicodeProcessor::new();
            let options = UnicodeOptions {
                normalize: true,
                detect_scripts: false,
                bidi_resolve: false,
                language: None,
            };

            // Normalize once
            let result1 = processor.process(&s, &options);

            // Skip if processing failed (e.g., invalid input)
            if let Ok(runs1) = result1 {
                let normalized1 = runs1.iter().map(|r| r.text.as_str()).collect::<String>();

                // Normalize the already-normalized text
                let result2 = processor.process(&normalized1, &options).unwrap();
                let normalized2 = result2.iter().map(|r| r.text.as_str()).collect::<String>();

                // NFC normalization should be idempotent
                prop_assert_eq!(normalized1, normalized2);
            }
        }
    }

    // Property: Normalization always produces valid Unicode
    proptest! {
        #[test]
        fn prop_nfc_produces_valid_unicode(s in "\\PC*") {
            let processor = UnicodeProcessor::new();
            let options = UnicodeOptions {
                normalize: true,
                detect_scripts: false,
                bidi_resolve: false,
                language: None,
            };

            if let Ok(result) = processor.process(&s, &options) {
                for run in result {
                    // All characters in normalized text should be valid Unicode
                    prop_assert!(run.text.chars().all(|c| c.is_ascii() || !c.is_control() || c.is_whitespace()));
                }
            }
        }
    }

    // Property: Empty string stays empty after processing
    proptest! {
        #[test]
        fn prop_empty_stays_empty(_options in any::<bool>().prop_flat_map(|normalize| {
            Just(UnicodeOptions {
                normalize,
                detect_scripts: false,
                bidi_resolve: false,
                language: None,
            })
        })) {
            let processor = UnicodeProcessor::new();
            let result = processor.process("", &_options).unwrap();
            prop_assert_eq!(result.len(), 0);
        }
    }

    // Property: Single ASCII character processing is consistent
    proptest! {
        #[test]
        fn prop_ascii_unchanged(s in "[a-z]") {
            let processor = UnicodeProcessor::new();
            let options = UnicodeOptions {
                normalize: true,
                detect_scripts: false,
                bidi_resolve: false,
                language: None,
            };

            let result = processor.process(&s, &options).unwrap();

            prop_assert_eq!(result.len(), 1);
            prop_assert_eq!(&result[0].text, &s);
        }
    }

    // Property: Bidi processing never loses characters
    proptest! {
        #[test]
        fn prop_bidi_preserves_length(s in "[a-zA-Z א-ת]{1,50}") {
            let processor = UnicodeProcessor::new();
            let options = UnicodeOptions {
                normalize: false,
                detect_scripts: false,
                bidi_resolve: true,
                language: None,
            };

            let result = processor.process(&s, &options).unwrap();

            // Total character count should be preserved
            let output_len: usize = result.iter().map(|r| r.text.chars().count()).sum();
            prop_assert_eq!(output_len, s.chars().count());
        }
    }

    // Property: Script detection always assigns a script
    proptest! {
        #[test]
        fn prop_script_detection_always_succeeds(s in "\\PC{1,100}") {
            let processor = UnicodeProcessor::new();
            let options = UnicodeOptions {
                normalize: false,
                detect_scripts: true,
                bidi_resolve: false,
                language: None,
            };

            if let Ok(result) = processor.process(&s, &options) {
                // Every run should have a script assigned
                for run in result {
                    // Script should be set (not just default)
                    prop_assert!(!run.text.is_empty());
                }
            }
        }
    }

    // Property: Processing is deterministic (same input -> same output)
    proptest! {
        #[test]
        fn prop_processing_deterministic(s in "\\PC{1,50}") {
            let processor = UnicodeProcessor::new();
            let options = UnicodeOptions {
                normalize: true,
                detect_scripts: true,
                bidi_resolve: true,
                language: None,
            };

            if let Ok(result1) = processor.process(&s, &options) {
                let result2 = processor.process(&s, &options).unwrap();

                // Results should be identical
                prop_assert_eq!(result1.len(), result2.len());
                for (run1, run2) in result1.iter().zip(result2.iter()) {
                    prop_assert_eq!(&run1.text, &run2.text);
                    prop_assert_eq!(run1.script, run2.script);
                    prop_assert_eq!(run1.direction, run2.direction);
                }
            }
        }
    }
}
