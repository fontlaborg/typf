// this_file: crates/typf-render/src/output.rs

//! Image output generation (PGM and PNG formats).
//!
//! This module generates PGM (grayscale) and PNG images from rendered pixel data,
//! with base64 encoding for JSONL output. Ported from haforu for compatibility.

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use image::{ImageBuffer, Luma};
use std::io::Write;
use thiserror::Error;

/// Errors that can occur during image output generation.
#[derive(Debug, Error)]
pub enum OutputError {
    #[error("Pixel data size mismatch: expected {expected} bytes, got {actual}")]
    SizeMismatch { expected: usize, actual: usize },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image encoding error: {0}")]
    ImageEncode(#[from] image::ImageError),

    #[error("Invalid PGM format: {0}")]
    InvalidPgm(String),

    #[error("Base64 decode error: {0}")]
    Base64Decode(String),
}

pub type Result<T> = std::result::Result<T, OutputError>;

/// Image output format handler.
pub struct ImageOutput;

impl ImageOutput {
    /// Generate PGM P5 (binary) format from grayscale pixels.
    ///
    /// PGM format:
    /// ```text
    /// P5
    /// <width> <height>
    /// 255
    /// <binary pixel data>
    /// ```
    ///
    /// # Arguments
    /// * `pixels` - Grayscale pixel data (one byte per pixel, row-major order)
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    ///
    /// # Returns
    /// Binary PGM data ready to write to file or encode as base64
    pub fn write_pgm_binary(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
        let expected = (width * height) as usize;
        if pixels.len() != expected {
            return Err(OutputError::SizeMismatch {
                expected,
                actual: pixels.len(),
            });
        }

        let mut output = Vec::new();

        // Write PGM header
        writeln!(&mut output, "P5")?;
        writeln!(&mut output, "{} {}", width, height)?;
        writeln!(&mut output, "255")?;

        // Write binary pixel data
        output.extend_from_slice(pixels);

        Ok(output)
    }

    /// Generate PNG format from grayscale pixels.
    ///
    /// # Arguments
    /// * `pixels` - Grayscale pixel data (one byte per pixel, row-major order)
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    ///
    /// # Returns
    /// Binary PNG data ready to write to file or encode as base64
    pub fn write_png(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
        let expected = (width * height) as usize;
        if pixels.len() != expected {
            return Err(OutputError::SizeMismatch {
                expected,
                actual: pixels.len(),
            });
        }

        // Create image buffer
        let img: ImageBuffer<Luma<u8>, Vec<u8>> =
            ImageBuffer::from_raw(width, height, pixels.to_vec()).ok_or_else(|| {
                OutputError::ImageEncode(image::ImageError::Parameter(
                    image::error::ParameterError::from_kind(
                        image::error::ParameterErrorKind::DimensionMismatch,
                    ),
                ))
            })?;

        // Encode as PNG
        let mut output = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut output),
            image::ImageFormat::Png,
        )?;

