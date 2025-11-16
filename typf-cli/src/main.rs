// this_file: src/main.rs

//! TYPF CLI: High-performance font rendering engine.
//!
//! Provides batch rendering compatible with haforu job specifications.

use anyhow::{Context, Result};
use base64::Engine;
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Parser, Subcommand};
use typf_batch::{Job, JobResult, JobSpec};
use typf_core::types::{BoundingBox, Direction, Glyph, ShapingResult, SvgOptions};
use typf_fontdb::FontLoader;
use typf_icu_hb::shaping::{ShapedText, ShapeRequest, TextShaper};
use typf_render::svg::SvgRenderer;
use typf_zeno::GlyphRasterizer;
use rayon::prelude::*;
use std::io::{self, BufRead, Read, Write};
use std::time::Instant;

const DEFAULT_MAX_FONTS: usize = 512;

/// Find the start of pixel data in a PGM P5 file.
///
/// PGM P5 format: "P5\n<width> <height>\n255\n<binary data>"
fn find_pgm_data_start(data: &[u8]) -> Option<usize> {
    let mut pos = 0;
    let mut newlines = 0;

    // Skip past 3 newlines (P5, dimensions, maxval)
    while pos < data.len() && newlines < 3 {
        if data[pos] == b'\n' {
            newlines += 1;
        }
        pos += 1;
    }

    if newlines == 3 && pos < data.len() {
        Some(pos)
    } else {
        None
    }
}

/// Crop a grayscale image to non-inked pixels with 1px margin.
///
/// Returns (cropped_data, new_width, new_height).
fn crop_image(pixels: &[u8], width: u32, height: u32) -> (Vec<u8>, u32, u32) {
    if pixels.is_empty() || width == 0 || height == 0 {
        return (pixels.to_vec(), width, height);
    }

    // Find bounding box of non-white pixels (glyph pixels)
    // Note: In grayscale images, 255 = white (background), 0 = black (foreground/ink)
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0u32;
    let mut max_y = 0u32;

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            // Look for non-white pixels (anything < 255 is considered ink)
            if idx < pixels.len() && pixels[idx] < 255 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    // If no inked pixels found, return original
    if min_x > max_x || min_y > max_y {
        return (pixels.to_vec(), width, height);
    }

    // Add 1px margin (clamped to image bounds)
    min_x = min_x.saturating_sub(1);
    min_y = min_y.saturating_sub(1);
    max_x = (max_x + 1).min(width - 1);
    max_y = (max_y + 1).min(height - 1);

    let crop_width = max_x - min_x + 1;
    let crop_height = max_y - min_y + 1;

    // Extract cropped region
    let mut cropped = Vec::with_capacity((crop_width * crop_height) as usize);
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let idx = (y * width + x) as usize;
            cropped.push(pixels[idx]);
        }
    }

    (cropped, crop_width, crop_height)
}

/// Convert ShapedText to ShapingResult for SVG rendering.
///
/// This is a workaround until we have a unified shaping API.
fn shaped_text_to_shaping_result(
    shaped: &ShapedText,
    text: &str,
    bbox: BoundingBox,
) -> ShapingResult {
    let mut glyphs = Vec::new();
    let mut x_cursor = 0.0;

    for shaped_glyph in &shaped.glyphs {
        let glyph = Glyph {
            id: shaped_glyph.glyph_id,
            cluster: 0, // We don't have cluster info from ShapedText
            x: x_cursor + (shaped_glyph.x_offset as f32 / 64.0),
            y: shaped_glyph.y_offset as f32 / 64.0,
            advance: shaped_glyph.x_advance as f32 / 64.0,
        };
        x_cursor += glyph.advance;
        glyphs.push(glyph);
    }

    ShapingResult {
        text: text.to_string(),
        glyphs,
        advance: x_cursor,
        bbox,
        font: None,
        direction: Direction::LeftToRight,
    }
}

