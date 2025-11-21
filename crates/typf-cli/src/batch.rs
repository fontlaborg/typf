//! Turn lists of text into images, fast and organized
//!
//! Process hundreds or thousands of text lines from files or stdin.
//! Perfect for testing fonts, generating samples, or batch processing.

#![allow(dead_code)] // Legacy batch processing infrastructure - retained for future v2.1 REPL mode

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use typf::error::{Result, TypfError};
use typf_core::{
    traits::{Exporter, FontRef, Renderer, Shaper},
    types::Direction,
    Color, RenderParams, ShapingParams,
};

/// How the batch processor should behave
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Where we're reading from (stdin if None)
    pub input: Option<PathBuf>,
    /// Where the output files go
    pub output_dir: PathBuf,
    /// How to name the output files ({} gets replaced with line number)
    pub output_pattern: String,
    /// How big the text should be
    pub size: f32,
    /// What format we're outputting
    pub format: String,
    /// Show progress or work silently
    pub verbose: bool,
}

impl BatchConfig {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            input: None,
            output_dir,
            output_pattern: "output_{}.ppm".to_string(),
            size: 16.0,
            format: "ppm".to_string(),
            verbose: true,
        }
    }

    /// Parse batch config from command-line arguments
    pub fn parse(args: &[String]) -> Result<Self> {
        let mut config = BatchConfig::new(PathBuf::from("."));
        let mut i = 0;

        while i < args.len() {
            match args[i].as_str() {
                "--batch-input" | "-b" => {
                    if i + 1 < args.len() {
                        config.input = Some(PathBuf::from(&args[i + 1]));
                        i += 2;
                    } else {
                        return Err(TypfError::Other("--batch-input requires a file path".into()));
                    }
                },
                "--batch-output" | "-B" => {
                    if i + 1 < args.len() {
                        config.output_dir = PathBuf::from(&args[i + 1]);
                        i += 2;
                    } else {
                        return Err(TypfError::Other(
                            "--batch-output requires a directory path".into(),
                        ));
                    }
                },
                "--batch-pattern" => {
                    if i + 1 < args.len() {
                        config.output_pattern = args[i + 1].clone();
                        if !config.output_pattern.contains("{}") {
                            return Err(TypfError::Other(
                                "Batch pattern must contain {} placeholder".into(),
                            ));
                        }
                        i += 2;
                    } else {
                        return Err(TypfError::Other("--batch-pattern requires a pattern".into()));
                    }
                },
                "--size" | "-s" => {
                    if i + 1 < args.len() {
                        config.size = args[i + 1]
                            .parse()
                            .map_err(|_| TypfError::Other("Invalid size value".into()))?;
                        i += 2;
                    } else {
                        return Err(TypfError::Other("--size requires a value".into()));
                    }
                },
                "--format" | "-f" => {
                    if i + 1 < args.len() {
                        config.format = args[i + 1].clone();
                        if !["ppm", "pgm", "pbm"].contains(&config.format.as_str()) {
                            return Err(TypfError::Other("Format must be ppm, pgm, or pbm".into()));
                        }
                        i += 2;
                    } else {
                        return Err(TypfError::Other("--format requires a value".into()));
                    }
                },
                "--quiet" | "-q" => {
                    config.verbose = false;
                    i += 1;
                },
                _ => {
                    i += 1;
                },
            }
        }

        // Create output directory if it doesn't exist
        if !config.output_dir.exists() {
            std::fs::create_dir_all(&config.output_dir).map_err(|e| {
                TypfError::Other(format!("Failed to create output directory: {}", e))
            })?;
        }

        Ok(config)
    }
}

