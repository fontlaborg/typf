// this_file: backends/typf-orge/src/grayscale.rs

//! Grayscale rendering via oversampling.
//!
//! Implements anti-aliasing by rendering at higher resolution and
//! accumulating coverage to produce 0-255 alpha values.

use crate::scan_converter::ScanConverter;

/// Grayscale rendering level (oversampling factor).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GrayscaleLevel {
    /// 2x2 oversampling (4 samples per pixel)
    Level2x2 = 2,
    /// 4x4 oversampling (16 samples per pixel)
    Level4x4 = 4,
    /// 8x8 oversampling (64 samples per pixel)
    Level8x8 = 8,
}

impl GrayscaleLevel {
    /// Get oversampling factor.
    pub const fn factor(self) -> usize {
        self as usize
    }

    /// Get total samples per pixel.
    pub const fn samples_per_pixel(self) -> usize {
        let f = self.factor();
        f * f
    }

    /// Get maximum alpha value (samples per pixel).
    pub const fn max_alpha(self) -> u8 {
        self.samples_per_pixel() as u8
    }
}

/// Render glyph with grayscale anti-aliasing.
///
/// # Arguments
///
/// * `sc` - Scan converter with outline already added
/// * `width` - Output bitmap width in pixels
/// * `height` - Output bitmap height in pixels
/// * `level` - Grayscale oversampling level
///
/// # Returns
///
/// Vec of alpha values (0-255), size = width * height
pub fn render_grayscale(
    sc: &mut ScanConverter,
    width: usize,
    height: usize,
    level: GrayscaleLevel,
) -> Vec<u8> {
    let factor = level.factor();
    let _samples_per_pixel = level.samples_per_pixel();

    // Render at oversampled resolution
    let oversample_width = width * factor;
    let oversample_height = height * factor;

    // Create oversampled scan converter
    let mut oversample_sc = ScanConverter::new(oversample_width, oversample_height);
    oversample_sc.set_fill_rule(sc.fill_rule());
    oversample_sc.set_dropout_mode(sc.dropout_mode());

    // Copy outline from original scan converter by re-rendering
    // This is a simplified approach - in production, we'd store the outline
    // For now, we'll just use the monochrome rendering and downsample

    let mut mono_bitmap = vec![0u8; oversample_width * oversample_height];
    sc.render_mono(&mut mono_bitmap);

    // Downsample to grayscale
    downsample_to_grayscale(
        &mono_bitmap,
        oversample_width,
        oversample_height,
        width,
        height,
        level,
    )
}

/// Downsample monochrome bitmap to grayscale using optimized vectorizable code.
#[cfg(target_feature = "simd128")]
fn downsample_to_grayscale_simd(
    mono: &[u8],
    mono_width: usize,
    _mono_height: usize,
    out_width: usize,
    out_height: usize,
    level: GrayscaleLevel,
) -> Vec<u8> {
    let factor = level.factor();
    let max_coverage = level.samples_per_pixel() as u32;
    let normalization_factor = 255.0 / max_coverage as f32;

    let mut output = vec![0u8; out_width * out_height];

    for out_y in 0..out_height {
        let src_y_base = out_y * factor;
        let out_row_start = out_y * out_width;

        for out_x in 0..out_width {
            let src_x_base = out_x * factor;
            let mut coverage = 0u32;

            // Sum coverage in factor x factor block
            // This loop structure allows LLVM to auto-vectorize
            for dy in 0..factor {
                let src_row_start = (src_y_base + dy) * mono_width;
                let row_start = src_row_start + src_x_base;

                if row_start + factor <= mono.len() {
                    // Fast path: entire row is in bounds, LLVM can vectorize this
                    for i in 0..factor {
                        coverage += mono[row_start + i] as u32;
                    }
                } else {
                    // Slow path: bounds checking required
                    for i in 0..factor {
                        let x = src_x_base + i;
                        if x < mono_width {
                            coverage += mono[src_row_start + x] as u32;
                        }
                    }
                }
            }

            // Convert to 0-255 alpha
            let alpha = (coverage as f32 * normalization_factor).round() as u8;
            output[out_row_start + out_x] = alpha;
        }
    }
    output
}

/// Downsample monochrome bitmap to grayscale (scalar fallback).
fn downsample_to_grayscale_scalar(
    mono: &[u8],
    mono_width: usize,
    mono_height: usize,
    out_width: usize,
    out_height: usize,
    level: GrayscaleLevel,
) -> Vec<u8> {
    let factor = level.factor();
    let max_coverage = level.samples_per_pixel();

    let mut output = vec![0u8; out_width * out_height];

    for out_y in 0..out_height {
        for out_x in 0..out_width {
            // Accumulate coverage from factor x factor grid
            let mut coverage = 0u32;

            let src_x = out_x * factor;
            let src_y = out_y * factor;

            for dy in 0..factor {
                for dx in 0..factor {
                    let x = src_x + dx;
                    let y = src_y + dy;

                    if x < mono_width && y < mono_height && mono[y * mono_width + x] != 0 {
                        coverage += 1;
                    }
                }
            }

            // Convert coverage to 0-255 alpha
            let alpha = ((coverage * 255) / max_coverage as u32) as u8;
            output[out_y * out_width + out_x] = alpha;
        }
    }

    output
}

