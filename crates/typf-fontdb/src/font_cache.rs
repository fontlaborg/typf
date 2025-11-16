// this_file: crates/typf-fontdb/src/font_cache.rs

//! Font loading, variation handling, and caching (ported from haforu).
//!
//! This module provides zero-copy font loading via memory mapping,
//! variable font coordinate application, and simple cache eviction.

use camino::Utf8Path;
use dashmap::DashMap;
use harfbuzz_rs::{Face as HbFace, Owned};
use memmap2::Mmap;
use read_fonts::{types::Tag, FileRef, FontRef};
use skrifa::MetadataProvider;
use std::collections::HashMap;
use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Maximum font file size (50MB) to prevent resource exhaustion.
pub const MAX_FONT_SIZE: u64 = 50 * 1024 * 1024;

/// Memory-mapped font loader with instance cache.
///
/// Provides deterministic font loading via memory mapping for performance,
/// with support for variable font coordinates and HarfBuzz integration.
pub struct FontLoader {
    cache: Arc<DashMap<FontCacheKey, Arc<FontInstance>>>,
    max_capacity: usize,
    current_size: Arc<AtomicUsize>,
}

/// Font cache statistics for observability.
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    /// Maximum number of cached font instances.
    pub capacity: usize,
    /// Currently cached font instances.
    pub entries: usize,
}

/// Font instance with applied variation coordinates.
///
/// Holds memory-mapped font data, parsed font reference, and cached
/// HarfBuzz font object for efficient repeated shaping.
pub struct FontInstance {
    /// Memory-mapped font data (kept alive for 'static references)
    #[allow(dead_code)]
    mmap: Arc<Mmap>,
    /// Font reference (zero-copy view into mmap)
    font_ref: FontRef<'static>,
    /// Applied variation coordinates
    coordinates: HashMap<String, f32>,
    /// Cached HarfBuzz font with variations pre-applied
    hb_font: Arc<Mutex<Owned<harfbuzz_rs::Font<'static>>>>,
}

/// Cache key for font instances.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct FontCacheKey {
    path: String,
    coordinates: Vec<(String, u32)>, // (axis, f32::to_bits)
}

impl FontLoader {
    /// Create a new font loader with specified cache size.
    ///
    /// # Arguments
    /// * `cache_size` - Maximum number of font instances to cache (minimum 1)
    ///
    /// # Examples
    /// ```ignore
    /// use typf_fontdb::font_cache::FontLoader;
    ///
    /// let loader = FontLoader::new(512);
    /// ```
    pub fn new(cache_size: usize) -> Self {
        let cache_size = cache_size.max(1);
        Self {
            cache: Arc::new(DashMap::with_capacity(cache_size)),
            max_capacity: cache_size,
            current_size: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Load a font and apply variable font coordinates.
    ///
    /// Returns a cached instance if available (fast path), otherwise loads
    /// from disk via memory mapping (slow path).
    ///
    /// # Arguments
    /// * `path` - Font file path (UTF-8 encoded)
    /// * `coordinates` - Variable font axis coordinates (e.g., `{"wght": 700.0}`)
    ///
    /// # Errors
    /// Returns error if:
    /// - File not found or unreadable
    /// - File exceeds MAX_FONT_SIZE
    /// - Memory mapping fails
    /// - Font data is invalid or corrupted
    ///
    /// # Examples
    /// ```ignore
    /// use camino::Utf8Path;
    /// use std::collections::HashMap;
    ///
    /// let loader = FontLoader::new(512);
    /// let path = Utf8Path::new("fonts/Roboto-Regular.ttf");
    /// let coords = HashMap::new();
    /// let font = loader.load_font(path, &coords)?;
    /// ```
    pub fn load_font(
        &self,
        path: &Utf8Path,
        coordinates: &HashMap<String, f32>,
    ) -> Result<Arc<FontInstance>, FontCacheError> {
        // Build cache key
        let cache_key = FontCacheKey {
            path: path.to_string(),
            coordinates: coordinates
                .iter()
                .map(|(k, v)| (k.clone(), v.to_bits()))
                .collect(),
        };

        // Fast path: check cache with lock-free read
        if let Some(instance) = self.cache.get(&cache_key) {
            return Ok(Arc::clone(instance.value()));
        }

        // Slow path: load from disk
        let instance = Self::load_font_impl(path, coordinates)?;
        let instance = Arc::new(instance);

        // Store in cache with simple size-based eviction
        let current = self.current_size.fetch_add(1, Ordering::Relaxed);
        if current >= self.max_capacity {
            // Cache full - evict first entry (simple FIFO-like eviction)
            // Note: DashMap doesn't have LRU built-in, this is a pragmatic trade-off
            if let Some(first_key) = self.cache.iter().next().map(|e| e.key().clone()) {
                self.cache.remove(&first_key);
                self.current_size.fetch_sub(1, Ordering::Relaxed);
            }
        }

        self.cache.insert(cache_key, Arc::clone(&instance));
        Ok(instance)
    }

    /// Clear all cached font instances.
    ///
    /// Useful for freeing memory or when fonts on disk may have changed.
    pub fn clear(&self) {
        self.cache.clear();
        self.current_size.store(0, Ordering::Relaxed);
    }

    /// Return current cache statistics.
    ///
    /// # Examples
    /// ```ignore
    /// let stats = loader.stats();
    /// println!("Cache: {}/{} entries", stats.entries, stats.capacity);
    /// ```
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            capacity: self.max_capacity,
            entries: self.cache.len(),
        }
    }

