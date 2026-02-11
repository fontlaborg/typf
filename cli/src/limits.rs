//! Shared input/resource limit helpers for CLI commands.
// this_file: crates/typf-cli/src/limits.rs

use std::io::Read;
use std::path::Path;

pub const MAX_FONT_FILE_BYTES: u64 = 100 * 1024 * 1024;
pub const MAX_JSONL_BATCH_INPUT_BYTES: u64 = 32 * 1024 * 1024;
pub const MAX_TEXT_CONTENT_BYTES: usize = 1_000_000;

pub fn read_to_string_with_limit<R: Read>(
    reader: R,
    max_bytes: u64,
    label: &str,
) -> Result<String, String> {
    let mut limited_reader = reader.take(max_bytes + 1);
    let mut input = String::new();
    limited_reader
        .read_to_string(&mut input)
        .map_err(|error| format!("Failed to read {}: {}", label, error))?;

    if input.len() as u64 > max_bytes {
        return Err(format!(
            "{} exceeds max size of {} bytes (got at least {})",
            label,
            max_bytes,
            input.len()
        ));
    }

    Ok(input)
}

pub fn validate_file_size_limit(path: &Path, max_bytes: u64, label: &str) -> Result<(), String> {
    let metadata = std::fs::metadata(path).map_err(|error| {
        format!(
            "Failed to inspect {} '{}': {}",
            label,
            path.display(),
            error
        )
    })?;
    let size = metadata.len();

    if size > max_bytes {
        return Err(format!(
            "{} '{}' exceeds max size of {} bytes (got {})",
            label,
            path.display(),
            max_bytes,
            size
        ));
    }

    Ok(())
}

pub fn validate_text_size_limit(text: &str, max_bytes: usize, label: &str) -> Result<(), String> {
    let size = text.len();
    if size > max_bytes {
        return Err(format!(
            "{} exceeds max size of {} bytes (got {})",
            label, max_bytes, size
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file(prefix: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        path.push(format!("typf_limits_{}_{}", prefix, nanos));
        path
    }

    #[test]
    fn test_read_to_string_with_limit_when_under_limit_then_succeeds() {
        let input = Cursor::new(b"{\"jobs\":[]}".to_vec());
        let output = read_to_string_with_limit(input, 32, "json input")
            .expect("input below limit should be accepted");
        assert_eq!(output, "{\"jobs\":[]}");
    }

    #[test]
    fn test_read_to_string_with_limit_when_over_limit_then_errors() {
        let input = Cursor::new(vec![b'a'; 10]);
        let error = read_to_string_with_limit(input, 4, "json input")
            .expect_err("input above limit should fail");
        assert!(
            error.contains("exceeds max size"),
            "expected size-limit validation message, got: {}",
            error
        );
    }

    #[test]
    fn test_validate_file_size_limit_when_under_limit_then_succeeds() {
        let path = temp_file("under");
        std::fs::write(&path, b"abcd").expect("temp file should be writable");

        let result = validate_file_size_limit(&path, 4, "font file");
        std::fs::remove_file(&path).expect("temp file cleanup should succeed");

        assert!(result.is_ok(), "file at limit should be accepted");
    }

    #[test]
    fn test_validate_file_size_limit_when_over_limit_then_errors() {
        let path = temp_file("over");
        let file = std::fs::File::create(&path).expect("temp file should be creatable");
        file.set_len(5).expect("temp file length should be set");

        let error = validate_file_size_limit(&path, 4, "font file")
            .expect_err("oversized file should be rejected");
        std::fs::remove_file(&path).expect("temp file cleanup should succeed");

        assert!(
            error.contains("exceeds max size"),
            "expected size-limit validation message, got: {}",
            error
        );
    }

    #[test]
    fn test_validate_text_size_limit_when_at_limit_then_succeeds() {
        let text = "a".repeat(8);
        let result = validate_text_size_limit(&text, 8, "input text");
        assert!(result.is_ok(), "text exactly at limit should be accepted");
    }

    #[test]
    fn test_validate_text_size_limit_when_over_limit_then_errors() {
        let text = "a".repeat(9);
        let error = validate_text_size_limit(&text, 8, "input text")
            .expect_err("text over limit should be rejected");
        assert!(
            error.contains("exceeds max size"),
            "expected size-limit validation message, got: {}",
            error
        );
    }
}
