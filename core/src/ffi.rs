//! C-ABI compatible types for FFI consumers.
//!
// FFI code requires unsafe by nature - all unsafe usage is documented
#![allow(unsafe_code)]
//!
//! This module provides `#[repr(C)]` types that can be safely passed across FFI
//! boundaries. These types mirror the internal Rust types but use C-compatible
//! layouts for integration with C, Python (via ctypes/cffi), and other languages.
//!
//! # Memory Ownership
//!
//! - Types ending in `C` are C-ABI compatible copies
//! - Slice views (`*const T`, `len`) are borrowed - caller must not free
//! - Owned arrays should be freed via the corresponding `_free()` function
//!
//! # Example (conceptual C usage)
//!
//! ```c
//! TypfShapingResultC result;
//! int err = typf_shape_text("hb", "Hello", "/path/font.ttf", 24.0, &result);
//! if (err == 0) {
//!     for (uint32_t i = 0; i < result.glyph_count; i++) {
//!         printf("Glyph %u at (%.1f, %.1f)\n",
//!                result.glyphs[i].glyph_id,
//!                result.glyphs[i].x,
//!                result.glyphs[i].y);
//!     }
//!     typf_shaping_result_free(&result);
//! }
//! ```

// this_file: crates/typf-core/src/ffi.rs

use crate::types::{Direction, PositionedGlyph, ShapingResult};

/// C-ABI compatible positioned glyph.
///
/// This struct matches the layout expected by external renderers like Cairo:
/// - `glyph_id`: Index into the font's glyph table
/// - `x`, `y`: Position in user space (typically pixels)
/// - `advance`: Horizontal advance width
/// - `cluster`: Cluster index for text segmentation (useful for cursor positioning)
///
/// # Size and Alignment
///
/// This struct is 20 bytes with 4-byte alignment on all platforms.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionedGlyphC {
    /// Glyph index in the font (maps to `cairo_glyph_t.index`)
    pub glyph_id: u32,
    /// Horizontal position in user space
    pub x: f32,
    /// Vertical position in user space
    pub y: f32,
    /// Horizontal advance width
    pub advance: f32,
    /// Cluster index (maps to original text position)
    pub cluster: u32,
}

impl From<&PositionedGlyph> for PositionedGlyphC {
    fn from(g: &PositionedGlyph) -> Self {
        Self {
            glyph_id: g.id,
            x: g.x,
            y: g.y,
            advance: g.advance,
            cluster: g.cluster,
        }
    }
}

impl From<PositionedGlyph> for PositionedGlyphC {
    fn from(g: PositionedGlyph) -> Self {
        Self::from(&g)
    }
}

impl From<&PositionedGlyphC> for PositionedGlyph {
    fn from(g: &PositionedGlyphC) -> Self {
        Self {
            id: g.glyph_id,
            x: g.x,
            y: g.y,
            advance: g.advance,
            cluster: g.cluster,
        }
    }
}

/// Text direction as a C-compatible enum.
///
/// Values match common conventions:
/// - 0: Left-to-right (Latin, Cyrillic)
/// - 1: Right-to-left (Arabic, Hebrew)
/// - 2: Top-to-bottom (Traditional Chinese, Japanese)
/// - 3: Bottom-to-top (rare)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionC {
    LeftToRight = 0,
    RightToLeft = 1,
    TopToBottom = 2,
    BottomToTop = 3,
}

impl From<Direction> for DirectionC {
    fn from(d: Direction) -> Self {
        match d {
            Direction::LeftToRight => DirectionC::LeftToRight,
            Direction::RightToLeft => DirectionC::RightToLeft,
            Direction::TopToBottom => DirectionC::TopToBottom,
            Direction::BottomToTop => DirectionC::BottomToTop,
        }
    }
}

impl From<DirectionC> for Direction {
    fn from(d: DirectionC) -> Self {
        match d {
            DirectionC::LeftToRight => Direction::LeftToRight,
            DirectionC::RightToLeft => Direction::RightToLeft,
            DirectionC::TopToBottom => Direction::TopToBottom,
            DirectionC::BottomToTop => Direction::BottomToTop,
        }
    }
}