/// Turn a file of text lines into a folder of images
pub fn process_batch<S, R, E, F>(
    config: &BatchConfig,
    shaper: Arc<S>,
    renderer: Arc<R>,
    exporter: Arc<E>,
    font: Arc<F>,
) -> Result<usize>
where
    S: Shaper + 'static,
    R: Renderer + 'static,
    E: Exporter + 'static,
    F: FontRef + 'static,
{
    // Open our input source (file or stdin)
    let reader: Box<dyn BufRead> = match &config.input {
        Some(path) => {
            let file = File::open(path).map_err(|e| {
                TypfError::Other(format!("Failed to open input file {}: {}", path.display(), e))
            })?;
            Box::new(BufReader::new(file))
        },
        None => Box::new(BufReader::new(std::io::stdin())),
    };

    let shaping_params = ShapingParams {
        size: config.size,
        direction: Direction::LeftToRight,
        language: Some("en".to_string()),
        script: None,
        features: Vec::new(),
        variations: Vec::new(),
        letter_spacing: 0.0,
    };

    let render_params = RenderParams {
        foreground: Color::black(),
        background: Some(Color::white()),
        padding: 10,
        antialias: true,
        variations: Vec::new(),
    };

    let components = PipelineComponents {
        shaper,
        renderer,
        exporter,
        font,
    };

    let mut count = 0;
    let mut errors = 0;

    // Process each line as a separate rendering job
    for (line_num, line_result) in reader.lines().enumerate() {
        let line = match line_result {
            Ok(l) => l.trim().to_string(),
            Err(e) => {
                if config.verbose {
                    eprintln!("Error reading line {}: {}", line_num + 1, e);
                }
                errors += 1;
                continue;
            },
        };

        // Skip empty lines - nothing to render
        if line.is_empty() {
            continue;
        }

        // Figure out where this output file should go
        let output_filename = config
            .output_pattern
            .replace("{}", &(count + 1).to_string());
        let output_path = config.output_dir.join(output_filename);

        if config.verbose {
            println!("[{}/...] Processing: \"{}\"", count + 1, line);
        }

        // Shape, render, and export this line
        match process_single_line(&line, &output_path, &shaping_params, &render_params, &components)
        {
            Ok(_) => {
                count += 1;
                if config.verbose {
                    println!("  -> Saved to {}", output_path.display());
                }
            },
            Err(e) => {
                errors += 1;
                if config.verbose {
                    eprintln!("  -> Error: {}", e);
                }
            },
        }
    }

    if config.verbose {
        println!("\nBatch processing complete:");
        println!("  Processed: {}", count);
        if errors > 0 {
            println!("  Errors: {}", errors);
        }
    }

    Ok(count)
}

/// All the pipeline pieces, bundled together for easy passing
struct PipelineComponents<S, R, E, F> {
    shaper: Arc<S>,
    renderer: Arc<R>,
    exporter: Arc<E>,
    font: Arc<F>,
}

/// One line of text, one rendered image
fn process_single_line<S, R, E, F>(
    text: &str,
    output_path: &Path,
    shaping_params: &ShapingParams,
    render_params: &RenderParams,
    components: &PipelineComponents<S, R, E, F>,
) -> Result<()>
where
    S: Shaper + 'static,
    R: Renderer + 'static,
    E: Exporter + 'static,
    F: FontRef + 'static,
{
    // Shape the text into positioned glyphs
    let shaped = components
        .shaper
        .shape(text, components.font.clone(), shaping_params)?;

    // Render those glyphs to pixels
    let rendered = components
        .renderer
        .render(&shaped, components.font.clone(), render_params)?;

    // Export the pixels to the chosen format
    let exported = components.exporter.export(&rendered)?;

    // Write the result to disk
    let mut file = File::create(output_path)
        .map_err(|e| TypfError::Other(format!("Failed to create output file: {}", e)))?;
    file.write_all(&exported)
        .map_err(|e| TypfError::Other(format!("Failed to write output file: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config_parse() {
        let args = vec![
            "--batch-output".to_string(),
            "/tmp/output".to_string(),
            "--size".to_string(),
            "24".to_string(),
            "--format".to_string(),
            "pgm".to_string(),
        ];

        let config = BatchConfig::parse(&args).unwrap();
        assert_eq!(config.output_dir, PathBuf::from("/tmp/output"));
        assert_eq!(config.size, 24.0);
        assert_eq!(config.format, "pgm");
    }

    #[test]
    fn test_batch_pattern_validation() {
        let args = vec!["--batch-pattern".to_string(), "no_placeholder".to_string()];

        let result = BatchConfig::parse(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("placeholder"));
    }

    #[test]
    fn test_batch_config_defaults() {
        let config = BatchConfig::new(PathBuf::from("."));
        assert_eq!(config.size, 16.0);
        assert_eq!(config.format, "ppm");
        assert!(config.verbose);
        assert!(config.input.is_none());
    }
}
