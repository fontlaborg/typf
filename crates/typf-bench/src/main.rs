//! Typf Benchmark Tool - Comprehensive performance testing
//!
//! This tool performs extensive benchmarking of all compiled Typf backends
//! across various fonts, sizes, texts, and parameter combinations. Results
//! are output progressively to ensure crash recovery.

use clap::Parser;
use colored::*;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use typf_core::{
    traits::{Renderer, Shaper},
    types::Direction,
    Color, RenderParams, ShapingParams, TypfError,
};
use typf_fontdb::TypfFontFace;
use typf_render_opixa::OpixaRenderer;
use typf_shape_none::NoneShaper;

#[cfg(feature = "render-cg")]
use typf_render_cg::CoreGraphicsRenderer;
#[cfg(feature = "render-skia")]
use typf_render_skia::SkiaRenderer;
#[cfg(feature = "render-vello")]
use typf_render_vello::VelloRenderer;
#[cfg(feature = "render-vello-cpu")]
use typf_render_vello_cpu::VelloCpuRenderer;
#[cfg(feature = "render-zeno")]
use typf_render_zeno::ZenoRenderer;
#[cfg(feature = "shaping-ct")]
use typf_shape_ct::CoreTextShaper;
#[cfg(feature = "shaping-hb")]
use typf_shape_hb::HarfBuzzShaper;
#[cfg(feature = "shaping-icu-hb")]
use typf_shape_icu_hb::IcuHarfBuzzShaper;

// Linra renderer (single-pass shaping+rendering)
#[cfg(all(feature = "linra-os-mac", target_os = "macos"))]
use typf_core::linra::{LinraRenderParams, LinraRenderer};
#[cfg(all(feature = "linra-os-mac", target_os = "macos"))]
use typf_os_mac::CoreTextLinraRenderer;

/// Benchmark configuration for different intensity levels
#[derive(Debug, Clone)]
struct BenchmarkConfig {
    font_sizes: Vec<f32>,
    sample_texts: Vec<&'static str>,
    text_lengths: Vec<usize>,
    #[allow(dead_code)]
    render_sizes: Vec<(u32, u32)>,
    iterations_per_combo: u32,
}