/// C-ABI compatible shaping result.
///
/// Contains a pointer to an array of positioned glyphs plus metadata.
///
/// # Memory Ownership
///
/// When returned from FFI functions, the caller owns the memory and must call
/// `typf_shaping_result_free()` to release it. The `glyphs` pointer is valid
/// until freed.
///
/// # Null Safety
///
/// - `glyphs` may be null if `glyph_count` is 0
/// - Always check `glyph_count` before dereferencing `glyphs`
#[repr(C)]
#[derive(Debug)]
pub struct ShapingResultC {
    /// Pointer to array of positioned glyphs (owned)
    pub glyphs: *mut PositionedGlyphC,
    /// Number of glyphs in the array
    pub glyph_count: u32,
    /// Total horizontal advance width
    pub advance_width: f32,
    /// Total vertical advance height
    pub advance_height: f32,
    /// Text direction
    pub direction: DirectionC,
    /// Reserved for future use (padding)
    pub _reserved: [u8; 3],
}

impl ShapingResultC {
    /// Creates a new ShapingResultC by converting from a Rust ShapingResult.
    ///
    /// This allocates a new array for the glyphs. The caller is responsible
    /// for freeing it via `free()`.
    pub fn from_rust(result: &ShapingResult) -> Self {
        let glyph_count = result.glyphs.len() as u32;
        let glyphs = if glyph_count > 0 {
            let mut vec: Vec<PositionedGlyphC> =
                result.glyphs.iter().map(PositionedGlyphC::from).collect();
            let ptr = vec.as_mut_ptr();
            std::mem::forget(vec); // Transfer ownership to C
            ptr
        } else {
            std::ptr::null_mut()
        };

        Self {
            glyphs,
            glyph_count,
            advance_width: result.advance_width,
            advance_height: result.advance_height,
            direction: result.direction.into(),
            _reserved: [0; 3],
        }
    }

    /// Frees the memory allocated for glyphs.
    ///
    /// # Safety
    ///
    /// - Must only be called once per ShapingResultC
    /// - The glyphs pointer must have been allocated by `from_rust()`
    pub unsafe fn free(&mut self) {
        if !self.glyphs.is_null() && self.glyph_count > 0 {
            let _ = Vec::from_raw_parts(
                self.glyphs,
                self.glyph_count as usize,
                self.glyph_count as usize,
            );
            self.glyphs = std::ptr::null_mut();
            self.glyph_count = 0;
        }
    }

    /// Returns a slice view of the glyphs.
    ///
    /// # Safety
    ///
    /// The returned slice is valid only while this ShapingResultC is valid
    /// and has not been freed.
    pub unsafe fn glyphs_slice(&self) -> &[PositionedGlyphC] {
        if self.glyphs.is_null() || self.glyph_count == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(self.glyphs, self.glyph_count as usize)
        }
    }

    /// Converts back to a Rust ShapingResult.
    ///
    /// This creates a new owned Vec, leaving this ShapingResultC unchanged.
    ///
    /// # Safety
    ///
    /// The glyphs pointer must be valid.
    pub unsafe fn to_rust(&self) -> ShapingResult {
        let glyphs: Vec<PositionedGlyph> = self
            .glyphs_slice()
            .iter()
            .map(PositionedGlyph::from)
            .collect();

        ShapingResult {
            glyphs,
            advance_width: self.advance_width,
            advance_height: self.advance_height,
            direction: self.direction.into(),
        }
    }
}

/// Glyph iterator for zero-copy consumption.
///
/// This provides a way for layout engines to iterate over shaped glyphs
/// without taking ownership of the underlying data.
///
/// # Example
///
/// ```rust
/// use typf_core::ffi::GlyphIterator;
/// use typf_core::types::ShapingResult;
///
/// fn process_glyphs(result: &ShapingResult) {
///     let iter = GlyphIterator::new(result);
///     for glyph in iter {
///         println!("Glyph {} at ({}, {})", glyph.glyph_id, glyph.x, glyph.y);
///     }
/// }
/// ```
pub struct GlyphIterator<'a> {
    glyphs: &'a [PositionedGlyph],
    index: usize,
}

