//! Script-tag normalization helpers for CLI inputs.
//!
//! this_file: crates/typf-cli/src/script.rs

/// Normalize an optional ISO 15924 script tag.
///
/// - `None`, blank, and `"auto"` inputs become `Ok(None)`.
/// - Valid tags are canonicalized to titlecase (`arab` -> `Arab`).
/// - Invalid tags return a descriptive error.
pub fn normalize_script_tag(raw: Option<&str>) -> Result<Option<String>, String> {
    let Some(value) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    if value.eq_ignore_ascii_case("auto") {
        return Ok(None);
    }

    if value.len() != 4 {
        return Err(format!(
            "'{}' must be exactly 4 ASCII letters (ISO 15924)",
            value
        ));
    }

    if !value.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return Err(format!(
            "'{}' must contain only ASCII letters (ISO 15924)",
            value
        ));
    }

    let mut canonical = value.to_ascii_lowercase();
    if let Some(first) = canonical.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    Ok(Some(canonical))
}

#[cfg(test)]
mod tests {
    use super::normalize_script_tag;

    #[test]
    fn normalize_script_tag_when_missing_blank_or_auto_then_none() {
        assert_eq!(
            normalize_script_tag(None).expect("missing script tag should parse"),
            None,
            "missing script tag should remain unset"
        );
        assert_eq!(
            normalize_script_tag(Some(" \t\n")).expect("blank script tag should parse"),
            None,
            "blank script tag should normalize to unset"
        );
        assert_eq!(
            normalize_script_tag(Some("AUTO")).expect("auto script tag should parse"),
            None,
            "auto script tag should normalize to unset"
        );
    }

    #[test]
    fn normalize_script_tag_when_valid_then_titlecases() {
        assert_eq!(
            normalize_script_tag(Some(" aRAb ")).expect("valid script tag should parse"),
            Some("Arab".to_string()),
            "script tags should canonicalize to titlecase"
        );
    }

    #[test]
    fn normalize_script_tag_when_invalid_length_then_error() {
        let error = normalize_script_tag(Some("Latin"))
            .expect_err("script tags must be exactly four letters");
        assert!(
            error.contains("exactly 4 ASCII letters"),
            "expected length validation guidance, got: {}",
            error
        );
    }

    #[test]
    fn normalize_script_tag_when_non_alpha_then_error() {
        let error = normalize_script_tag(Some("Ar4b"))
            .expect_err("script tags with non-letter chars should fail");
        assert!(
            error.contains("only ASCII letters"),
            "expected alphabetic validation guidance, got: {}",
            error
        );
    }
}