impl BenchmarkConfig {
    fn get(level: u8) -> Self {
        match level {
            // Level 0: Ultra-quick sanity check (< 10 seconds)
            0 => Self {
                font_sizes: vec![24.0],
                sample_texts: vec![
                    "Hello World",
                    "The quick brown fox",
                ],
                text_lengths: vec![20],
                render_sizes: vec![(200, 100)],
                iterations_per_combo: 5,
            },
            1 => Self {
                font_sizes: vec![12.0, 16.0, 24.0, 36.0, 48.0],
                sample_texts: vec![
                    "Hello World",
                    "The quick brown fox jumps over the lazy dog",
                    "Lorem ipsum dolor sit amet",
                    "Hello 世界 مرحبا עולם",
                ],
                text_lengths: vec![10, 50, 100, 500],
                render_sizes: vec![(100, 50), (200, 100), (400, 200), (800, 400)],
                iterations_per_combo: 10,
            },
            2 => Self {
                font_sizes: vec![8.0, 12.0, 16.0, 24.0, 36.0, 48.0, 72.0, 96.0],
                sample_texts: vec![
                    "Hello World",
                    "The quick brown fox jumps over the lazy dog",
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit",
                    "Hello 世界 مرحبا עולם Здравствуй мир",
                    "1234567890 !@#$%^&*()",
                ],
                text_lengths: vec![10, 25, 50, 100, 250, 500, 1000],
                render_sizes: vec![(100, 50), (200, 100), (400, 200), (800, 400), (1600, 800)],
                iterations_per_combo: 20,
            },
            3 => Self {
                font_sizes: vec![6.0, 8.0, 12.0, 16.0, 24.0, 36.0, 48.0, 72.0, 96.0, 144.0],
                sample_texts: vec![
                    "A",
                    "Hello World",
                    "The quick brown fox jumps over the lazy dog",
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt",
                    "Hello 世界 مرحبا עולם Здравствуй мир 안녕하세요",
                    "1234567890 !@#$%^&*()_+-=[]{}|;':\",./<>?",
                ],
                text_lengths: vec![1, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000],
                render_sizes: vec![(50, 25), (100, 50), (200, 100), (400, 200), (800, 400), (1600, 800), (3200, 1600)],
                iterations_per_combo: 50,
            },
            4 => Self {
                font_sizes: vec![4.0, 6.0, 8.0, 12.0, 16.0, 24.0, 36.0, 48.0, 72.0, 96.0, 144.0, 288.0],
                sample_texts: vec![
                    "A",
                    "Hello",
                    "Hello World",
                    "The quick brown fox jumps over the lazy dog",
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua",
                    "Hello 世界 مرحبا עולם こんにちは 안녕하세요 你好世界",
                    "1234567890 !@#$%^&*()_+-=[]{}|;':\",./<>?`~",
                ],
                text_lengths: vec![1, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000],
                render_sizes: vec![(25, 12), (50, 25), (100, 50), (200, 100), (400, 200), (800, 400), (1600, 800), (3200, 1600), (6400, 3200)],
                iterations_per_combo: 100,
            },
            5 => Self {
                font_sizes: vec![2.0, 4.0, 6.0, 8.0, 12.0, 16.0, 24.0, 36.0, 48.0, 72.0, 96.0, 144.0, 288.0, 576.0],
                sample_texts: vec![
                    "A",
                    "Hello",
                    "Hello World",
                    "The quick brown fox jumps over the lazy dog",
                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris",
                    "Hello 世界 مرحبا עולם こんにちは 안녕하세요 你好世界 مرحبا Здравствуй мир שלום",
                    "1234567890 !@#$%^&*()_+-=[]{}|;':\",./<>?`~",
                    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
                ],
                text_lengths: vec![1, 2, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000, 25000],
                render_sizes: vec![(12, 6), (25, 12), (50, 25), (100, 50), (200, 100), (400, 200), (800, 400), (1600, 800), (3200, 1600), (6400, 3200), (12800, 6400)],
                iterations_per_combo: 200,
            },
            _ => Self::get(1),
        }
    }
}

/// Command line arguments
#[derive(Parser)]
#[command(name = "typf-bench")]
#[command(about = "Comprehensive Typf benchmarking tool")]
struct Args {
    /// Directory containing font files (.ttf, .otf)
    #[arg(short = 'i', long = "input_dir")]
    input_dir: String,

    /// Benchmark intensity level (1-5, higher = more extensive)
    #[arg(short = 'l', long = "level", default_value = "1")]
    level: u8,

    /// Output results as JSON (for CI comparison)
    #[arg(short = 'j', long = "json")]
    json_output: bool,

    /// Output file for JSON results (default: stdout)
    #[arg(short = 'o', long = "output")]
    output_file: Option<String>,
}

/// Benchmark result for a single combination
#[derive(Debug, serde::Serialize)]
struct BenchmarkResult {
    shaper_name: String,
    renderer_name: String,
    font_name: String,
    text_sample: String,
    font_size: f32,
    text_length: usize,
    render_size: (u32, u32),
    ns_per_op: f64,
    total_time_ns: u128,
    iterations: u32,
}

/// JSON output structure for benchmark results
#[derive(Debug, serde::Serialize)]
struct BenchmarkOutput {
    version: String,
    timestamp: String,
    level: u8,
    results: Vec<BenchmarkResult>,
}

/// Main benchmark runner
struct BenchmarkRunner {
    fonts: Vec<Arc<TypfFontFace>>,
    config: BenchmarkConfig,
    json_output: bool,
    level: u8,
}

