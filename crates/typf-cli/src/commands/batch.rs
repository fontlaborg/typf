//! Batch command implementation
//!
//! Processes multiple rendering jobs from a JSONL file.
// this_file: crates/typf-cli/src/commands/batch.rs

use crate::cli::{BatchArgs, OutputFormat};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Component, Path, PathBuf};
use typf::error::{Result, TypfError};

/// JSONL job specification
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BatchJob {
    /// Text to render
    text: String,
    /// Font file path
    #[serde(skip_serializing_if = "Option::is_none")]
    font: Option<String>,
    /// Output file name (relative to output directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<String>,
    /// Font size
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<f32>,
    /// Shaper backend
    #[serde(skip_serializing_if = "Option::is_none")]
    shaper: Option<String>,
    /// Renderer backend
    #[serde(skip_serializing_if = "Option::is_none")]
    renderer: Option<String>,
    /// Output format
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    /// Foreground color (RRGGBBAA)
    #[serde(skip_serializing_if = "Option::is_none")]
    foreground: Option<String>,
    /// Background color (RRGGBBAA)
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<String>,
    /// Language tag
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
}

pub fn run(args: &BatchArgs) -> Result<()> {
    if !args.quiet {
        eprintln!("Typf Batch Processor v{}", env!("CARGO_PKG_VERSION"));
        eprintln!();
    }

    // Open input source
    let reader: Box<dyn BufRead> = if let Some(ref input_path) = args.input {
        Box::new(BufReader::new(File::open(input_path)?))
    } else {
        if !args.quiet {
            eprintln!("Reading jobs from stdin...");
        }
        Box::new(BufReader::new(io::stdin()))
    };

    // Create output directory if it doesn't exist
    if !args.output.exists() {
        std::fs::create_dir_all(&args.output)?;
    }
    validate_output_pattern(&args.pattern)?;

    // Process each job
    let mut job_count = 0;
    let mut success_count = 0;
    let mut error_count = 0;

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        job_count += 1;

        // Parse JSON job
        let job: BatchJob = match serde_json::from_str(&line) {
            Ok(job) => job,
            Err(e) => {
                eprintln!("Error parsing job {}: {}", line_num + 1, e);
                error_count += 1;
                continue;
            },
        };

        // Determine output file
        let output_file = match resolve_output_file(
            &args.output,
            &args.pattern,
            job_count,
            job.output.as_deref(),
        ) {
            Ok(path) => path,
            Err(e) => {
                eprintln!(
                    "Error processing job {} (line {}): {}",
                    job_count,
                    line_num + 1,
                    e
                );
                error_count += 1;
                continue;
            },
        };

        if args.verbose {
            eprintln!(
                "Job {}: Rendering \"{}\" to {}",
                job_count,
                job.text,
                output_file.display()
            );
        }

        // Build render command arguments
        // For now, we'll use a simple implementation
        // In a full version, this would invoke the render module
        match process_job(&job, &output_file, args) {
            Ok(()) => {
                success_count += 1;
                if !args.quiet && !args.verbose {
                    eprint!(".");
                }
            },
            Err(e) => {
                eprintln!("\nError processing job {}: {}", job_count, e);
                error_count += 1;
            },
        }
    }

    if !args.quiet {
        eprintln!();
        eprintln!();
        eprintln!("Batch processing complete:");
        eprintln!("  Total jobs: {}", job_count);
        eprintln!("  Successful: {}", success_count);
        eprintln!("  Failed: {}", error_count);
    }

    if error_count > 0 {
        Err(TypfError::Other(format!("{} jobs failed", error_count)))
    } else {
        Ok(())
    }
}

fn validate_output_pattern(pattern: &str) -> Result<()> {
    if pattern.contains("{}") {
        Ok(())
    } else {
        Err(TypfError::ConfigError(
            "Output filename pattern must contain '{}' placeholder".to_string(),
        ))
    }
}

fn validate_relative_output_path(relative_path: &Path) -> Result<()> {
    if relative_path.as_os_str().is_empty() {
        return Err(TypfError::ConfigError(
            "Output filename cannot be empty".to_string(),
        ));
    }
    if relative_path.is_absolute() {
        return Err(TypfError::ConfigError(
            "Batch job output path must be relative to --output directory".to_string(),
        ));
    }
    #[cfg(windows)]
    if relative_path
        .components()
        .any(|component| matches!(component, Component::Prefix(_)))
    {
        return Err(TypfError::ConfigError(
            "Batch job output path must not include a drive prefix".to_string(),
        ));
    }
    if relative_path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(TypfError::ConfigError(
            "Batch job output path must not contain '..' segments".to_string(),
        ));
    }
    if relative_path.file_name().is_none() {
        return Err(TypfError::ConfigError(
            "Batch job output path must include a file name".to_string(),
        ));
    }

    Ok(())
}

fn resolve_output_file(
    output_dir: &Path,
    output_pattern: &str,
    job_index: usize,
    job_output: Option<&str>,
) -> Result<PathBuf> {
    let relative_path = match job_output {
        Some(output) => PathBuf::from(output),
        None => PathBuf::from(output_pattern.replace("{}", &job_index.to_string())),
    };

    validate_relative_output_path(&relative_path)?;

    let output_file = output_dir.join(relative_path);
    if let Some(parent) = output_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(output_file)
}