impl<'a> GlyphIterator<'a> {
    /// Creates a new iterator over the glyphs in a shaping result.
    pub fn new(result: &'a ShapingResult) -> Self {
        Self {
            glyphs: &result.glyphs,
            index: 0,
        }
    }

    /// Returns the total number of glyphs.
    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    /// Returns true if there are no glyphs.
    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }

    /// Returns the remaining number of glyphs to iterate.
    pub fn remaining(&self) -> usize {
        self.glyphs.len().saturating_sub(self.index)
    }
}

impl<'a> Iterator for GlyphIterator<'a> {
    type Item = PositionedGlyphC;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.glyphs.len() {
            let glyph = PositionedGlyphC::from(&self.glyphs[self.index]);
            self.index += 1;
            Some(glyph)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.remaining();
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for GlyphIterator<'a> {}

// =============================================================================
// Stage 5: Mesh ABI for GPU Upload
// =============================================================================

/// Vertex for GPU rendering (2D position only).
///
/// This is the minimal vertex format for text rendering. Position is in
/// normalized device coordinates or pixel space depending on your pipeline.
///
/// Compatible with wgpu `VertexFormat::Float32x2` at offset 0.
///
/// # Memory Layout
///
/// - Size: 8 bytes
/// - Alignment: 4 bytes
/// - Can be cast to `&[u8]` via `bytemuck::cast_slice()` or `as_bytes()`
///
/// # Example (wgpu)
///
/// ```rust,ignore
/// wgpu::VertexBufferLayout {
///     array_stride: 8,
///     step_mode: wgpu::VertexStepMode::Vertex,
///     attributes: &[wgpu::VertexAttribute {
///         offset: 0,
///         shader_location: 0,
///         format: wgpu::VertexFormat::Float32x2,
///     }],
/// }
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vertex2D {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
}

impl Vertex2D {
    /// Creates a new 2D vertex.
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns the vertex data as a byte slice.
    ///
    /// # Safety
    ///
    /// The returned slice is valid for the lifetime of self.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: Vertex2D is repr(C) with only f32 fields, no padding
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, 8) }
    }
}

/// Vertex with position and UV coordinates for textured rendering.
///
/// Useful for rendering bitmap glyphs or SDF textures. UV coordinates
/// are typically in [0, 1] range for texture sampling.
///
/// Compatible with wgpu `Float32x2` at offset 0 (position) and offset 8 (uv).
///
/// # Memory Layout
///
/// - Size: 16 bytes
/// - Alignment: 4 bytes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct VertexUV {
    /// X coordinate (position)
    pub x: f32,
    /// Y coordinate (position)
    pub y: f32,
    /// U coordinate (texture)
    pub u: f32,
    /// V coordinate (texture)
    pub v: f32,
}

impl VertexUV {
    /// Creates a new vertex with position and UV.
    #[inline]
    pub const fn new(x: f32, y: f32, u: f32, v: f32) -> Self {
        Self { x, y, u, v }
    }

    /// Returns the vertex data as a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: VertexUV is repr(C) with only f32 fields, no padding
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, 16) }
    }
}

/// Vertex with position and RGBA color.
///
/// Useful for colored glyph rendering (COLR glyphs, colored text).
/// Color values are in [0, 1] range.
///
/// # Memory Layout
///
/// - Size: 24 bytes
/// - Alignment: 4 bytes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct VertexColor {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    /// Red component [0, 1]
    pub r: f32,
    /// Green component [0, 1]
    pub g: f32,
    /// Blue component [0, 1]
    pub b: f32,
    /// Alpha component [0, 1]
    pub a: f32,
}