/// TYPF: High-performance multi-backend text rendering
#[derive(Parser)]
#[command(name = "typf")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Process a batch of rendering jobs from stdin (JSON)
    Batch {
        /// Font cache size (number of font instances)
        #[arg(long = "max-fonts", default_value_t = DEFAULT_MAX_FONTS)]
        max_fonts: usize,

        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,

        /// Constrain font paths to this base directory
        #[arg(long)]
        base_dir: Option<Utf8PathBuf>,

        /// Number of parallel worker threads (0 = auto)
        #[arg(long = "jobs", default_value = "0")]
        jobs: usize,
    },

    /// Process jobs from stdin in streaming mode (JSONL input)
    Stream {
        /// Font cache size (number of font instances)
        #[arg(long = "max-fonts", default_value_t = DEFAULT_MAX_FONTS)]
        max_fonts: usize,

        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,

        /// Constrain font paths to this base directory
        #[arg(long)]
        base_dir: Option<Utf8PathBuf>,
    },

    /// Render text to image or metrics
    Render {
        /// Font file path
        #[arg(short = 'f', long = "font-file", alias = "font")]
        font_file: Utf8PathBuf,

        /// Font size in points
        #[arg(short = 's', long = "font-size", alias = "size", default_value = "72")]
        font_size: f32,

        /// Text to render
        #[arg(short = 't', long = "text")]
        text: String,

        /// Font variations (e.g., "wght=700,wdth=100")
        #[arg(long = "variations", alias = "var")]
        variations: Option<String>,

        /// Output file (stdout if not specified)
        #[arg(short = 'o', long = "output-file", alias = "output")]
        output_file: Option<Utf8PathBuf>,

        /// Output format (pgm, png, svg, metrics)
        #[arg(long = "format", default_value = "pgm")]
        format: String,

        /// Canvas width (ignored if --auto-size is used)
        #[arg(long = "width", default_value = "800")]
        width: u32,

        /// Canvas height (ignored if --auto-size is used)
        #[arg(long = "height", default_value = "600")]
        height: u32,

        /// Auto-calculate canvas size from text metrics
        #[arg(long = "auto-size")]
        auto_size: bool,

        /// Padding around auto-sized canvas in pixels
        #[arg(long = "padding", default_value = "10")]
        padding: u32,

        /// Crop to non-inked pixels with 1px margin
        #[arg(long = "crop")]
        crop: bool,

        /// Script hint (e.g., "Latn", "Arab")
        #[arg(long)]
        script: Option<String>,

        /// Language hint (e.g., "en", "ar")
        #[arg(long)]
        language: Option<String>,

        /// Text direction
        #[arg(long = "direction", default_value = "ltr")]
        direction: String,

        /// OpenType features (comma-separated, e.g., "liga=1,kern=0")
        #[arg(long)]
        features: Option<String>,

        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },

    /// Print version information
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Batch {
            max_fonts,
            verbose,
            base_dir,
            jobs,
        } => {
            init_logger(verbose);
            if jobs > 0 {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(jobs)
                    .build_global()
                    .context("Failed to initialize thread pool")?;
            }
            run_batch(max_fonts, base_dir.as_deref())?;
        }

        Commands::Stream {
            max_fonts,
            verbose,
            base_dir,
        } => {
            init_logger(verbose);
            run_stream(max_fonts, base_dir.as_deref())?;
        }

        Commands::Render {
            font_file,
            font_size,
            text,
            variations,
            output_file,
            format,
            width,
            height,
            auto_size,
            padding,
            crop,
            script,
            language,
            direction,
            features,
            verbose,
        } => {
            init_logger(verbose);
            run_render(
                font_file,
                font_size,
                text,
                variations,
                output_file,
                format,
                width,
                height,
                auto_size,
                padding,
                crop,
                script,
                language,
                direction,
                features,
            )?;
        }

        Commands::Version => {
            println!("typf {}", env!("CARGO_PKG_VERSION"));
            println!("Open Font Engine - High-performance multi-backend text rendering");
        }
    }

    Ok(())
}

/// Initialize logging based on verbosity flag.
fn init_logger(verbose: bool) {
    if verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Warn)
            .init();
    }
}

