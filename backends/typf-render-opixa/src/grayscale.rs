//! Where beauty meets precision: the art of anti-aliased text
//!
//! Jagged edges betray amateur rendering. Professional text embraces
//! grayscale—rendering at higher resolution then gracefully downscaling
//! to create smooth edges that please the eye. This module transforms
//! monochrome precision into 256 levels of visual perfection.

use crate::scan_converter::ScanConverter;

/// The quality spectrum: how smooth do you want your text?
///
/// More samples mean smoother edges but slower rendering. Choose your
/// balance between speed and beauty based on your needs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GrayscaleLevel {
    /// Good enough: 2x2 oversampling for fast, decent results
    Level2x2 = 2,
    /// Sweet spot: 4x4 oversampling that most users love
    Level4x4 = 4,
    /// Perfectionist: 8x8 oversampling for magazine-quality text
    Level8x8 = 8,
}

impl GrayscaleLevel {
    /// How many pixels wide and tall we render internally
    pub const fn factor(self) -> usize {
        self as usize
    }

    /// The total sample count that determines alpha precision
    pub const fn samples_per_pixel(self) -> usize {
        let f = self.factor();
        f * f
    }

    /// The highest alpha value achievable at this quality level
    pub const fn max_alpha(self) -> u8 {
        self.samples_per_pixel() as u8
    }
}

/// Transform crisp edges into smooth beauty
///
/// We take your perfectly outlined glyph and apply oversampling magic.
/// The result is an alpha map where 255 means fully covered and 0 means
/// completely transparent—all the values in between create the visual
/// smoothness that makes text readable at any size.
///
/// # The Beauty Recipe
///
/// * `sc` - Your scan converter, loaded with glyph outlines
/// * `width` - How wide the final beauty will be
/// * `height` - How tall the final beauty will be
/// * `level` - Your chosen quality setting
///
/// # Returns
///
/// A vector of alpha values, each begging to be blended into your canvas
pub fn render_grayscale(
    sc: &mut ScanConverter,
    width: usize,
    height: usize,
    level: GrayscaleLevel,
) -> Vec<u8> {
    let factor = level.factor();
    let _samples_per_pixel = level.samples_per_pixel();

    // The scan converter is already at the correct resolution (oversampled)
    // We just need to render it and downsample
    // Note: The scan converter passed in should already be at the oversampled resolution
    let mono_width = width * factor;
    let mono_height = height * factor;

    let mut mono_bitmap = vec![0u8; mono_width * mono_height];
    sc.render_mono(&mut mono_bitmap);

    // Downsample to grayscale
    downsample_to_grayscale(&mono_bitmap, mono_width, mono_height, width, height, level)
}

/// SIMD-accelerated downsampling: when speed matters as much as beauty
///
/// Modern CPUs can process multiple pixels at once. This function leverages
/// SIMD instructions to transform high-resolution monochrome into smooth
/// grayscale with remarkable speed.
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

/// The reliable workhorse: pixel-by-pixel grayscale transformation
///
/// When SIMD isn't available, we fall back to careful scalar processing.
/// Slower, but compatible with every CPU and equally precise.
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

/// Choose your weapon: SIMD or scalar, automatically selected
///
/// We detect CPU capabilities at compile time and choose the fastest
/// available implementation. No configuration needed—just performance.
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

/// Build at high resolution, render at perfection
///
/// Instead of rendering twice (mono then downsample), we build the outline
/// directly at oversampled resolution. Less memory, fewer operations,
/// better performance—the holy trinity of optimization.
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
    downsample_to_grayscale(&mono_bitmap, oversample_width, oversample_height, width, height, level)
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
        assert!(gray[5 * 10 + 5] > 200, "Center alpha = {}", gray[5 * 10 + 5]);

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