impl VertexColor {
    /// Creates a new vertex with position and color.
    #[inline]
    pub const fn new(x: f32, y: f32, r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { x, y, r, g, b, a }
    }

    /// Returns the vertex data as a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: VertexColor is repr(C) with only f32 fields, no padding
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, 24) }
    }
}

/// A mesh of triangles for GPU rendering.
///
/// Contains vertices and indices suitable for indexed drawing.
/// Indices reference vertices by their position in the `vertices` array.
///
/// # Zero-Copy Upload
///
/// Use `vertices_bytes()` and `indices_bytes()` to get byte slices
/// that can be directly uploaded to GPU buffers without copying:
///
/// ```rust,ignore
/// let mesh: GlyphMesh<Vertex2D> = /* ... */;
/// let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
///     label: Some("Glyph Vertices"),
///     contents: mesh.vertices_bytes(),
///     usage: wgpu::BufferUsages::VERTEX,
/// });
/// let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
///     label: Some("Glyph Indices"),
///     contents: mesh.indices_bytes(),
///     usage: wgpu::BufferUsages::INDEX,
/// });
/// ```
#[derive(Debug, Clone)]
pub struct GlyphMesh<V> {
    /// Vertex data (position, optionally UV/color)
    pub vertices: Vec<V>,
    /// Triangle indices (3 indices per triangle)
    pub indices: Vec<u32>,
    /// Glyph ID this mesh represents
    pub glyph_id: u32,
}

impl<V> GlyphMesh<V> {
    /// Creates a new empty mesh for a glyph.
    pub fn new(glyph_id: u32) -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            glyph_id,
        }
    }

    /// Creates a mesh with pre-allocated capacity.
    pub fn with_capacity(glyph_id: u32, vertex_capacity: usize, index_capacity: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_capacity),
            indices: Vec::with_capacity(index_capacity),
            glyph_id,
        }
    }

    /// Returns the number of triangles in this mesh.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Returns true if the mesh has no geometry.
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() || self.indices.is_empty()
    }
}