/// Run batch mode: read JobSpec from stdin, process jobs in parallel, output JSONL results.
fn run_batch(max_fonts: usize, base_dir: Option<&Utf8Path>) -> Result<()> {
    let start = Instant::now();

    // Read entire stdin as JSON
    let stdin = io::stdin();
    let mut input = String::new();
    stdin
        .lock()
        .read_to_string(&mut input)
        .context("Failed to read stdin")?;

    // Parse job specification
    let spec: JobSpec =
        serde_json::from_str(&input).context("Failed to parse JSON job specification")?;

    // Validate job specification
    spec.validate()
        .context("Job specification validation failed")?;

    log::info!("Processing {} jobs", spec.jobs.len());

    // Create shared font loader
    let font_loader = FontLoader::new(max_fonts);

    // Process jobs in parallel
    let results: Vec<JobResult> = spec
        .jobs
        .par_iter()
        .map(|job| process_job(job, &font_loader, base_dir))
        .collect();

    // Output JSONL results to stdout
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for result in results {
        serde_json::to_writer(&mut handle, &result)
            .context("Failed to write JSON result")?;
        writeln!(&mut handle).context("Failed to write newline")?;
    }

    let elapsed = start.elapsed();
    log::info!(
        "Completed {} jobs in {:.2}s ({:.1} jobs/sec)",
        spec.jobs.len(),
        elapsed.as_secs_f64(),
        spec.jobs.len() as f64 / elapsed.as_secs_f64()
    );

    Ok(())
}

/// Run streaming mode: read JSONL from stdin, process each job, output JSONL results.
fn run_stream(max_fonts: usize, base_dir: Option<&Utf8Path>) -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out_handle = stdout.lock();

    let font_loader = FontLoader::new(max_fonts);

    for (line_num, line) in stdin.lock().lines().enumerate() {
        let line = line.context("Failed to read line from stdin")?;

        // Parse job from JSONL
        let job: Job = match serde_json::from_str(&line) {
            Ok(job) => job,
            Err(e) => {
                log::error!("Line {}: Failed to parse JSON: {}", line_num + 1, e);
                continue;
            }
        };

        // Process job
        let result = process_job(&job, &font_loader, base_dir);

        // Output result
        serde_json::to_writer(&mut out_handle, &result)
            .context("Failed to write JSON result")?;
        writeln!(&mut out_handle).context("Failed to write newline")?;
        out_handle.flush().context("Failed to flush stdout")?;
    }

    Ok(())
}