impl BenchmarkRunner {
    fn new(
        input_dir: &str,
        config: BenchmarkConfig,
        json_output: bool,
        level: u8,
    ) -> Result<Self, TypfError> {
        let mut fonts = Vec::new();
        let font_dir = Path::new(input_dir);

        if !font_dir.exists() {
            return Err(TypfError::FontLoad(
                typf_core::error::FontLoadError::FileNotFound(input_dir.to_string()),
            ));
        }

        // Discover font files
        for entry in fs::read_dir(font_dir)
            .map_err(|_| TypfError::FontLoad(typf_core::error::FontLoadError::InvalidData))?
        {
            let entry = entry
                .map_err(|_| TypfError::FontLoad(typf_core::error::FontLoadError::InvalidData))?;
            let path = entry.path();

            if let Some(extension) = path.extension() {
                if let Some(ext_str) = extension.to_str() {
                    if matches!(
                        ext_str.to_lowercase().as_str(),
                        "ttf" | "otf" | "ttc" | "woff" | "woff2"
                    ) {
                        match TypfFontFace::from_file(&path) {
                            Ok(font) => {
                                if !json_output {
                                    println!(
                                        "{}",
                                        format!("Loaded font: {}", path.display()).green()
                                    );
                                }
                                fonts.push(Arc::new(font));
                            },
                            Err(e) => {
                                if !json_output {
                                    eprintln!(
                                        "{}",
                                        format!(
                                            "Warning: Failed to load {}: {}",
                                            path.display(),
                                            e
                                        )
                                        .yellow()
                                    );
                                }
                            },
                        }
                    }
                }
            }
        }

        if fonts.is_empty() {
            return Err(TypfError::FontLoad(
                typf_core::error::FontLoadError::FileNotFound(
                    "No valid font files found".to_string(),
                ),
            ));
        }

        if !json_output {
            println!(
                "{}",
                format!("Loaded {} fonts for benchmarking", fonts.len()).cyan()
            );
        }

        Ok(Self {
            fonts,
            config,
            json_output,
            level,
        })
    }

    #[allow(clippy::vec_init_then_push)]
    fn get_shapers(&self) -> Vec<Arc<dyn Shaper>> {
        let mut shapers: Vec<Arc<dyn Shaper>> = Vec::new();

        shapers.push(Arc::new(NoneShaper::new()));

        #[cfg(feature = "shaping-hb")]
        shapers.push(Arc::new(HarfBuzzShaper::new()));

        #[cfg(feature = "shaping-ct")]
        shapers.push(Arc::new(CoreTextShaper::new()));

        #[cfg(feature = "shaping-icu-hb")]
        shapers.push(Arc::new(IcuHarfBuzzShaper::new()));

        shapers
    }

    #[allow(clippy::vec_init_then_push)]
    fn get_renderers(&self) -> Vec<Arc<dyn Renderer>> {
        let mut renderers: Vec<Arc<dyn Renderer>> = Vec::new();

        renderers.push(Arc::new(OpixaRenderer::new()));

        #[cfg(feature = "render-skia")]
        renderers.push(Arc::new(SkiaRenderer::new()));

        #[cfg(feature = "render-zeno")]
        renderers.push(Arc::new(ZenoRenderer::new()));

        #[cfg(feature = "render-cg")]
        renderers.push(Arc::new(CoreGraphicsRenderer::new()));

        #[cfg(feature = "render-vello-cpu")]
        renderers.push(Arc::new(VelloCpuRenderer::new()));

        #[cfg(feature = "render-vello")]
        if let Ok(renderer) = VelloRenderer::new() {
            renderers.push(Arc::new(renderer));
        }

        renderers
    }

    fn generate_text_sample(&self, base_text: &str, target_length: usize) -> String {
        if base_text.len() >= target_length {
            return base_text.chars().take(target_length).collect();
        }

        let mut result = String::new();
        while result.len() < target_length {
            result.push_str(base_text);
        }
        result.chars().take(target_length).collect()
    }

