//! Zero-copy mesh upload pattern for wgpu integration
//!
//! This example demonstrates how to use typf's mesh ABI types for GPU rendering
//! with wgpu. The types in `typf_core::ffi` are designed for direct buffer upload
//! without intermediate copies.
//!
//! ## Key Types
//!
//! - `Vertex2D`: 8 bytes, position only (x, y)
//! - `VertexUV`: 16 bytes, position + texture coords
//! - `VertexColor`: 24 bytes, position + RGBA color
//! - `GlyphMesh<V>`: vertices + indices for one glyph
//! - `RenderMesh<V>`: collection with merge_all() for batching
//!
//! ## Memory Layout
//!
//! All vertex types are `#[repr(C)]` with no padding, allowing direct reinterpret
//! as byte slices via `vertices_bytes()` and `indices_bytes()`.
//!
//! ## Integration Pattern
//!
//! 1. Generate meshes from glyph outlines (tessellation)
//! 2. Collect into `RenderMesh<Vertex2D>`
//! 3. Call `merge_all()` to combine for batch rendering
//! 4. Upload bytes directly to GPU buffers
//!
//! ## Note
//!
//! This example doesn't depend on wgpu at runtime - it demonstrates the pattern
//! and shows the types. For actual GPU rendering, add wgpu to your project.

use typf_core::ffi::{GlyphMesh, RenderMesh, Vertex2D, VertexColor, VertexUV};

/// Simulates tessellating a glyph outline into triangles.
///
/// In a real implementation, this would use a tessellation library like
/// lyon or earcutr to convert path operations into triangle meshes.
fn tessellate_glyph(glyph_id: u32, x_offset: f32, _y_offset: f32) -> GlyphMesh<Vertex2D> {
    // Simulate a simple quad (2 triangles) for each glyph
    // Real tessellation would use GlyphPath from ffi.rs path ops
    let mut mesh = GlyphMesh::with_capacity(glyph_id, 4, 6);

    // Quad vertices (counterclockwise)
    let width = 0.5;
    let height = 1.0;
    mesh.vertices = vec![
        Vertex2D::new(x_offset, 0.0),            // bottom-left
        Vertex2D::new(x_offset + width, 0.0),    // bottom-right
        Vertex2D::new(x_offset + width, height), // top-right
        Vertex2D::new(x_offset, height),         // top-left
    ];

    // Two triangles (CCW winding)
    mesh.indices = vec![
        0, 1, 2, // first triangle
        0, 2, 3, // second triangle
    ];

    mesh
}

/// Demonstrates vertex buffer layout for wgpu pipeline creation.
///
/// This shows the wgpu::VertexBufferLayout configuration for each vertex type.
fn print_vertex_layouts() {
    println!("=== wgpu Vertex Buffer Layouts ===\n");

    // Vertex2D layout
    println!("Vertex2D (8 bytes):");
    println!("  wgpu::VertexBufferLayout {{");
    println!("      array_stride: 8,");
    println!("      step_mode: wgpu::VertexStepMode::Vertex,");
    println!("      attributes: &[");
    println!("          wgpu::VertexAttribute {{");
    println!("              format: wgpu::VertexFormat::Float32x2,");
    println!("              offset: 0,");
    println!("              shader_location: 0, // @location(0) position");
    println!("          }},");
    println!("      ],");
    println!("  }}\n");

    // VertexUV layout
    println!("VertexUV (16 bytes):");
    println!("  wgpu::VertexBufferLayout {{");
    println!("      array_stride: 16,");
    println!("      step_mode: wgpu::VertexStepMode::Vertex,");
    println!("      attributes: &[");
    println!("          wgpu::VertexAttribute {{");
    println!("              format: wgpu::VertexFormat::Float32x2,");
    println!("              offset: 0,");
    println!("              shader_location: 0, // @location(0) position");
    println!("          }},");
    println!("          wgpu::VertexAttribute {{");
    println!("              format: wgpu::VertexFormat::Float32x2,");
    println!("              offset: 8,");
    println!("              shader_location: 1, // @location(1) uv");
    println!("          }},");
    println!("      ],");
    println!("  }}\n");

    // VertexColor layout
    println!("VertexColor (24 bytes):");
    println!("  wgpu::VertexBufferLayout {{");
    println!("      array_stride: 24,");
    println!("      step_mode: wgpu::VertexStepMode::Vertex,");
    println!("      attributes: &[");
    println!("          wgpu::VertexAttribute {{");
    println!("              format: wgpu::VertexFormat::Float32x2,");
    println!("              offset: 0,");
    println!("              shader_location: 0, // @location(0) position");
    println!("          }},");
    println!("          wgpu::VertexAttribute {{");
    println!("              format: wgpu::VertexFormat::Float32x4,");
    println!("              offset: 8,");
    println!("              shader_location: 1, // @location(1) color (rgba)");
    println!("          }},");
    println!("      ],");
    println!("  }}\n");
}