        Ok(output)
    }

    /// Base64-encode image data for JSONL output.
    ///
    /// # Arguments
    /// * `data` - Binary image data (PGM or PNG)
    ///
    /// # Returns
    /// Base64-encoded string ready for JSON embedding
    pub fn encode_base64(data: &[u8]) -> String {
        BASE64.encode(data)
    }

    /// Decode base64-encoded image data (for testing).
    #[cfg(test)]
    pub fn decode_base64(encoded: &str) -> Result<Vec<u8>> {
        BASE64
            .decode(encoded)
            .map_err(|e| OutputError::Base64Decode(e.to_string()))
    }

    /// Decode PGM P5 format (for testing).
    #[cfg(test)]
    pub fn decode_pgm(data: &[u8]) -> Result<(Vec<u8>, u32, u32)> {
        use std::io::{BufRead, BufReader, Read};

        let mut reader = BufReader::new(data);
        let mut line = String::new();

        // Read "P5"
        reader.read_line(&mut line)?;
        if line.trim() != "P5" {
            return Err(OutputError::InvalidPgm(format!(
                "expected 'P5', got '{}'",
                line.trim()
            )));
        }

        // Read width and height
        line.clear();
        reader.read_line(&mut line)?;
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.len() != 2 {
            return Err(OutputError::InvalidPgm("invalid dimensions".to_string()));
        }
        let width: u32 = parts[0]
            .parse()
            .map_err(|_| OutputError::InvalidPgm(format!("invalid width: {}", parts[0])))?;
        let height: u32 = parts[1]
            .parse()
            .map_err(|_| OutputError::InvalidPgm(format!("invalid height: {}", parts[1])))?;

        // Read maxval (should be 255)
        line.clear();
        reader.read_line(&mut line)?;
        let maxval: u32 = line
            .trim()
            .parse()
            .map_err(|_| OutputError::InvalidPgm(format!("invalid maxval: {}", line.trim())))?;
        if maxval != 255 {
            return Err(OutputError::InvalidPgm(format!(
                "unsupported maxval: {} (expected 255)",
                maxval
            )));
        }

        // Read binary pixel data
        let mut pixels = Vec::new();
        reader.read_to_end(&mut pixels)?;

        let expected = (width * height) as usize;
        if pixels.len() != expected {
            return Err(OutputError::SizeMismatch {
                expected,
                actual: pixels.len(),
            });
        }

        Ok((pixels, width, height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_pgm_binary() {
        let pixels = vec![0u8, 128, 255, 64];
        let pgm = ImageOutput::write_pgm_binary(&pixels, 2, 2).unwrap();

        // Check header (P5\n2 2\n255\n = 11 bytes)
        let header = String::from_utf8_lossy(&pgm[..11]);
        assert!(header.starts_with("P5"));
        assert!(header.contains("2 2"));
        assert!(header.contains("255"));

        // Check pixel data starts at byte 11
        assert_eq!(&pgm[11..], &pixels);
    }

    #[test]
    fn test_pgm_round_trip() {
        let original_pixels = vec![0u8, 50, 100, 150, 200, 255];
        let pgm = ImageOutput::write_pgm_binary(&original_pixels, 3, 2).unwrap();

        let (decoded_pixels, width, height) = ImageOutput::decode_pgm(&pgm).unwrap();
        assert_eq!(width, 3);
        assert_eq!(height, 2);
        assert_eq!(decoded_pixels, original_pixels);
    }

    #[test]
    fn test_base64_round_trip() {
        let data = b"Hello, TYPF!";
        let encoded = ImageOutput::encode_base64(data);
        let decoded = ImageOutput::decode_base64(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_write_png() {
        let pixels = vec![0u8; 100 * 50]; // 100×50 black image
        let png = ImageOutput::write_png(&pixels, 100, 50).unwrap();

        // Check PNG signature
        assert_eq!(&png[0..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn test_write_pgm_size_mismatch() {
        let pixels = vec![0u8; 10];
        let result = ImageOutput::write_pgm_binary(&pixels, 100, 50);
        assert!(result.is_err());
        match result.unwrap_err() {
            OutputError::SizeMismatch { expected, actual } => {
                assert_eq!(expected, 5000);
                assert_eq!(actual, 10);
            }
            _ => panic!("Expected SizeMismatch error"),
        }
    }

    #[test]
    fn test_write_png_size_mismatch() {
        let pixels = vec![0u8; 10];
        let result = ImageOutput::write_png(&pixels, 100, 50);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_pgm_format() {
        let invalid_pgm = b"P6\n2 2\n255\n\x00\x00\x00\x00"; // P6 instead of P5
        let result = ImageOutput::decode_pgm(invalid_pgm);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OutputError::InvalidPgm(_)));
    }

    #[test]
    fn test_decode_pgm_wrong_maxval() {
        let invalid_pgm = b"P5\n2 2\n65535\n\x00\x00\x00\x00"; // 16-bit maxval
        let result = ImageOutput::decode_pgm(invalid_pgm);
        assert!(result.is_err());
    }

    #[test]
    fn test_pgm_realistic_gradient() {
        // Create 10x10 gradient
        let mut pixels = Vec::with_capacity(100);
        for y in 0..10 {
            for x in 0..10 {
                pixels.push((x * 25 + y * 2) as u8);
            }
        }

        let pgm = ImageOutput::write_pgm_binary(&pixels, 10, 10).unwrap();
        let (decoded, w, h) = ImageOutput::decode_pgm(&pgm).unwrap();

        assert_eq!(w, 10);
        assert_eq!(h, 10);
        assert_eq!(decoded, pixels);
    }
}