/// Process a single rendering job using native typf-zeno rendering.
fn process_job(job: &Job, font_loader: &FontLoader, base_dir: Option<&Utf8Path>) -> JobResult {
    use typf_batch::{FontResult, JobResult, MetricsOutput, RenderingOutput, TimingInfo};

    let start = Instant::now();

    // Validate job
    if let Err(e) = job.validate() {
        return JobResult {
            id: job.id.clone(),
            status: "error".to_string(),
            error: Some(format!("Validation failed: {}", e)),
            rendering: None,
            metrics: None,
            font: None,
            timing: TimingInfo {
                total_ms: start.elapsed().as_millis() as f64,
                shape_ms: 0.0,
                render_ms: 0.0,
            },
            memory: None,
        };
    }

    // Sanitize and resolve font path
    let font_path = match job.sanitize_font_path(base_dir) {
        Ok(path) => path,
        Err(e) => {
            return JobResult {
                id: job.id.clone(),
                status: "error".to_string(),
                error: Some(format!("Invalid font path: {}", e)),
                rendering: None,
                metrics: None,
                font: None,
                timing: TimingInfo {
                    total_ms: start.elapsed().as_millis() as f64,
                    shape_ms: 0.0,
                    render_ms: 0.0,
                },
                memory: None,
            };
        }
    };

    let font_load_start = Instant::now();

    // Load font (validates font is accessible)
    let font_instance = match font_loader.load_font(&font_path, &job.font.variations) {
        Ok(font) => font,
        Err(e) => {
            return JobResult {
                id: job.id.clone(),
                status: "error".to_string(),
                error: Some(format!("Failed to load font: {}", e)),
                rendering: None,
                metrics: None,
                font: None,
                timing: TimingInfo {
                    total_ms: start.elapsed().as_millis() as f64,
                    shape_ms: 0.0,
                    render_ms: 0.0,
                },
                memory: None,
            };
        }
    };

    let _font_load_ms = font_load_start.elapsed().as_millis() as f64;
    let shaping_start = Instant::now();

    // Shape text (validates shaping works)
    let shaper = TextShaper;
    let shape_request = ShapeRequest {
        text: &job.text.content,
        script: job.text.script.as_deref(),
        direction: job.text.direction.as_deref(),
        language: job.text.language.as_deref(),
        features: &job.text.features,
    };

    let shaped = match shaper.shape_with_request(
        &font_instance,
        &shape_request,
        job.font.size as f32,
        font_path.as_std_path(),
    ) {
        Ok(shaped) => shaped,
        Err(e) => {
            return JobResult {
                id: job.id.clone(),
                status: "error".to_string(),
                error: Some(format!("Shaping failed: {}", e)),
                rendering: None,
                metrics: None,
                font: None,
                timing: TimingInfo {
                    total_ms: start.elapsed().as_millis() as f64,
                    shape_ms: shaping_start.elapsed().as_millis() as f64,
                    render_ms: 0.0,
                },
                memory: None,
            };
        }
    };

    let shape_ms = shaping_start.elapsed().as_millis() as f64;
    let render_start = Instant::now();

    // Render using native typf-zeno backend
    let rasterizer = GlyphRasterizer::new();
    let render_result = rasterizer.render_text(
        &font_instance,
        &shaped,
        job.rendering.width,
        job.rendering.height,
        0.0, // tracking (not used in current spec)
        font_path.as_std_path(),
    );

    let render_ms = render_start.elapsed().as_millis() as f64;
    let total_ms = start.elapsed().as_millis() as f64;

    match render_result {
        Ok(image) => {
            // Check what format was requested
            match job.rendering.format.as_str() {
                "svg" => {
                    // SVG output
                    let actual_bbox = image.calculate_bbox();

                    // Convert bbox tuple to BoundingBox struct
                    let bbox = BoundingBox {
                        x: actual_bbox.0 as f32,
                        y: actual_bbox.1 as f32,
                        width: actual_bbox.2 as f32,
                        height: actual_bbox.3 as f32,
                    };

                    // Convert ShapedText to ShapingResult for SVG renderer
                    let shaping_result = shaped_text_to_shaping_result(&shaped, &job.text.content, bbox);

                    let svg_renderer = SvgRenderer::new(&SvgOptions::default());
                    let svg_output = svg_renderer.render(&shaping_result, &SvgOptions::default());

                    // Encode as base64 for JSON transport
                    let data = match job.rendering.encoding.as_str() {
                        "base64" => base64::engine::general_purpose::STANDARD.encode(svg_output.as_bytes()),
                        "binary" => {
                            return JobResult::error(
                                job.id.clone(),
                                "Binary encoding not supported for SVG output",
                            );
                        }
                        _ => {
                            return JobResult::error(
                                job.id.clone(),
                                format!("Unsupported encoding: {}", job.rendering.encoding),
                            );
                        }
                    };

                    JobResult::success_render(
                        job.id.clone(),
                        RenderingOutput {
                            format: "svg".to_string(),
                            encoding: "base64".to_string(),
                            data,
                            width: job.rendering.width,
                            height: job.rendering.height,
                            actual_bbox,
                        },
                        FontResult {
                            path: font_path.to_string(),
                            variations: job.font.variations.clone(),
                        },
                        TimingInfo {
                            total_ms,
                            shape_ms,
                            render_ms,
                        },
                    )
                }
                "metrics" => {
                    // Metrics-only output
                    let density = image.density();
                    let beam = image.beam();
                    JobResult::success_metrics(
                        job.id.clone(),
                        MetricsOutput { density, beam },
                        FontResult {
                            path: font_path.to_string(),
                            variations: job.font.variations.clone(),
                        },
                        TimingInfo {
                            total_ms,
                            shape_ms,
                            render_ms,
                        },
                    )
                }
                "pgm" | "png" => {
                    // Image output
                    let pixels = image.pixels();
                    let width = image.width();
                    let height = image.height();
                    let actual_bbox = image.calculate_bbox();

                    // Encode based on requested encoding
                    let data = match job.rendering.encoding.as_str() {
                        "base64" => {
                            // For PGM, encode the raw pixels with PGM header
                            if job.rendering.format == "pgm" {
                                let mut pgm_data = Vec::new();
                                // PGM header: P5 <width> <height> 255
                                pgm_data.extend_from_slice(
                                    format!("P5\n{} {}\n255\n", width, height).as_bytes(),
                                );
                                pgm_data.extend_from_slice(pixels);
                                base64::engine::general_purpose::STANDARD.encode(&pgm_data)
                            } else {
                                // PNG encoding
                                let mut png_data = Vec::new();
                                {
                                    let mut encoder = png::Encoder::new(
                                        &mut png_data,
                                        width,
                                        height,
                                    );
                                    encoder.set_color(png::ColorType::Grayscale);
                                    encoder.set_depth(png::BitDepth::Eight);
                                    let mut writer = encoder.write_header().unwrap();
                                    writer.write_image_data(pixels).unwrap();
                                }
                                base64::engine::general_purpose::STANDARD.encode(&png_data)
                            }
                        }
                        "binary" => {
                            // Binary output not typical for JSONL
                            return JobResult::error(
                                job.id.clone(),
                                "Binary encoding not supported in JSONL output",
                            );
                        }
                        _ => {
                            return JobResult::error(
                                job.id.clone(),
                                format!("Unsupported encoding: {}", job.rendering.encoding),
                            );
                        }
                    };

                    JobResult::success_render(
                        job.id.clone(),
                        RenderingOutput {
                            format: job.rendering.format.clone(),
                            encoding: "base64".to_string(),
                            data,
                            width,
                            height,
                            actual_bbox,
                        },
                        FontResult {
                            path: font_path.to_string(),
                            variations: job.font.variations.clone(),
                        },
                        TimingInfo {
                            total_ms,
                            shape_ms,
                            render_ms,
                        },
                    )
                }
                _ => JobResult::error(
                    job.id.clone(),
                    format!("Unsupported output format: {}", job.rendering.format),
                ),
            }
        }
        Err(e) => JobResult {
            id: job.id.clone(),
            status: "error".to_string(),
            error: Some(format!("Rendering failed: {}", e)),
            rendering: None,
            metrics: None,
            font: Some(FontResult {
                path: font_path.to_string(),
                variations: job.font.variations.clone(),
            }),
            timing: TimingInfo {
                total_ms,
                shape_ms,
                render_ms,
            },
            memory: None,
        },
    }
}