impl<V: Copy> GlyphMesh<V> {
    /// Returns the vertex data as a byte slice for GPU upload.
    ///
    /// # Safety
    ///
    /// This assumes V is a repr(C) struct with no padding.
    /// Use only with Vertex2D, VertexUV, or VertexColor.
    pub fn vertices_bytes(&self) -> &[u8] {
        if self.vertices.is_empty() {
            return &[];
        }
        let ptr = self.vertices.as_ptr() as *const u8;
        let len = self.vertices.len() * std::mem::size_of::<V>();
        // SAFETY: vertices is a contiguous Vec, V is repr(C)
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

impl<V> GlyphMesh<V> {
    /// Returns the index data as a byte slice for GPU upload.
    pub fn indices_bytes(&self) -> &[u8] {
        if self.indices.is_empty() {
            return &[];
        }
        let ptr = self.indices.as_ptr() as *const u8;
        let len = self.indices.len() * 4; // u32 = 4 bytes
                                          // SAFETY: indices is a contiguous Vec of u32
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }
}

/// A collection of glyph meshes ready for GPU rendering.
///
/// This is the Stage 5 output format for GPU pipelines. Each glyph
/// has its own mesh with position offset applied.
///
/// # Memory Layout
///
/// All vertex types are `#[repr(C)]` with documented sizes:
/// - `Vertex2D`: 8 bytes (2 × f32)
/// - `VertexUV`: 16 bytes (4 × f32)
/// - `VertexColor`: 24 bytes (6 × f32)
/// - Indices: 4 bytes each (u32)
///
/// # Merging Meshes
///
/// For batch rendering, use `merge_all()` to combine glyphs into a single mesh:
///
/// ```rust,ignore
/// let render_mesh = RenderMesh::<Vertex2D>::new();
/// // ... add glyph meshes ...
/// let (vertices, indices) = render_mesh.merge_all();
/// ```
#[derive(Debug, Clone)]
pub struct RenderMesh<V> {
    /// Individual glyph meshes
    pub glyphs: Vec<GlyphMesh<V>>,
    /// Total vertex count across all glyphs
    pub total_vertices: usize,
    /// Total index count across all glyphs
    pub total_indices: usize,
}

impl<V> Default for RenderMesh<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> RenderMesh<V> {
    /// Creates a new empty render mesh.
    pub fn new() -> Self {
        Self {
            glyphs: Vec::new(),
            total_vertices: 0,
            total_indices: 0,
        }
    }

    /// Creates a render mesh with pre-allocated capacity.
    pub fn with_capacity(glyph_count: usize) -> Self {
        Self {
            glyphs: Vec::with_capacity(glyph_count),
            total_vertices: 0,
            total_indices: 0,
        }
    }

    /// Adds a glyph mesh to the render mesh.
    pub fn push(&mut self, mesh: GlyphMesh<V>) {
        self.total_vertices += mesh.vertices.len();
        self.total_indices += mesh.indices.len();
        self.glyphs.push(mesh);
    }

    /// Returns the number of glyphs in this mesh.
    pub fn glyph_count(&self) -> usize {
        self.glyphs.len()
    }

    /// Returns true if no geometry has been added.
    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }

    /// Returns the total triangle count across all glyphs.
    pub fn total_triangles(&self) -> usize {
        self.total_indices / 3
    }
}

impl<V: Copy> RenderMesh<V> {
    /// Merges all glyph meshes into single vertex and index buffers.
    ///
    /// Index values are adjusted to account for the merged vertex offset.
    /// This is the most efficient format for GPU rendering.
    ///
    /// Returns (vertices, indices) ready for buffer upload.
    pub fn merge_all(&self) -> (Vec<V>, Vec<u32>) {
        let mut vertices = Vec::with_capacity(self.total_vertices);
        let mut indices = Vec::with_capacity(self.total_indices);

        for glyph in &self.glyphs {
            let base_index = vertices.len() as u32;
            vertices.extend_from_slice(&glyph.vertices);
            indices.extend(glyph.indices.iter().map(|i| i + base_index));
        }

        (vertices, indices)
    }

