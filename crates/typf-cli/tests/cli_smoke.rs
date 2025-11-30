//! CLI Smoke Tests
//!
//! Integration tests for the typf CLI commands:
//! - `info`: Display backend information
//! - `render`: Render text to image
//! - `batch`: Process multiple rendering jobs
//!
//! Tests cover both success cases and failure cases (bad input, missing fonts).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Get the path to the typf binary
fn typf_binary() -> PathBuf {
    // During cargo test, the binary is in target/debug
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // crates
    path.pop(); // root
    path.push("target");
    path.push("debug");
    path.push("typf");
    path
}

/// Get the path to a test font
fn test_font(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // crates
    path.pop(); // root
    path.push("test-fonts");
    path.push(name);
    path
}

/// Create a temporary file path
fn temp_output(ext: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("typf_test_{}.{}", id, ext));
    path
}

// ============================================================================
// Info Command Tests
// ============================================================================

#[test]
fn test_info_help() {
    let output = Command::new(typf_binary())
        .args(["info", "--help"])
        .output()
        .expect("Failed to execute typf info --help");

    assert!(output.status.success(), "info --help should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Display information"),
        "Help should describe the command"
    );
}

#[test]
fn test_info_shapers() {
    let output = Command::new(typf_binary())
        .args(["info", "--shapers"])
        .output()
        .expect("Failed to execute typf info --shapers");

    assert!(output.status.success(), "info --shapers should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should list at least one shaper
    assert!(
        stdout.contains("hb") || stdout.contains("harfbuzz") || stdout.contains("Shaper"),
        "Should list available shapers"
    );
}

#[test]
fn test_info_renderers() {
    let output = Command::new(typf_binary())
        .args(["info", "--renderers"])
        .output()
        .expect("Failed to execute typf info --renderers");

    assert!(output.status.success(), "info --renderers should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should list at least one renderer
    assert!(
        stdout.contains("opixa")
            || stdout.contains("skia")
            || stdout.contains("zeno")
            || stdout.contains("Renderer"),
        "Should list available renderers"
    );
}

#[test]
fn test_info_formats() {
    let output = Command::new(typf_binary())
        .args(["info", "--formats"])
        .output()
        .expect("Failed to execute typf info --formats");

    assert!(output.status.success(), "info --formats should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should list at least one format
    assert!(
        stdout.contains("png") || stdout.contains("svg") || stdout.contains("Format"),
        "Should list available formats"
    );
}

// ============================================================================
// Render Command Tests - Success Cases
// ============================================================================

#[test]
fn test_render_png_to_file() {
    let font = test_font("NotoSans-Regular.ttf");
    if !font.exists() {
        eprintln!("Skipping test: font not found at {:?}", font);
        return;
    }

    let output_file = temp_output("png");

    let output = Command::new(typf_binary())
        .args([
            "render",
            "Hello",
            "-f",
            font.to_str().unwrap(),
            "-o",
            output_file.to_str().unwrap(),
            "-O",
            "png",
            "-q",
        ])
        .output()
        .expect("Failed to execute typf render");

    assert!(
        output.status.success(),
        "render should succeed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_file.exists(), "Output file should be created");

    // Verify it's a valid PNG (check magic bytes)
    let data = fs::read(&output_file).expect("Failed to read output");
    assert!(data.len() > 8, "PNG should have content");
    assert_eq!(
        &data[0..8],
        &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
        "Should be valid PNG"
    );

    // Cleanup
    let _ = fs::remove_file(output_file);
}

#[test]
fn test_render_svg_to_file() {
    let font = test_font("NotoSans-Regular.ttf");
    if !font.exists() {
        eprintln!("Skipping test: font not found at {:?}", font);
        return;
    }

    let output_file = temp_output("svg");

    let output = Command::new(typf_binary())
        .args([
            "render",
            "Test",
            "-f",
            font.to_str().unwrap(),
            "-o",
            output_file.to_str().unwrap(),
            "-O",
            "svg",
            "-q",
        ])
        .output()
        .expect("Failed to execute typf render");

    assert!(
        output.status.success(),
        "render SVG should succeed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_file.exists(), "Output file should be created");

    // Verify it's valid SVG
    let data = fs::read_to_string(&output_file).expect("Failed to read output");
    assert!(data.contains("<svg"), "Should contain SVG element");
    assert!(data.contains("</svg>"), "Should close SVG element");

    // Cleanup
    let _ = fs::remove_file(output_file);
}

#[test]
fn test_render_with_font_size() {
    let font = test_font("NotoSans-Regular.ttf");
    if !font.exists() {
        return;
    }

    let output_file = temp_output("png");

    let output = Command::new(typf_binary())
        .args([
            "render",
            "Size",
            "-f",
            font.to_str().unwrap(),
            "-s",
            "48",
            "-o",
            output_file.to_str().unwrap(),
            "-q",
        ])
        .output()
        .expect("Failed to execute typf render");

    assert!(output.status.success(), "render with custom size should succeed");
    assert!(output_file.exists());

    let _ = fs::remove_file(output_file);
}

#[test]
fn test_render_with_colors() {
    let font = test_font("NotoSans-Regular.ttf");
    if !font.exists() {
        return;
    }

    let output_file = temp_output("png");

    let output = Command::new(typf_binary())
        .args([
            "render",
            "Color",
            "-f",
            font.to_str().unwrap(),
            "-c",
            "FF0000FF", // Red foreground
            "-b",
            "0000FFFF", // Blue background
            "-o",
            output_file.to_str().unwrap(),
            "-q",
        ])
        .output()
        .expect("Failed to execute typf render");

    assert!(output.status.success(), "render with colors should succeed");
    assert!(output_file.exists());

    let _ = fs::remove_file(output_file);
}

#[test]
fn test_render_arabic_rtl() {
    let font = test_font("NotoNaskhArabic-Regular.ttf");
    if !font.exists() {
        return;
    }

    let output_file = temp_output("png");

    let output = Command::new(typf_binary())
        .args([
            "render",
            "مرحبا",
            "-f",
            font.to_str().unwrap(),
            "-d",
            "rtl",
            "-o",
            output_file.to_str().unwrap(),
            "-q",
        ])
        .output()
        .expect("Failed to execute typf render");

    assert!(
        output.status.success(),
        "render RTL should succeed: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_file.exists());

    let _ = fs::remove_file(output_file);
}

// ============================================================================
// Render Command Tests - Failure Cases
// ============================================================================

#[test]
fn test_render_missing_font_fails() {
    let output = Command::new(typf_binary())
        .args([
            "render",
            "Hello",
            "-f",
            "/nonexistent/path/to/font.ttf",
            "-q",
        ])
        .output()
        .expect("Failed to execute typf render");

    assert!(
        !output.status.success(),
        "render with missing font should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found")
            || stderr.contains("No such file")
            || stderr.contains("error")
            || stderr.contains("Error"),
        "Should report font not found error: {}",
        stderr
    );
}

#[test]
fn test_render_no_font_fails() {
    let output = Command::new(typf_binary())
        .args(["render", "Hello", "-q"])
        .output()
        .expect("Failed to execute typf render");

    // Should fail because no font is specified
    assert!(
        !output.status.success(),
        "render without font should fail"
    );
}

#[test]
fn test_render_invalid_format_fails() {
    let font = test_font("NotoSans-Regular.ttf");
    if !font.exists() {
        return;
    }

    let output = Command::new(typf_binary())
        .args([
            "render",
            "Hello",
            "-f",
            font.to_str().unwrap(),
            "-O",
            "invalid_format",
        ])
        .output()
        .expect("Failed to execute typf render");

    assert!(
        !output.status.success(),
        "render with invalid format should fail"
    );
}

#[test]
fn test_render_corrupted_font_fails() {
    // Create a temporary file with invalid font data
    let temp_font = temp_output("ttf");
    fs::write(&temp_font, b"not a real font file").expect("Failed to create temp file");

    let output = Command::new(typf_binary())
        .args([
            "render",
            "Hello",
            "-f",
            temp_font.to_str().unwrap(),
            "-q",
        ])
        .output()
        .expect("Failed to execute typf render");

    assert!(
        !output.status.success(),
        "render with corrupted font should fail"
    );

    let _ = fs::remove_file(temp_font);
}

// ============================================================================
// Batch Command Tests
// ============================================================================

#[test]
fn test_batch_help() {
    let output = Command::new(typf_binary())
        .args(["batch", "--help"])
        .output()
        .expect("Failed to execute typf batch --help");

    assert!(output.status.success(), "batch --help should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("batch") || stdout.contains("JSONL") || stdout.contains("jobs"),
        "Help should describe batch command"
    );
}

#[test]
fn test_batch_empty_input() {
    // Create empty JSONL file
    let input_file = temp_output("jsonl");
    fs::write(&input_file, "").expect("Failed to create temp file");

    let output = Command::new(typf_binary())
        .args([
            "batch",
            "-i",
            input_file.to_str().unwrap(),
            "-q",
        ])
        .output()
        .expect("Failed to execute typf batch");

    // Empty input should succeed (nothing to process)
    assert!(
        output.status.success(),
        "batch with empty input should succeed"
    );

    let _ = fs::remove_file(input_file);
}

#[test]
fn test_batch_invalid_json_fails() {
    // Create JSONL file with invalid JSON
    let input_file = temp_output("jsonl");
    fs::write(&input_file, "this is not valid json\n").expect("Failed to create temp file");

    let output = Command::new(typf_binary())
        .args([
            "batch",
            "-i",
            input_file.to_str().unwrap(),
            "-q",
        ])
        .output()
        .expect("Failed to execute typf batch");

    // Invalid JSON should fail
    assert!(
        !output.status.success(),
        "batch with invalid JSON should fail"
    );

    let _ = fs::remove_file(input_file);
}

// ============================================================================
// General CLI Tests
// ============================================================================

#[test]
fn test_version() {
    let output = Command::new(typf_binary())
        .args(["--version"])
        .output()
        .expect("Failed to execute typf --version");

    assert!(output.status.success(), "--version should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("typf") || stdout.contains('.'),
        "Should show version info"
    );
}

#[test]
fn test_help() {
    let output = Command::new(typf_binary())
        .args(["--help"])
        .output()
        .expect("Failed to execute typf --help");

    assert!(output.status.success(), "--help should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("render"), "Should list render command");
    assert!(stdout.contains("info"), "Should list info command");
    assert!(stdout.contains("batch"), "Should list batch command");
}

#[test]
fn test_unknown_command_fails() {
    let output = Command::new(typf_binary())
        .args(["unknown_command"])
        .output()
        .expect("Failed to execute typf");

    assert!(
        !output.status.success(),
        "unknown command should fail"
    );
}