fn process_job(job: &BatchJob, output_file: &Path, args: &BatchArgs) -> Result<()> {
    // Build RenderArgs from BatchJob
    // For now, this is a simplified version
    // In practice, you would construct proper RenderArgs and call render::run()

    use crate::cli::RenderArgs;
    use crate::commands::render;

    let format = parse_output_format(job.format.as_deref())?;

    let render_args = RenderArgs {
        text: Some(job.text.clone()),
        font_file: job.font.as_ref().map(PathBuf::from),
        face_index: 0,
        instance: None,
        text_arg: None,
        text_file: None,
        shaper: job.shaper.clone().unwrap_or_else(|| "auto".to_string()),
        renderer: job.renderer.clone().unwrap_or_else(|| "auto".to_string()),
        direction: "auto".to_string(),
        language: job.language.clone(),
        script: "auto".to_string(),
        features: None,
        font_size: job
            .size
            .map(|s| s.to_string())
            .unwrap_or_else(|| "200".to_string()),
        line_height: 120,
        width_height: "none".to_string(),
        margin: 10,
        font_optical_sizing: "auto".to_string(),
        foreground: job
            .foreground
            .clone()
            .unwrap_or_else(|| "000000FF".to_string()),
        background: job
            .background
            .clone()
            .unwrap_or_else(|| "FFFFFF00".to_string()),
        color_palette: 0,
        glyph_source: Vec::new(),
        no_shaping_cache: false,
        no_glyph_cache: false,
        output_file: Some(output_file.to_path_buf()),
        format,
        quiet: args.quiet,
        verbose: args.verbose,
    };

    render::run(&render_args)
}

fn parse_output_format(raw: Option<&str>) -> Result<OutputFormat> {
    let Some(raw) = raw else {
        return Ok(OutputFormat::Png);
    };
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "png" => Ok(OutputFormat::Png),
        "svg" => Ok(OutputFormat::Svg),
        "pbm" => Ok(OutputFormat::Pbm),
        "pgm" => Ok(OutputFormat::Pgm),
        "ppm" => Ok(OutputFormat::Ppm),
        _ => Err(TypfError::ConfigError(format!(
            "Unsupported batch output format '{}'; expected one of: png, svg, pbm, pgm, ppm",
            raw
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        path.push(format!("typf_cli_batch_{}_{}", prefix, nanos));
        std::fs::create_dir_all(&path).expect("temp dir should be created");
        path
    }

    #[test]
    fn test_validate_output_pattern_when_missing_placeholder_then_error() {
        let error = validate_output_pattern("out.png")
            .expect_err("pattern without placeholder should be rejected");
        assert!(
            error.to_string().contains("placeholder"),
            "expected placeholder validation message, got: {}",
            error
        );
    }

    #[test]
    fn test_validate_output_pattern_when_present_then_ok() {
        validate_output_pattern("out_{}.png").expect("pattern with placeholder should pass");
    }

    #[test]
    fn test_resolve_output_file_when_nested_relative_then_created_under_output_dir() {
        let output_dir = temp_dir("nested");
        let resolved = resolve_output_file(
            &output_dir,
            "unused_{}.png",
            1,
            Some("nested/job/output.png"),
        )
        .expect("nested relative output should be accepted");

        assert_eq!(
            resolved,
            output_dir.join("nested/job/output.png"),
            "output should stay inside output dir"
        );
        assert!(
            output_dir.join("nested/job").exists(),
            "parent directories should be created"
        );

        std::fs::remove_dir_all(output_dir).expect("temp dir cleanup should succeed");
    }

    #[test]
    fn test_resolve_output_file_when_parent_segment_then_error() {
        let output_dir = temp_dir("parent_dir");
        let error = resolve_output_file(&output_dir, "unused_{}.png", 1, Some("../escape.png"))
            .expect_err("parent-dir traversal should fail");
        assert!(
            error.to_string().contains(".."),
            "expected parent-dir validation message, got: {}",
            error
        );

        std::fs::remove_dir_all(output_dir).expect("temp dir cleanup should succeed");
    }

    #[test]
    fn test_resolve_output_file_when_absolute_path_then_error() {
        let output_dir = temp_dir("absolute");
        #[cfg(unix)]
        let absolute = "/tmp/escape.png";
        #[cfg(windows)]
        let absolute = "C:\\temp\\escape.png";

        let error = resolve_output_file(&output_dir, "unused_{}.png", 1, Some(absolute))
            .expect_err("absolute output path should fail");
        assert!(
            error.to_string().contains("relative"),
            "expected relative-path validation message, got: {}",
            error
        );

        std::fs::remove_dir_all(output_dir).expect("temp dir cleanup should succeed");
    }

    #[test]
    fn test_parse_output_format_when_none_then_defaults_to_png() {
        let format = parse_output_format(None).expect("default format should parse");
        assert!(matches!(format, crate::cli::OutputFormat::Png));
    }

    #[test]
    fn test_parse_output_format_when_uppercase_then_parsed_case_insensitively() {
        let format = parse_output_format(Some("SVG")).expect("uppercase SVG should parse");
        assert!(matches!(format, crate::cli::OutputFormat::Svg));
    }

    #[test]
    fn test_parse_output_format_when_unknown_then_error() {
        let error = parse_output_format(Some("gif"))
            .expect_err("unsupported output format should be rejected");
        assert!(
            error
                .to_string()
                .contains("Unsupported batch output format"),
            "expected unsupported-format error, got: {}",
            error
        );
    }

    #[test]
    fn test_batch_job_deserialization_when_unknown_field_then_error() {
        let input = r#"{
            "text":"Hello",
            "font":"font.ttf",
            "unknown_field":"unexpected value"
        }"#;
        let error = serde_json::from_str::<BatchJob>(input)
            .expect_err("unknown JSON field should fail due to deny_unknown_fields");
        assert!(
            error.to_string().contains("unknown field"),
            "expected serde unknown-field error, got: {}",
            error
        );
    }
}