    fn benchmark_combination(
        &self,
        shaper: Arc<dyn Shaper>,
        renderer: Arc<dyn Renderer>,
        font: Arc<TypfFontFace>,
        text: &str,
        font_size: f32,
    ) -> Result<BenchmarkResult, TypfError> {
        let iterations = self.config.iterations_per_combo;

        // Warmup iterations to avoid cold start effects
        for _ in 0..3 {
            let shaping_params = ShapingParams {
                size: font_size,
                direction: Direction::LeftToRight,
                ..Default::default()
            };
            let shaped = shaper.shape(text, font.clone(), &shaping_params)?;
            let render_params = RenderParams {
                foreground: Color::black(),
                background: Some(Color::white()),
                ..Default::default()
            };
            let _rendered = renderer.render(&shaped, font.clone(), &render_params)?;
        }

        // Timing setup - use more precise measurement
        let start_time = std::time::Instant::now();

        for _ in 0..iterations {
            // Shape the text
            let shaping_params = ShapingParams {
                size: font_size,
                direction: Direction::LeftToRight,
                ..Default::default()
            };

            let shaped = shaper.shape(text, font.clone(), &shaping_params)?;

            // Render the shaped text
            let render_params = RenderParams {
                foreground: Color::black(),
                background: Some(Color::white()),
                ..Default::default()
            };

            let _rendered = renderer.render(&shaped, font.clone(), &render_params)?;
        }

        let elapsed = start_time.elapsed();
        let total_time_ns = elapsed.as_nanos();
        let ns_per_op = if total_time_ns > 0 {
            total_time_ns as f64 / iterations as f64
        } else {
            0.0
        };

        let text_sample = if text.len() > 20 {
            // Safe Unicode truncation by character count, not byte count
            let chars: Vec<char> = text.chars().take(17).collect(); // 17 chars + "..." = 20
            format!("{}...", chars.into_iter().collect::<String>())
        } else {
            text.to_string()
        };

        Ok(BenchmarkResult {
            shaper_name: shaper.name().to_string(),
            renderer_name: renderer.name().to_string(),
            font_name: format!("font_{}", std::ptr::addr_of!(*font) as usize), // Simple identifier
            text_sample,
            font_size,
            text_length: text.len(),
            render_size: (0, 0), // Simplified for now
            ns_per_op,
            total_time_ns,
            iterations,
        })
    }

