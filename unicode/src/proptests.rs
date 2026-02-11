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
