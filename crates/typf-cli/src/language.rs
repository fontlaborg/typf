//! Language-tag normalization helpers for CLI inputs.
//!
//! this_file: crates/typf-cli/src/language.rs

use language_tags::LanguageTag;

/// Normalize an optional BCP 47 language tag.
///
/// - `None` and blank inputs become `Ok(None)`.
/// - Valid tags are canonicalized when possible.
/// - Invalid tags return a descriptive error.
pub fn normalize_language_tag(raw: Option<&str>) -> Result<Option<String>, String> {
    let Some(value) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    let parsed = LanguageTag::parse(value)
        .map_err(|error| format!("'{}' is not a valid BCP 47 language tag: {}", value, error))?;

    let canonical = parsed.canonicalize().unwrap_or(parsed).to_string();

    Ok(Some(canonical))
}

#[cfg(test)]
mod tests {
    use super::normalize_language_tag;

    #[test]
    fn normalize_language_tag_when_missing_or_blank_then_none() {
        assert_eq!(
            normalize_language_tag(None).expect("missing language should parse"),
            None,
            "missing language should remain unset"
        );
        assert_eq!(
            normalize_language_tag(Some(" \t\n")).expect("blank language should parse"),
            None,
            "blank language should normalize to unset"
        );
    }

    #[test]
    fn normalize_language_tag_when_valid_then_canonicalizes() {
        assert_eq!(
            normalize_language_tag(Some(" en-us ")).expect("language should parse"),
            Some("en-US".to_string()),
            "language tags should canonicalize case per BCP 47 conventions"
        );
        assert_eq!(
            normalize_language_tag(Some("zh-hans-cn")).expect("language should parse"),
            Some("zh-Hans-CN".to_string()),
            "script and region subtags should canonicalize case"
        );
    }

    #[test]
    fn normalize_language_tag_when_invalid_then_error() {
        let error = normalize_language_tag(Some("en_US"))
            .expect_err("underscore-delimited language tags should fail");
        assert!(
            error.contains("valid BCP 47"),
            "expected BCP 47 validation guidance, got: {}",
            error
        );
    }
}