    fn run_benchmarks(&self) -> Result<Vec<BenchmarkResult>, TypfError> {
        let shapers = self.get_shapers();
        let renderers = self.get_renderers();

        if !self.json_output {
            println!(
                "\n{}",
                "Starting comprehensive benchmark suite...".bold().cyan()
            );
            println!(
                "{} shapers × {} renderers × {} fonts × {} sizes × {} texts × {} lengths = {} combinations",
                shapers.len(),
                renderers.len(),
                self.fonts.len(),
                self.config.font_sizes.len(),
                self.config.sample_texts.len(),
                self.config.text_lengths.len(),
                shapers.len() * renderers.len() * self.fonts.len() * self.config.font_sizes.len() * self.config.sample_texts.len() * self.config.text_lengths.len()
            );

            println!("{}", "\nBenchmark Results:".bold());
            println!("{}", "─".repeat(80));
        }

        let mut all_results = Vec::new();

        for font in &self.fonts {
            for shaper in &shapers {
                for renderer in &renderers {
                    for &font_size in &self.config.font_sizes {
                        for sample_text in &self.config.sample_texts {
                            for &target_length in &self.config.text_lengths {
                                let text = self.generate_text_sample(sample_text, target_length);

                                match self.benchmark_combination(
                                    shaper.clone(),
                                    renderer.clone(),
                                    font.clone(),
                                    &text,
                                    font_size,
                                ) {
                                    Ok(result) => {
                                        if !self.json_output {
                                            // Output progressive result and flush
                                            println!(
                                                "{}",
                                                format!(
                                                    "S: {:12} | R: {:12} | Size: {:6.1} | Text: {:20} | Length: {:4} | ns/op: {:10.1}",
                                                    result.shaper_name,
                                                    result.renderer_name,
                                                    result.font_size,
                                                    result.text_sample,
                                                    result.text_length,
                                                    result.ns_per_op
                                                ).bright_black()
                                            );
                                            std::io::Write::flush(&mut std::io::stdout()).unwrap();
                                        }
                                        all_results.push(result);
                                    },
                                    Err(e) => {
                                        if !self.json_output {
                                            eprintln!(
                                                "{}",
                                                format!("Error in combination: {}", e).red()
                                            );
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }

        if !self.json_output {
            println!("{}", "─".repeat(80));
            println!(
                "{}",
                format!("Completed {} benchmark combinations", all_results.len())
                    .bold()
                    .green()
            );
        }

        Ok(all_results)
    }

    /// Output results as JSON
    fn output_json(
        &self,
        results: Vec<BenchmarkResult>,
        output_file: Option<&str>,
    ) -> Result<(), TypfError> {
        let output = BenchmarkOutput {
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            level: self.level,
            results,
        };

        let json = serde_json::to_string_pretty(&output)
            .map_err(|e| TypfError::Other(format!("JSON serialization failed: {}", e)))?;

        match output_file {
            Some(path) => {
                fs::write(path, &json).map_err(TypfError::Io)?;
            },
            None => {
                println!("{}", json);
            },
        }

        Ok(())
    }

    /// Run linra renderer benchmarks (single-pass shaping+rendering)
    #[cfg(all(feature = "linra-os-mac", target_os = "macos"))]
    fn run_linra_benchmarks(&self) -> Result<(), TypfError> {
        use typf_core::types::Direction;

        println!(
            "\n{}",
            "Linra Renderer Benchmark (CoreText single-pass):"
                .bold()
                .cyan()
        );
        println!("{}", "─".repeat(80));

        let linra = CoreTextLinraRenderer::new();
        let mut total_combinations = 0;

        for font in &self.fonts {
            for &font_size in &self.config.font_sizes {
                for sample_text in &self.config.sample_texts {
                    for &target_length in &self.config.text_lengths {
                        let text = self.generate_text_sample(sample_text, target_length);
                        let iterations = self.config.iterations_per_combo;

                        // Build params
                        let params = LinraRenderParams {
                            size: font_size,
                            direction: Direction::LeftToRight,
                            foreground: Color::black(),
                            background: Some(Color::white()),
                            ..Default::default()
                        };

                        // Warmup
                        for _ in 0..3 {
                            let _ = linra.render_text(&text, font.clone(), &params);
                        }

                        // Benchmark
                        let start_time = std::time::Instant::now();
                        for _ in 0..iterations {
                            let _ = linra.render_text(&text, font.clone(), &params);
                        }
                        let elapsed = start_time.elapsed();
                        let ns_per_op = elapsed.as_nanos() as f64 / iterations as f64;

                        let text_sample = if text.len() > 20 {
                            let chars: Vec<char> = text.chars().take(17).collect();
                            format!("{}...", chars.into_iter().collect::<String>())
                        } else {
                            text.to_string()
                        };

                        println!(
                            "{}",
                            format!(
                                "U: {:12} |              | Size: {:6.1} | Text: {:20} | Length: {:4} | ns/op: {:10.1}",
                                "linra-mac",
                                font_size,
                                text_sample,
                                text.len(),
                                ns_per_op
                            ).bright_blue()
                        );
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();

                        total_combinations += 1;
                    }
                }
            }
        }

        println!("{}", "─".repeat(80));
        println!(
            "{}",
            format!(
                "Completed {} linra benchmark combinations",
                total_combinations
            )
            .bold()
            .green()
        );

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    let args = Args::parse();

    // Validate level parameter (0-5, 0 = ultra-quick)
    if args.level > 5 {
        eprintln!("Error: Level must be between 0 and 5 (got {})", args.level);
        std::process::exit(1);
    }

    let config = BenchmarkConfig::get(args.level);

    if !args.json_output {
        println!("{}", "Typf Comprehensive Benchmark Tool".bold().cyan());
        println!(
            "Input directory: {} | Level: {}",
            args.input_dir, args.level
        );
    }

    let runner = BenchmarkRunner::new(&args.input_dir, config, args.json_output, args.level)?;
    let results = runner.run_benchmarks()?;

    // Output JSON if requested
    if args.json_output {
        runner.output_json(results, args.output_file.as_deref())?;
    }

    // Run linra renderer benchmark if available (not in JSON mode for now)
    #[cfg(all(feature = "linra-os-mac", target_os = "macos"))]
    if !args.json_output {
        runner.run_linra_benchmarks()?;
    }

    Ok(())
}
