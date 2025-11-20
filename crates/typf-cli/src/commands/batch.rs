///! Batch command implementation
///!
///! Processes multiple rendering jobs from a JSONL file.

use crate::cli::BatchArgs;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use typf::error::{Result, TypfError};

/// JSONL job specification
#[derive(Debug, Serialize, Deserialize)]
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
        eprintln!("TYPF Batch Processor v{}", env!("CARGO_PKG_VERSION"));
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
            }
        };

        // Determine output file
        let output_file = if let Some(ref output) = job.output {
            args.output.join(output)
        } else {
            let filename = args.pattern.replace("{}", &job_count.to_string());
            args.output.join(filename)
        };

        if args.verbose {
            eprintln!("Job {}: Rendering \"{}\" to {}", job_count, job.text, output_file.display());
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
            }
            Err(e) => {
                eprintln!("\nError processing job {}: {}", job_count, e);
                error_count += 1;
            }
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
        Err(TypfError::Other(format!(
            "{} jobs failed",
            error_count
        )))
    } else {
        Ok(())
    }
}

fn process_job(job: &BatchJob, output_file: &PathBuf, args: &BatchArgs) -> Result<()> {
    // Build RenderArgs from BatchJob
    // For now, this is a simplified version
    // In practice, you would construct proper RenderArgs and call render::run()

    use crate::commands::render;
    use crate::cli::{OutputFormat, RenderArgs};

    let format = match job.format.as_deref() {
        Some("png") => OutputFormat::Png,
        Some("svg") => OutputFormat::Svg,
        Some("pbm") => OutputFormat::Pbm,
        Some("pgm") => OutputFormat::Pgm,
        Some("ppm") => OutputFormat::Ppm,
        _ => OutputFormat::Png,
    };

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
        font_size: job.size.map(|s| s.to_string()).unwrap_or_else(|| "200".to_string()),
        line_height: 120,
        width_height: "none".to_string(),
        margin: 10,
        font_optical_sizing: "auto".to_string(),
        foreground: job.foreground.clone().unwrap_or_else(|| "000000FF".to_string()),
        background: job.background.clone().unwrap_or_else(|| "FFFFFF00".to_string()),
        color_palette: 0,
        output_file: Some(output_file.clone()),
        format,
        quiet: args.quiet,
        verbose: args.verbose,
    };

    render::run(&render_args)
}