    /// Internal implementation: load font from disk and apply variations.
    fn load_font_impl(
        path: &Utf8Path,
        coordinates: &HashMap<String, f32>,
    ) -> Result<FontInstance, FontCacheError> {
        // Memory-map the font file
        let std_path = path.as_std_path();
        let file = File::open(std_path).map_err(|e| match e.kind() {
            ErrorKind::NotFound => FontCacheError::FontNotFound {
                path: std_path.to_path_buf(),
            },
            _ => FontCacheError::Mmap {
                path: std_path.to_path_buf(),
                source: e,
            },
        })?;

        // Validate file size against limit
        let meta = file.metadata().map_err(|e| FontCacheError::Mmap {
            path: std_path.to_path_buf(),
            source: e,
        })?;

        if meta.len() > MAX_FONT_SIZE {
            return Err(FontCacheError::FontTooLarge {
                path: std_path.to_path_buf(),
                size: meta.len(),
                max: MAX_FONT_SIZE,
            });
        }

        let mmap = unsafe {
            Mmap::map(&file).map_err(|e| FontCacheError::Mmap {
                path: std_path.to_path_buf(),
                source: e,
            })?
        };

        let mmap = Arc::new(mmap);

        // Parse font with 'static lifetime (safe because mmap is Arc'd)
        let font_data: &'static [u8] =
            unsafe { std::slice::from_raw_parts(mmap.as_ptr(), mmap.len()) };

        let file_ref = FileRef::new(font_data).map_err(|e| FontCacheError::InvalidFont {
            path: path.as_std_path().to_path_buf(),
            reason: format!("Failed to parse font file: {}", e),
        })?;

        let font_ref = match file_ref {
            FileRef::Font(f) => f,
            FileRef::Collection(c) => c.get(0).map_err(|e| FontCacheError::InvalidFont {
                path: path.as_std_path().to_path_buf(),
                reason: format!("Failed to get font from collection: {}", e),
            })?,
        };

        // Validate and clamp variation coordinates
        let clamped_coords = if !coordinates.is_empty() {
            Self::validate_and_clamp_coordinates(&font_ref, path.as_std_path(), coordinates)?
        } else {
            coordinates.clone()
        };

        // Create HarfBuzz font with variations pre-applied
        let hb_font = Self::create_harfbuzz_font(&mmap, &clamped_coords)?;

        Ok(FontInstance {
            mmap,
            font_ref,
            coordinates: clamped_coords,
            hb_font: Arc::new(Mutex::new(hb_font)),
        })
    }

    /// Validate variation axes and clamp coordinates to font-defined bounds.
    ///
    /// For well-known axes (wght, wdth), applies additional hard limits as a safeguard.
    /// Warns and drops coordinates for axes not present in the font.
    fn validate_and_clamp_coordinates(
        font: &FontRef,
        path: &Path,
        coordinates: &HashMap<String, f32>,
    ) -> Result<HashMap<String, f32>, FontCacheError> {
        // Extract available axes from font
        let axes: HashMap<String, (f32, f32, f32)> = font
            .axes()
            .iter()
            .map(|axis| {
                let tag = axis.tag().to_string();
                (
                    tag,
                    (axis.min_value(), axis.default_value(), axis.max_value()),
                )
            })
            .collect();

        if axes.is_empty() {
            // Static font - ignore all coordinates
            if !coordinates.is_empty() {
                log::warn!(
                    "Font {} is static but coordinates provided - ignoring",
                    path.display()
                );
            }
            return Ok(HashMap::new());
        }

        // Validate and clamp each coordinate
        let mut clamped = HashMap::new();
        for (axis, value) in coordinates {
            if let Some((min, _default, max)) = axes.get(axis) {
                // Apply well-known sane clamps for common axes
                let (hard_min, hard_max) = match axis.as_str() {
                    "wght" => (100.0_f32, 900.0_f32),
                    "wdth" => (50.0_f32, 200.0_f32),
                    _ => (*min, *max),
                };

                // Combine clamps conservatively (intersection of bounds)
                let eff_min = hard_min.max(*min);
                let eff_max = hard_max.min(*max);
                let clamped_value = value.clamp(eff_min, eff_max);

                if (clamped_value - value).abs() > 0.001 {
                    log::warn!(
                        "Coordinate for axis '{}' clamped from {} to {} (font bounds: [{}, {}], hard limits: [{}, {}])",
                        axis,
                        value,
                        clamped_value,
                        min,
                        max,
                        hard_min,
                        hard_max
                    );
                }
                clamped.insert(axis.clone(), clamped_value);
            } else {
                // Axis not present in font - warn and drop
                log::warn!(
                    "Unknown variation axis '{}' for font {} — dropping coordinate",
                    axis,
                    path.display()
                );
            }
        }

        Ok(clamped)
    }

