<!-- this_file: PLANSTEPS/05-integration-recipes.md -->

## 4. Concrete Integration Recipes

### Recipe 1: Integrating `typf` Shaping with `cosmic-text` (Rust)

**Scope rule:** Use `typf` for efficient shaping and delegate line layout to `cosmic-text`.

```rust
use std::sync::Arc;
use typf_core::{Pipeline, ShapingParams, PositionedGlyph};
use cosmic_text::{Buffer, Metrics, ShapeGlyph};
// Assuming necessary 'From'/'Into' traits are implemented for PositionedGlyph
// fn main() -> Result<(), Box<dyn std::error::Error>> { // Boilerplate removed for brevity

// 1. Setup the high-performance shaping pipeline
let pipeline = Pipeline::builder()
    .shaper(Arc::new(HarfBuzzShaper::new())) // Assuming HarfBuzz is registered
    .build()?;

// 2. Execute shaping (Stage 4)
let text = "Complex script mixing: LTR and RTL عربى";
let shaped_result: Vec<PositionedGlyph> = pipeline.run_shaping(text, font_ref, &ShapingParams::default())?;

// 3. Seamless conversion to target library's primitive
let cosmic_glyphs: Vec<ShapeGlyph> = shaped_result.into_iter()
    .map(Into::into) // Uses the provided Into<ShapeGlyph> implementation
    .collect();

// 4. Load into target layout buffer
let mut font_system = FontSystem::new();
let metrics = Metrics::new(14.0, 20.0);
let mut buffer = Buffer::new(&mut font_system, metrics);
// Note: Actual implementation depends on how cosmic-text exposes Buffer loading
// The core action is feeding converted glyph data.

// ... proceed with cosmic-text layout (line breaking, editing)
// }
```

### Recipe 2: Generating Egui/Wgpu Meshes (Rust GPU Path)

**Scope rule:** Use `typf`'s GPU rendering specialization to provide zero-copy mesh data for immediate-mode rendering frameworks.

```rust
use std::sync::Arc;
use typf_core::{Pipeline, RenderParams, GpuRenderer, RenderMesh};
use wgpu; // Graphics API dependency
use zerocopy::AsBytes; // Needed to get raw byte slice
// fn main() -> Result<(), Box<dyn std::error::Error>> { // Boilerplate removed

// 1. Setup GPU Context and pipeline (Stage 1-4 completed internally)
let pipeline = Pipeline::builder()
    // Assume a custom GpuRenderer is registered and fetched
    .renderer(Arc::new(CustomGpuRenderer::new()))
    .build()?;
let shaped_data = pipeline.run_shaping("High-throughput UI Text", font_ref, &ShapingParams::default())?;
let gpu_renderer: Arc<dyn GpuRenderer> = /* Get the active GpuRenderer implementation */ ;

// 2. Execute GPU Mesh Generation (Specialized Stage 5)
let mesh: RenderMesh = gpu_renderer.generate_mesh(shaped_data, &RenderParams::default());

// 3. Allocate WGPU Buffer (WGPU setup omitted)
let device: &wgpu::Device = /* initialized device */ ;
let vertex_buffer = device.create_buffer_init(
    &wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        // Direct conversion of #[repr(C)] vertex struct array to raw bytes
        contents: mesh.vertices.as_bytes(), 
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    }
);
// This achieves maximum efficiency by eliminating CPU copies (Goal A)

// ... proceed with Wgpu/Egui drawing calls
// }
```

### Recipe 3: Zero-Copy Glyphs to Pycairo (Python FFI)

**Scope rule:** Provide structured positional data optimized for the C ABI used by Pycairo, avoiding data duplication (Goal B).

```python
import typfpy
import cairo
import numpy # implicitly required for the zero-copy buffer protocol

# 1. Request zero-copy view of shaped data (Stage 4 output, FFI Contract)
# The output numpy.ndarray view adheres to the CairoGlyph layout (index: u32, x: f64, y: f64)
glyph_data_view = typfpy.Typf().get_cairo_glyphs_view("Hello Pycairo")

# 2. Consume the raw structured NumPy array view to create cairo.Glyph objects
# Since the memory layout matches cairo_glyph_t, creation is fast
cairo_glyphs = [
    cairo.Glyph(index=g['index'], x=g['x'], y=g['y'])
    for g in glyph_data_view
]

# 3. Use the glyphs in a Pycairo drawing context
ctx = cairo.Context(cairo.ImageSurface(cairo.FORMAT_ARGB32, 100, 100))
ctx.show_glyphs(cairo_glyphs)
ctx.write_to_png("cairo_output.png") #
```

### Recipe 4: Vector Paths for ReportLab (Python FFI)

**Scope rule:** Convert vector outlines from Rust (Stage 5 geometry) into ReportLab's low-level drawing primitives for high-fidelity PDF output.

```python
import typfpy
from reportlab.pdfgen import canvas

def render_vector_text_to_pdf(filename, text, font_path, size):
    engine = typfpy.Typf(shaper="harfbuzz", renderer="svg") # Use SVG Renderer/Geometry Provider
    
    # 1. Request vector primitives (Python list[dict]) (Stage 5 output)
    path_commands = engine.export_vector_paths_as_primitives(
        text, 
        font_path=font_path, 
        size=size
    )

    c = canvas.Canvas(filename)
    
    # 2. Map structured commands directly to ReportLab's primitive API
    for cmd in path_commands:
        cmd_type = cmd.get('type')
        if cmd_type == 'moveTo':
            c.moveTo(cmd['x'], cmd['y'])
        elif cmd_type == 'lineTo':
            c.lineTo(cmd['x'], cmd['y'])
        elif cmd_type == 'curveTo':
            # ReportLab curveTo takes three control points
            c.curveTo(cmd['x1'], cmd['y1'], cmd['x2'], cmd['y2'], cmd['x3'], cmd['y3'])
        elif cmd_type == 'closePath':
            c.closePath()
    
    c.fill()
    c.showPage()
    c.save()

# render_vector_text_to_pdf("report.pdf", "Vector Text", "font.ttf", 32)
```
