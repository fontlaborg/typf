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
mod tests;

#[cfg(test)]
mod proptests;