/// Demonstrates buffer creation code pattern for wgpu.
fn print_buffer_creation_pattern(vertex_bytes: &[u8], index_bytes: &[u8]) {
    println!("=== wgpu Buffer Creation Pattern ===\n");
    println!("// Zero-copy buffer upload (using wgpu::util::DeviceExt)");
    println!("let vertex_buffer = device.create_buffer_init(");
    println!("    &wgpu::util::BufferInitDescriptor {{");
    println!("        label: Some(\"Text Vertices\"),");
    println!(
        "        contents: &vertex_bytes, // {} bytes",
        vertex_bytes.len()
    );
    println!("        usage: wgpu::BufferUsages::VERTEX,");
    println!("    }},");
    println!(");\n");
    println!("let index_buffer = device.create_buffer_init(");
    println!("    &wgpu::util::BufferInitDescriptor {{");
    println!("        label: Some(\"Text Indices\"),");
    println!(
        "        contents: &index_bytes, // {} bytes",
        index_bytes.len()
    );
    println!("        usage: wgpu::BufferUsages::INDEX,");
    println!("    }},");
    println!(");\n");
}

/// Demonstrates the draw call pattern.
fn print_draw_call_pattern(index_count: usize) {
    println!("=== wgpu Draw Call Pattern ===\n");
    println!("// In render pass:");
    println!("render_pass.set_pipeline(&text_pipeline);");
    println!("render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));");
    println!("render_pass.set_index_buffer(");
    println!("    index_buffer.slice(..),");
    println!("    wgpu::IndexFormat::Uint32,");
    println!(");");
    println!("render_pass.draw_indexed(0..{}, 0, 0..1);\n", index_count);
}

fn main() {
    println!("=== typf + wgpu Zero-Copy Mesh Upload Demo ===\n");

    // Step 1: Create mesh collection for a text string
    let text = "Hello";
    let mut render_mesh: RenderMesh<Vertex2D> = RenderMesh::with_capacity(text.len());

    println!("Tessellating {} glyphs...", text.len());
    let mut x_offset = 0.0;
    for ch in text.chars() {
        let glyph_id = ch as u32;
        let mesh = tessellate_glyph(glyph_id, x_offset, 0.0);

        println!(
            "  Glyph '{}' (id={}): {} vertices, {} indices",
            ch,
            glyph_id,
            mesh.vertices.len(),
            mesh.indices.len()
        );

        render_mesh.push(mesh);
        x_offset += 0.6; // Advance position
    }

    println!(
        "\nTotal: {} glyphs, {} vertices, {} indices, {} triangles\n",
        render_mesh.glyph_count(),
        render_mesh.total_vertices,
        render_mesh.total_indices,
        render_mesh.total_triangles()
    );

    // Step 2: Merge all glyphs into single buffers for batch rendering
    let (vertices, indices) = render_mesh.merge_all();
    println!(
        "Merged buffers: {} vertices, {} indices",
        vertices.len(),
        indices.len()
    );

    // Step 3: Get byte slices for direct GPU upload
    let (vertex_bytes, index_bytes) = render_mesh.merge_to_bytes();
    println!(
        "Byte sizes: {} vertex bytes, {} index bytes\n",
        vertex_bytes.len(),
        index_bytes.len()
    );

    // Verify merged indices are correctly offset
    println!("Merged index verification:");
    for (tri_idx, chunk) in indices.chunks(3).enumerate() {
        println!("  Triangle {}: indices {:?}", tri_idx, chunk);
    }
    println!();

    // Print wgpu integration patterns
    print_vertex_layouts();
    print_buffer_creation_pattern(&vertex_bytes, &index_bytes);
    print_draw_call_pattern(indices.len());

    // Demonstrate other vertex types
    println!("=== Other Vertex Type Examples ===\n");

    // VertexUV for textured glyphs (bitmap/SDF)
    let uv_vertex = VertexUV::new(0.0, 0.0, 0.0, 1.0);
    println!(
        "VertexUV example: pos=({}, {}), uv=({}, {})",
        uv_vertex.x, uv_vertex.y, uv_vertex.u, uv_vertex.v
    );
    println!("  Byte representation: {:?}\n", uv_vertex.as_bytes());

    // VertexColor for COLR glyphs
    let color_vertex = VertexColor::new(0.0, 0.0, 1.0, 0.5, 0.0, 1.0);
    println!(
        "VertexColor example: pos=({}, {}), color=({}, {}, {}, {})",
        color_vertex.x,
        color_vertex.y,
        color_vertex.r,
        color_vertex.g,
        color_vertex.b,
        color_vertex.a
    );
    println!("  Byte representation: {:?}\n", color_vertex.as_bytes());

    // Summary
    println!("=== Integration Summary ===\n");
    println!("1. typf tessellates glyph outlines → GlyphMesh<V>");
    println!("2. Collect glyphs into RenderMesh<V>");
    println!("3. Call merge_all() or merge_to_bytes() for batching");
    println!("4. Upload bytes directly to wgpu buffers (zero-copy)");
    println!("5. Draw with indexed triangles");
    println!();
    println!("Memory safety: All vertex types are #[repr(C)] with documented sizes.");
    println!("No bytemuck dependency required - types provide as_bytes() methods.");
}