    /// Merges all meshes and returns byte slices for direct GPU upload.
    ///
    /// This avoids intermediate allocations when possible.
    pub fn merge_to_bytes(&self) -> (Vec<u8>, Vec<u8>) {
        let (vertices, indices) = self.merge_all();

        let vertex_bytes = if vertices.is_empty() {
            Vec::new()
        } else {
            let ptr = vertices.as_ptr() as *const u8;
            let len = vertices.len() * std::mem::size_of::<V>();
            // SAFETY: vertices is a contiguous Vec, V is repr(C)
            unsafe { std::slice::from_raw_parts(ptr, len).to_vec() }
        };

        let index_bytes = if indices.is_empty() {
            Vec::new()
        } else {
            let ptr = indices.as_ptr() as *const u8;
            let len = indices.len() * 4;
            // SAFETY: indices is a contiguous Vec of u32
            unsafe { std::slice::from_raw_parts(ptr, len).to_vec() }
        };

        (vertex_bytes, index_bytes)
    }
}

// =============================================================================
// Size and alignment assertions (compile-time checks)
// =============================================================================

const _: () = {
    // PositionedGlyphC should be exactly 20 bytes
    assert!(std::mem::size_of::<PositionedGlyphC>() == 20);
    // PositionedGlyphC should have 4-byte alignment
    assert!(std::mem::align_of::<PositionedGlyphC>() == 4);
    // DirectionC should be 1 byte
    assert!(std::mem::size_of::<DirectionC>() == 1);

    // Stage 5 mesh vertex types
    assert!(std::mem::size_of::<Vertex2D>() == 8);
    assert!(std::mem::align_of::<Vertex2D>() == 4);
    assert!(std::mem::size_of::<VertexUV>() == 16);
    assert!(std::mem::align_of::<VertexUV>() == 4);
    assert!(std::mem::size_of::<VertexColor>() == 24);
    assert!(std::mem::align_of::<VertexColor>() == 4);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positioned_glyph_c_conversion() {
        let rust_glyph = PositionedGlyph {
            id: 42,
            x: 10.5,
            y: 20.5,
            advance: 15.0,
            cluster: 3,
        };

        let c_glyph = PositionedGlyphC::from(&rust_glyph);
        assert_eq!(c_glyph.glyph_id, 42);
        assert_eq!(c_glyph.x, 10.5);
        assert_eq!(c_glyph.y, 20.5);
        assert_eq!(c_glyph.advance, 15.0);
        assert_eq!(c_glyph.cluster, 3);

        let back = PositionedGlyph::from(&c_glyph);
        assert_eq!(back, rust_glyph);
    }

    #[test]
    fn test_direction_c_conversion() {
        assert_eq!(
            DirectionC::from(Direction::LeftToRight),
            DirectionC::LeftToRight
        );
        assert_eq!(
            DirectionC::from(Direction::RightToLeft),
            DirectionC::RightToLeft
        );
        assert_eq!(
            DirectionC::from(Direction::TopToBottom),
            DirectionC::TopToBottom
        );
        assert_eq!(
            DirectionC::from(Direction::BottomToTop),
            DirectionC::BottomToTop
        );

        assert_eq!(
            Direction::from(DirectionC::LeftToRight),
            Direction::LeftToRight
        );
        assert_eq!(
            Direction::from(DirectionC::RightToLeft),
            Direction::RightToLeft
        );
    }

    #[test]
    fn test_shaping_result_c_roundtrip() {
        let rust_result = ShapingResult {
            glyphs: vec![
                PositionedGlyph {
                    id: 1,
                    x: 0.0,
                    y: 0.0,
                    advance: 10.0,
                    cluster: 0,
                },
                PositionedGlyph {
                    id: 2,
                    x: 10.0,
                    y: 0.0,
                    advance: 12.0,
                    cluster: 1,
                },
                PositionedGlyph {
                    id: 3,
                    x: 22.0,
                    y: 0.0,
                    advance: 8.0,
                    cluster: 2,
                },
            ],
            advance_width: 30.0,
            advance_height: 0.0,
            direction: Direction::LeftToRight,
        };

        let mut c_result = ShapingResultC::from_rust(&rust_result);
        assert_eq!(c_result.glyph_count, 3);
        assert_eq!(c_result.advance_width, 30.0);
        assert_eq!(c_result.direction, DirectionC::LeftToRight);

        unsafe {
            let slice = c_result.glyphs_slice();
            assert_eq!(slice.len(), 3);
            assert_eq!(slice[0].glyph_id, 1);
            assert_eq!(slice[1].x, 10.0);
            assert_eq!(slice[2].advance, 8.0);

            let back = c_result.to_rust();
            assert_eq!(back.glyphs.len(), 3);
            assert_eq!(back.advance_width, 30.0);
            assert_eq!(back.direction, Direction::LeftToRight);

            c_result.free();
            assert!(c_result.glyphs.is_null());
            assert_eq!(c_result.glyph_count, 0);
        }
    }

    #[test]
    fn test_shaping_result_c_empty() {
        let rust_result = ShapingResult {
            glyphs: vec![],
            advance_width: 0.0,
            advance_height: 0.0,
            direction: Direction::LeftToRight,
        };

        let mut c_result = ShapingResultC::from_rust(&rust_result);
        assert_eq!(c_result.glyph_count, 0);
        assert!(c_result.glyphs.is_null());

        unsafe {
            let slice = c_result.glyphs_slice();
            assert!(slice.is_empty());
            c_result.free(); // Should be safe even for empty
        }
    }

    #[test]
    fn test_glyph_iterator() {
        let result = ShapingResult {
            glyphs: vec![
                PositionedGlyph {
                    id: 1,
                    x: 0.0,
                    y: 0.0,
                    advance: 10.0,
                    cluster: 0,
                },
                PositionedGlyph {
                    id: 2,
                    x: 10.0,
                    y: 0.0,
                    advance: 12.0,
                    cluster: 1,
                },
            ],
            advance_width: 22.0,
            advance_height: 0.0,
            direction: Direction::LeftToRight,
        };

        let iter = GlyphIterator::new(&result);
        assert_eq!(iter.len(), 2);
        assert!(!iter.is_empty());
        assert_eq!(iter.remaining(), 2);

        let collected: Vec<_> = GlyphIterator::new(&result).collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0].glyph_id, 1);
        assert_eq!(collected[1].glyph_id, 2);
    }

    #[test]
    fn test_size_alignment() {
        // Verify our compile-time assertions match runtime
        assert_eq!(std::mem::size_of::<PositionedGlyphC>(), 20);
        assert_eq!(std::mem::align_of::<PositionedGlyphC>(), 4);
        assert_eq!(std::mem::size_of::<DirectionC>(), 1);
    }

    // =========================================================================
    // Stage 5 Mesh ABI Tests
    // =========================================================================

    #[test]
    fn test_vertex2d_size_alignment() {
        assert_eq!(std::mem::size_of::<Vertex2D>(), 8);
        assert_eq!(std::mem::align_of::<Vertex2D>(), 4);
    }

    #[test]
    fn test_vertex_uv_size_alignment() {
        assert_eq!(std::mem::size_of::<VertexUV>(), 16);
        assert_eq!(std::mem::align_of::<VertexUV>(), 4);
    }

    #[test]
    fn test_vertex_color_size_alignment() {
        assert_eq!(std::mem::size_of::<VertexColor>(), 24);
        assert_eq!(std::mem::align_of::<VertexColor>(), 4);
    }

    #[test]
    fn test_vertex2d_as_bytes() {
        let v = Vertex2D::new(1.0, 2.0);
        let bytes = v.as_bytes();
        assert_eq!(bytes.len(), 8);

        // Verify byte representation matches f32 layout
        let x_bytes = 1.0_f32.to_ne_bytes();
        let y_bytes = 2.0_f32.to_ne_bytes();
        assert_eq!(&bytes[0..4], &x_bytes);
        assert_eq!(&bytes[4..8], &y_bytes);
    }

    #[test]
    fn test_vertex_uv_as_bytes() {
        let v = VertexUV::new(1.0, 2.0, 0.5, 0.75);
        let bytes = v.as_bytes();
        assert_eq!(bytes.len(), 16);

        // Verify byte representation
        let x_bytes = 1.0_f32.to_ne_bytes();
        let u_bytes = 0.5_f32.to_ne_bytes();
        assert_eq!(&bytes[0..4], &x_bytes);
        assert_eq!(&bytes[8..12], &u_bytes);
    }

    #[test]
    fn test_vertex_color_as_bytes() {
        let v = VertexColor::new(1.0, 2.0, 1.0, 0.0, 0.0, 1.0);
        let bytes = v.as_bytes();
        assert_eq!(bytes.len(), 24);

        // Verify byte representation
        let r_bytes = 1.0_f32.to_ne_bytes();
        assert_eq!(&bytes[8..12], &r_bytes); // r is at offset 8
    }

    #[test]
    fn test_glyph_mesh_creation() {
        let mesh: GlyphMesh<Vertex2D> = GlyphMesh::new(42);
        assert_eq!(mesh.glyph_id, 42);
        assert!(mesh.is_empty());
        assert_eq!(mesh.triangle_count(), 0);
    }

    #[test]
    fn test_glyph_mesh_with_triangle() {
        let mut mesh: GlyphMesh<Vertex2D> = GlyphMesh::new(1);
        mesh.vertices = vec![
            Vertex2D::new(0.0, 0.0),
            Vertex2D::new(1.0, 0.0),
            Vertex2D::new(0.5, 1.0),
        ];
        mesh.indices = vec![0, 1, 2];

        assert!(!mesh.is_empty());
        assert_eq!(mesh.triangle_count(), 1);

        let vb = mesh.vertices_bytes();
        assert_eq!(vb.len(), 24); // 3 vertices × 8 bytes

        let ib = mesh.indices_bytes();
        assert_eq!(ib.len(), 12); // 3 indices × 4 bytes
    }

    #[test]
    fn test_render_mesh_push() {
        let mut rm: RenderMesh<Vertex2D> = RenderMesh::new();
        assert!(rm.is_empty());

        let mut mesh1 = GlyphMesh::new(1);
        mesh1.vertices = vec![
            Vertex2D::new(0.0, 0.0),
            Vertex2D::new(1.0, 0.0),
            Vertex2D::new(0.5, 1.0),
        ];
        mesh1.indices = vec![0, 1, 2];

        rm.push(mesh1);
        assert_eq!(rm.glyph_count(), 1);
        assert_eq!(rm.total_vertices, 3);
        assert_eq!(rm.total_indices, 3);
        assert_eq!(rm.total_triangles(), 1);
    }

    #[test]
    fn test_render_mesh_merge() {
        let mut rm: RenderMesh<Vertex2D> = RenderMesh::new();

        // First glyph: triangle at origin
        let mut mesh1 = GlyphMesh::new(1);
        mesh1.vertices = vec![
            Vertex2D::new(0.0, 0.0),
            Vertex2D::new(1.0, 0.0),
            Vertex2D::new(0.5, 1.0),
        ];
        mesh1.indices = vec![0, 1, 2];

        // Second glyph: triangle offset by 2
        let mut mesh2 = GlyphMesh::new(2);
        mesh2.vertices = vec![
            Vertex2D::new(2.0, 0.0),
            Vertex2D::new(3.0, 0.0),
            Vertex2D::new(2.5, 1.0),
        ];
        mesh2.indices = vec![0, 1, 2];

        rm.push(mesh1);
        rm.push(mesh2);

        let (vertices, indices) = rm.merge_all();
        assert_eq!(vertices.len(), 6);
        assert_eq!(indices.len(), 6);

        // First triangle indices unchanged
        assert_eq!(indices[0], 0);
        assert_eq!(indices[1], 1);
        assert_eq!(indices[2], 2);

        // Second triangle indices offset by 3 (base_index)
        assert_eq!(indices[3], 3);
        assert_eq!(indices[4], 4);
        assert_eq!(indices[5], 5);
    }

    #[test]
    fn test_render_mesh_merge_to_bytes() {
        let mut rm: RenderMesh<Vertex2D> = RenderMesh::new();

        let mut mesh = GlyphMesh::new(1);
        mesh.vertices = vec![
            Vertex2D::new(0.0, 0.0),
            Vertex2D::new(1.0, 0.0),
            Vertex2D::new(0.5, 1.0),
        ];
        mesh.indices = vec![0, 1, 2];

        rm.push(mesh);

        let (vb, ib) = rm.merge_to_bytes();
        assert_eq!(vb.len(), 24); // 3 vertices × 8 bytes
        assert_eq!(ib.len(), 12); // 3 indices × 4 bytes
    }

    #[test]
    fn test_render_mesh_empty() {
        let rm: RenderMesh<Vertex2D> = RenderMesh::new();
        let (vertices, indices) = rm.merge_all();
        assert!(vertices.is_empty());
        assert!(indices.is_empty());

        let (vb, ib) = rm.merge_to_bytes();
        assert!(vb.is_empty());
        assert!(ib.is_empty());
    }

    #[test]
    fn test_vertex_default() {
        let v2d = Vertex2D::default();
        assert_eq!(v2d.x, 0.0);
        assert_eq!(v2d.y, 0.0);

        let vuv = VertexUV::default();
        assert_eq!(vuv.x, 0.0);
        assert_eq!(vuv.u, 0.0);

        let vc = VertexColor::default();
        assert_eq!(vc.x, 0.0);
        assert_eq!(vc.r, 0.0);
        assert_eq!(vc.a, 0.0);
    }
}