    /// Create a HarfBuzz font from memory-mapped data with variations applied.
    fn create_harfbuzz_font(
        mmap: &Arc<Mmap>,
        coordinates: &HashMap<String, f32>,
    ) -> Result<Owned<harfbuzz_rs::Font<'static>>, FontCacheError> {
        // Convert mmap bytes to 'static lifetime (safe because mmap is Arc'd)
        let font_data: &'static [u8] =
            unsafe { std::slice::from_raw_parts(mmap.as_ptr(), mmap.len()) };

        // Create HarfBuzz face and font
        let face = HbFace::from_bytes(font_data, 0);
        let mut hb_font = harfbuzz_rs::Font::new(face);

        // Apply variation coordinates if present
        if !coordinates.is_empty() {
            let variations: Vec<harfbuzz_rs::Variation> = coordinates
                .iter()
                .filter_map(|(tag, value)| {
                    let chars: Vec<char> = tag.chars().collect();
                    if chars.len() == 4 {
                        Some(harfbuzz_rs::Variation::new(
                            harfbuzz_rs::Tag::new(chars[0], chars[1], chars[2], chars[3]),
                            *value,
                        ))
                    } else {
                        log::warn!("Invalid variation tag '{}' - skipping", tag);
                        None
                    }
                })
                .collect();
            hb_font.set_variations(&variations);
        }

        Ok(hb_font)
    }

    /// Get current cache usage (entries, capacity).
    ///
    /// # Returns
    /// Tuple of (current_entries, max_capacity)
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.max_capacity)
    }
}

impl FontInstance {
    /// Get the font reference (zero-copy view into memory-mapped data).
    ///
    /// Use this for skrifa operations (outline extraction, metrics).
    pub fn font_ref(&self) -> &FontRef<'static> {
        &self.font_ref
    }

    /// Get the applied variation coordinates.
    ///
    /// Returns clamped coordinates (may differ from requested).
    pub fn coordinates(&self) -> &HashMap<String, f32> {
        &self.coordinates
    }

    /// Get the raw font data bytes.
    ///
    /// Useful for passing to other font libraries or caching.
    pub fn font_data(&self) -> &[u8] {
        self.mmap.as_ref()
    }

    /// Create a skrifa Location for rendering with variation coordinates.
    ///
    /// Converts string-based coordinates to skrifa's Tag-based format.
    ///
    /// # Examples
    /// ```ignore
    /// let location = font_instance.location();
    /// // Use with skrifa: instance.with_location(&location)
    /// ```
    pub fn location(&self) -> Vec<(Tag, f32)> {
        self.coordinates
            .iter()
            .filter_map(|(tag_str, value)| {
                Tag::new_checked(tag_str.as_bytes())
                    .ok()
                    .map(|tag| (tag, *value))
            })
            .collect()
    }

    /// Get reference to the cached HarfBuzz font.
    ///
    /// Variations are pre-applied for performance. Use for text shaping.
    ///
    /// # Examples
    /// ```ignore
    /// let hb_font = font_instance.hb_font();
    /// let mut hb = hb_font.lock().unwrap();
    /// // Use for shaping with harfbuzz_rs
    /// ```
    pub fn hb_font(&self) -> &Arc<Mutex<Owned<harfbuzz_rs::Font<'static>>>> {
        &self.hb_font
    }
}

/// Errors that can occur during font loading and caching.
#[derive(Debug, thiserror::Error)]
pub enum FontCacheError {
    /// Font file not found at specified path.
    #[error("Font not found: {}", path.display())]
    FontNotFound {
        /// Path to the missing font file.
        path: std::path::PathBuf
    },

    /// Font file exceeds maximum size limit.
    #[error("Font file too large: {} bytes (max: {} bytes) at {}", size, max, path.display())]
    FontTooLarge {
        /// Path to the oversized font file.
        path: std::path::PathBuf,
        /// Actual size of the font file in bytes.
        size: u64,
        /// Maximum allowed size in bytes.
        max: u64,
    },

    /// Memory mapping failed.
    #[error("Failed to memory-map font at {}: {}", path.display(), source)]
    Mmap {
        /// Path to the font file that failed to mmap.
        path: std::path::PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// Font data is invalid or corrupted.
    #[error("Invalid font at {}: {}", path.display(), reason)]
    InvalidFont {
        /// Path to the invalid font file.
        path: std::path::PathBuf,
        /// Reason why the font is invalid.
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_loader_new() {
        let loader = FontLoader::new(512);
        let stats = loader.stats();
        assert_eq!(stats.capacity, 512);
        assert_eq!(stats.entries, 0);
    }

    #[test]
    fn test_font_loader_minimum_capacity() {
        let loader = FontLoader::new(0);
        let stats = loader.stats();
        assert_eq!(stats.capacity, 1); // minimum enforced
    }

    #[test]
    fn test_cache_clear() {
        let loader = FontLoader::new(512);
        loader.clear();
        let stats = loader.stats();
        assert_eq!(stats.entries, 0);
    }

    // Note: Integration tests with actual fonts should be in tests/ directory
    // These tests verify the API without requiring font files
}
