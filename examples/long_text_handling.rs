//! Example: Handling Long Text with Line Wrapping and SVG Export
//!
//! This example demonstrates strategies for rendering long text that exceeds
//! the bitmap width limit (~10,000 pixels). It shows:
//!
//! 1. Detecting when text is too long for bitmap rendering
//! 2. Using SVG export as an alternative (no width limits)
//! 3. Implementing simple line wrapping for multi-line bitmap rendering
//! 4. Measuring text width to make informed decisions
//!
//! Run with: `cargo run --example long_text_handling --features shaping-hb,export-svg`

fn main() {
    // Sample long text (from typography essay - ~1000 chars)
    let long_text = "Typography is the art and technique of arranging type to make written \
        language legible, readable, and appealing when displayed. The arrangement \
        of type involves selecting typefaces, point sizes, line lengths, line-spacing, \
        and letter-spacing, and adjusting the space between pairs of letters. \
        The term typography is also applied to the style, arrangement, and appearance \
        of the letters, numbers, and symbols created by the process. Type design is \
        a closely related craft, sometimes considered part of typography; most typographers \
        do not design typefaces, and some type designers do not consider themselves typographers. \
        Typography also may be used as an ornamental and decorative device, unrelated to the \
        communication of information. In contemporary use, the practice and study of typography \
        include a broad range, covering all aspects of letter design and application, both \
        mechanical (typesetting, type design, and typefaces) and manual (handwriting and calligraphy).";

    println!("TYPF Long Text Handling Examples");
    println!("{}", "=".repeat(80));
    println!("\nText length: {} characters\n", long_text.len());

    // Strategy 1: Check if text fits within bitmap limits
    println!("Strategy 1: Check Width Before Rendering");
    println!("{}", "-".repeat(80));

    let font_size = 48.0;
    let max_bitmap_width = 10_000;

    // Estimate: typical character width is ~0.5-0.6 of font size
    let estimated_char_width = font_size * 0.55;
    let estimated_width = (long_text.len() as f32 * estimated_char_width) as u32;

    println!("Font size: {}px", font_size);
    println!("Estimated width: {}px", estimated_width);
    println!("Bitmap limit: {}px", max_bitmap_width);

    if estimated_width > max_bitmap_width {
        println!("⚠️  Text too wide for bitmap rendering!");
        println!("   Recommendation: Use SVG export or line wrapping\n");
    }

    // Strategy 2: Use SVG export (no width limits)
    #[cfg(feature = "export-svg")]
    {
        println!("Strategy 2: SVG Export (No Width Limits)");
        println!("{}", "-".repeat(80));
        println!("✓ SVG export supports arbitrary text length");
        println!("  (Use render_to_svg() in Python bindings or CLI)\n");
    }

    // Strategy 3: Simple line wrapping
    println!("Strategy 3: Line Wrapping for Multi-line Rendering");
    println!("{}", "-".repeat(80));

    let max_chars_per_line = (max_bitmap_width as f32 / estimated_char_width) as usize;
    println!("Max characters per line: ~{}", max_chars_per_line);

    // Simple word-based line wrapping
    let lines = wrap_text(long_text, max_chars_per_line);
    println!("Wrapped into {} lines:", lines.len());

    for (i, line) in lines.iter().enumerate() {
        if i < 3 {
            println!("  Line {}: {} chars - \"{}...\"",
                     i + 1, line.len(), &line[..line.len().min(50)]);
        }
    }
    if lines.len() > 3 {
        println!("  ... {} more lines", lines.len() - 3);
    }
    println!();

    // Strategy 4: Adaptive font sizing
    println!("Strategy 4: Adaptive Font Sizing");
    println!("{}", "-".repeat(80));

    let target_width = 800; // Target width in pixels
    let adaptive_size = calculate_adaptive_font_size(long_text.len(), target_width);

    println!("For target width of {}px:", target_width);
    println!("Recommended font size: {:.1}px", adaptive_size);
    println!("This would fit {} characters\n",
             (target_width as f32 / (adaptive_size * 0.55)) as usize);

    // Strategy 5: Chunked rendering
    println!("Strategy 5: Chunked Rendering");
    println!("{}", "-".repeat(80));

    let chunk_size = 200; // Characters per chunk
    let chunks: Vec<String> = long_text
        .chars()
        .collect::<Vec<_>>()
        .chunks(chunk_size)
        .map(|c| c.iter().collect::<String>())
        .collect();

    println!("Split into {} chunks of ~{} characters:", chunks.len(), chunk_size);
    println!("Each chunk can be rendered separately and composited");
    println!("Chunk 1: \"{}...\"", &chunks[0][..50]);
    println!();

    println!("{}", "=".repeat(80));
    println!("Summary:");
    println!("- For text < 200 chars at 48px: Bitmap rendering works fine");
    println!("- For text > 200 chars: Use SVG export or line wrapping");
    println!("- For very long documents: Use adaptive sizing or chunking");
    println!("- SVG export is recommended for production use with long texts");
}

/// Simple word-based line wrapping
fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + word.len() + 1 <= max_chars {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Calculate adaptive font size to fit text in target width
fn calculate_adaptive_font_size(char_count: usize, target_width: u32) -> f32 {
    // Assuming ~0.55 character width ratio
    let char_width_ratio = 0.55;
    let max_font_size = 72.0; // Don't go above this
    let min_font_size = 8.0;  // Don't go below this

    let calculated_size = (target_width as f32) / (char_count as f32 * char_width_ratio);
    calculated_size.clamp(min_font_size, max_font_size)
}