/// Run render mode: render a single text string and output to file or stdout.
#[allow(clippy::too_many_arguments)]
fn run_render(
    font_file: Utf8PathBuf,
    font_size: f32,
    text: String,
    variations: Option<String>,
    output_file: Option<Utf8PathBuf>,
    format: String,
    mut width: u32,
    mut height: u32,
    auto_size: bool,
    padding: u32,
    crop: bool,
    script: Option<String>,
    language: Option<String>,
    direction: String,
    features: Option<String>,
) -> Result<()> {
    use typf_batch::{FontConfig, Job, RenderingConfig, TextConfig};
    use std::collections::HashMap;

    // Parse variations from string format
    let mut var_map = HashMap::new();
    if let Some(vars) = variations {
        for pair in vars.split(',') {
            if let Some((key, val)) = pair.split_once('=') {
                if let Ok(value) = val.parse::<f32>() {
                    var_map.insert(key.to_string(), value);
                }
            }
        }
    }

    // Parse features
    let mut feature_list = Vec::new();
    if let Some(list) = features {
        for item in list.split(',') {
            let trimmed = item.trim();
            if !trimmed.is_empty() {
                feature_list.push(trimmed.to_string());
            }
        }
    }

    let font_size_u32 = font_size.max(1.0).round() as u32;

    // Auto-size canvas if requested
    if auto_size {
        // Quick estimate: use font size to calculate dimensions
        // This is a heuristic - more accurate sizing would require pre-shaping
        let char_count = text.chars().count() as f32;
        let estimated_width = (font_size * 0.6 * char_count) as u32 + (padding * 2);
        let estimated_height = (font_size * 1.4) as u32 + (padding * 2);

        width = estimated_width;
        height = estimated_height;

        log::debug!("Auto-sized canvas: {}x{} (text: '{}', size: {}, padding: {})",
                   width, height, text, font_size, padding);
    }

    // Create a job
    let job = Job {
        id: "render".to_string(),
        font: FontConfig {
            path: font_file,
            size: font_size_u32,
            variations: var_map,
        },
        text: TextConfig {
            content: text,
            script,
            direction: Some(direction),
            language,
            features: feature_list,
        },
        rendering: RenderingConfig {
            format: format.clone(),
            encoding: if format == "metrics" {
                "json".to_string()
            } else {
                "base64".to_string()
            },
            width,
            height,
        },
    };

    // Process the job
    let font_loader = FontLoader::new(1);
    let result = process_job(&job, &font_loader, None);

    // Output the result
    if let Some(output_path) = output_file {
        if format == "metrics" {
            // Write JSON metrics to file
            std::fs::write(&output_path, serde_json::to_string_pretty(&result)?)?;
            println!("Metrics written to: {}", output_path);
        } else if format == "svg" {
            // Write SVG directly (SVG doesn't support crop)
            if result.status == "success" {
                if let Some(rendering) = result.rendering {
                    let svg_data = base64::engine::general_purpose::STANDARD.decode(rendering.data)?;
                    std::fs::write(&output_path, svg_data)?;
                    println!("SVG written to: {}", output_path);
                }
            } else {
                eprintln!("Render failed: {}", result.error.unwrap_or_default());
                std::process::exit(1);
            }
        } else if result.status == "success" {
            if let Some(rendering) = result.rendering {
                let mut image_bytes = base64::engine::general_purpose::STANDARD.decode(rendering.data)?;

                // Apply crop if requested (only for bitmap formats)
                if crop && (format == "pgm" || format == "png") {
                    // Decode the image format to get raw pixels
                    if format == "pgm" {
                        // Parse PGM header and extract pixels
                        if let Some(data_start) = find_pgm_data_start(&image_bytes) {
                            let pixels = &image_bytes[data_start..];
                            let (cropped, new_width, new_height) = crop_image(pixels, rendering.width, rendering.height);

                            // Rebuild PGM with new dimensions
                            let mut new_pgm = Vec::new();
                            write!(&mut new_pgm, "P5\n{} {}\n255\n", new_width, new_height)?;
                            new_pgm.extend_from_slice(&cropped);
                            image_bytes = new_pgm;

                            log::debug!("Cropped PGM from {}x{} to {}x{}",
                                       rendering.width, rendering.height, new_width, new_height);
                        }
                    } else if format == "png" {
                        // For PNG, we'd need to decode, crop, re-encode
                        // This requires the `image` crate which we already have
                        use image::{ImageBuffer, Luma};

                        if let Ok(img) = image::load_from_memory(&image_bytes) {
                            let gray_img = img.to_luma8();
                            let (img_width, img_height) = gray_img.dimensions();
                            let (cropped_pixels, new_width, new_height) = crop_image(
                                gray_img.as_raw(),
                                img_width,
                                img_height
                            );

                            // Create new image from cropped pixels
                            if let Some(cropped_img) = ImageBuffer::<Luma<u8>, Vec<u8>>::from_raw(
                                new_width,
                                new_height,
                                cropped_pixels
                            ) {
                                let mut png_data = Vec::new();
                                cropped_img.write_to(
                                    &mut std::io::Cursor::new(&mut png_data),
                                    image::ImageFormat::Png
                                )?;
                                image_bytes = png_data;

                                log::debug!("Cropped PNG from {}x{} to {}x{}",
                                           img_width, img_height, new_width, new_height);
                            }
                        }
                    }
                }

                std::fs::write(&output_path, image_bytes)?;
                println!("Image written to: {}", output_path);
            }
        } else {
            eprintln!("Render failed: {}", result.error.unwrap_or_default());
            std::process::exit(1);
        }
    } else {
        // Output to stdout
        if format == "metrics" {
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else if format == "svg" {
            // Output SVG to stdout
            if result.status == "success" {
                if let Some(rendering) = result.rendering {
                    let svg_data = base64::engine::general_purpose::STANDARD.decode(rendering.data)?;
                    std::io::stdout().write_all(&svg_data)?;
                }
            } else {
                eprintln!("Render failed: {}", result.error.unwrap_or_default());
                std::process::exit(1);
            }
        } else if result.status == "success" {
            if let Some(rendering) = result.rendering {
                let image_bytes = base64::engine::general_purpose::STANDARD.decode(rendering.data)?;
                std::io::stdout().write_all(&image_bytes)?;
            }
        } else {
            eprintln!("Render failed: {}", result.error.unwrap_or_default());
            std::process::exit(1);
        }
    }

    Ok(())
}