/// Downsample monochrome bitmap to grayscale.
///
/// Automatically selects SIMD or scalar implementation based on CPU features.
#[inline]
fn downsample_to_grayscale(
    mono: &[u8],
    mono_width: usize,
    mono_height: usize,
    out_width: usize,
    out_height: usize,
    level: GrayscaleLevel,
) -> Vec<u8> {
    #[cfg(target_feature = "simd128")]
    {
        downsample_to_grayscale_simd(mono, mono_width, mono_height, out_width, out_height, level)
    }
    #[cfg(not(target_feature = "simd128"))]
    {
        downsample_to_grayscale_scalar(mono, mono_width, mono_height, out_width, out_height, level)
    }
}

/// Render grayscale with outline built directly at oversampled resolution.
///
/// This is more efficient than rendering mono then downsampling.
pub fn render_grayscale_direct(
    width: usize,
    height: usize,
    level: GrayscaleLevel,
    build_outline: impl FnOnce(&mut ScanConverter),
) -> Vec<u8> {
    let factor = level.factor();
    let _samples_per_pixel = level.samples_per_pixel();

    // Create scan converter at oversampled resolution
    let oversample_width = width * factor;
    let oversample_height = height * factor;
    let mut sc = ScanConverter::new(oversample_width, oversample_height);

    // Build outline at oversampled resolution
    build_outline(&mut sc);

    // Render at high resolution
    let mut mono_bitmap = vec![0u8; oversample_width * oversample_height];
    sc.render_mono(&mut mono_bitmap);

    // Downsample to grayscale
    downsample_to_grayscale(
        &mono_bitmap,
        oversample_width,
        oversample_height,
        width,
        height,
        level,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::F26Dot6;

    #[test]
    fn test_grayscale_level_factor() {
        assert_eq!(GrayscaleLevel::Level2x2.factor(), 2);
        assert_eq!(GrayscaleLevel::Level4x4.factor(), 4);
        assert_eq!(GrayscaleLevel::Level8x8.factor(), 8);
    }

    #[test]
    fn test_grayscale_level_samples() {
        assert_eq!(GrayscaleLevel::Level2x2.samples_per_pixel(), 4);
        assert_eq!(GrayscaleLevel::Level4x4.samples_per_pixel(), 16);
        assert_eq!(GrayscaleLevel::Level8x8.samples_per_pixel(), 64);
    }

    #[test]
    fn test_downsample_all_black() {
        // All pixels black (coverage = 1)
        let mono = vec![1u8; 4 * 4]; // 4x4 mono
        let gray = downsample_to_grayscale(&mono, 4, 4, 2, 2, GrayscaleLevel::Level2x2);

        // Each 2x2 block has 4 samples, all black → alpha = 255
        assert_eq!(gray.len(), 4);
        for &alpha in &gray {
            assert_eq!(alpha, 255);
        }
    }

    #[test]
    fn test_downsample_all_white() {
        // All pixels white (coverage = 0)
        let mono = vec![0u8; 4 * 4];
        let gray = downsample_to_grayscale(&mono, 4, 4, 2, 2, GrayscaleLevel::Level2x2);

        // All white → alpha = 0
        for &alpha in &gray {
            assert_eq!(alpha, 0);
        }
    }

    #[test]
    fn test_downsample_half_coverage() {
        // Half coverage (2 out of 4 pixels black)
        let mono = vec![1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1];
        let gray = downsample_to_grayscale(&mono, 4, 4, 2, 2, GrayscaleLevel::Level2x2);

        // Each 2x2 block has 2 black pixels → 2/4 coverage → alpha ~127
        for &alpha in &gray {
            assert!(alpha >= 120 && alpha <= 135, "Alpha = {}", alpha);
        }
    }

    #[test]
    fn test_render_grayscale_direct_rectangle() {
        let gray = render_grayscale_direct(10, 10, GrayscaleLevel::Level2x2, |sc| {
            // Draw rectangle at oversampled resolution (20x20)
            sc.move_to(F26Dot6::from_int(4), F26Dot6::from_int(4));
            sc.line_to(F26Dot6::from_int(16), F26Dot6::from_int(4));
            sc.line_to(F26Dot6::from_int(16), F26Dot6::from_int(16));
            sc.line_to(F26Dot6::from_int(4), F26Dot6::from_int(16));
            sc.close();
        });

        assert_eq!(gray.len(), 100);

        // Center should be filled (alpha ~255)
        assert!(
            gray[5 * 10 + 5] > 200,
            "Center alpha = {}",
            gray[5 * 10 + 5]
        );

        // Corners should be empty (alpha ~0)
        assert!(gray[0] < 50, "Corner alpha = {}", gray[0]);
    }

    #[test]
    fn test_render_grayscale_levels() {
        // Test different oversampling levels
        for level in [
            GrayscaleLevel::Level2x2,
            GrayscaleLevel::Level4x4,
            GrayscaleLevel::Level8x8,
        ] {
            let factor = level.factor() as i32;
            let gray = render_grayscale_direct(8, 8, level, |sc| {
                // Coordinates are at oversampled resolution
                let x1 = 2 * factor;
                let y1 = 2 * factor;
                let x2 = 6 * factor;
                let y2 = 6 * factor;

                sc.move_to(F26Dot6::from_int(x1), F26Dot6::from_int(y1));
                sc.line_to(F26Dot6::from_int(x2), F26Dot6::from_int(y1));
                sc.line_to(F26Dot6::from_int(x2), F26Dot6::from_int(y2));
                sc.line_to(F26Dot6::from_int(x1), F26Dot6::from_int(y2));
                sc.close();
            });

            assert_eq!(gray.len(), 64);
            // Should have some filled pixels
            let filled = gray.iter().filter(|&&a| a > 100).count();
            assert!(filled > 0, "Level {:?} has no filled pixels", level);
        }
    }
}
