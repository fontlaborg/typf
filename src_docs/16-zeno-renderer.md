# Chapter 16: Zeno Renderer

## Overview

The Zeno renderer is TYPF's specialized vector graphics output backend, designed for creating high-quality SVG, PDF, and other vector format outputs. Unlike raster renderers that produce pixel-based images, Zeno generates resolution-independent vector representations that maintain perfect quality at any scale, making it ideal for printing, web graphics, and scalable content.

## Architecture

### Vector Processing Pipeline

```rust
#[derive(Debug, Clone)]
pub struct ZenoRenderer {
    pub path_builder: PathBuilder,           // Vector path construction
    pub stroke_processor: StrokeProcessor,   // Outline stroking
    pub fill_processor: FillProcessor,       // Fill pattern generation
    pub export_engine: VectorExportEngine,  // Format-specific output
    pub config: ZenoConfig,
}

pub struct PathBuilder {
    pub curves: Vec<BezierCurve>,           // Bézier curve definitions
    pub commands: Vec<PathCommand>,         // Path command stream
    pub bounds: Rect,                       // Computed path bounds
    pub metadata: PathMetadata,             // Path grouping and styling
}

pub struct VectorExportEngine {
    pub svg_exporter: SvgExporter,
    pub pdf_exporter: PdfExporter,
    pub eps_exporter: EpsExporter,
    pub format: VectorFormat,
}
```

### Rendering Flow

```
Shaping Result → Zeno Renderer → Vector Paths → Export Engine → SVG/PDF/EPS
                ↗              ↘                ↘
            Glyph Paths    Stroke/Fill      Format Headers
```

1. **Input**: Shaped glyph data from any shaper
2. **Path Extraction**: Convert glyphs to vector paths
3. **Styling**: Apply fill, stroke, and effects
4. **Export**: Generate format-specific vector output

## Vector Path Generation

### Glyph Outline Processing

```rust
impl ZenoRenderer {
    pub fn render_shaped_text(
        &self,
        shaped: &ShapingResult,
        font: &Font,
        viewport: ViewportConfig,
    ) -> Result<ZenoRenderResult> {
        // 1. Extract vector paths from font glyphs
        let mut vector_paths = Vec::new();
        
        for (glyph_id, position) in shaped.glyphs.iter().zip(shaped.positions.iter()) {
            let glyph_path = self.extract_glyph_path(*glyph_id, font)?;
            let transformed_path = glyph_path.transform(&Transform::translation(
                position.x_offset,
                position.y_offset,
            ));
            
            vector_paths.push(transformed_path);
        }
        
        // 2. Apply styling and effects
        let styled_paths = self.apply_styling(&vector_paths, &font, &viewport)?;
        
        // 3. Generate vector output
        let output = match self.config.format {
            VectorFormat::SVG => self.generate_svg_output(&styled_paths)?,
            VectorFormat::PDF => self.generate_pdf_output(&styled_paths)?,
            VectorFormat::EPS => self.generate_eps_output(&styled_paths)?,
        };
        
        Ok(ZenoRenderResult {
            output,
            paths: styled_paths,
            bounds: self.compute_total_bounds(&styled_paths),
        })
    }
    
    fn extract_glyph_path(&self, glyph_id: u32, font: &Font) -> Result<VectorPath> {
        let outline = font.get_glyph_outline(glyph_id)?;
        
        let mut path_builder = PathBuilder::new();
        
        // Convert outline segments to Bézier curves
        for segment in outline.segments() {
            match segment {
                OutlineSegment::MoveTo(pt) => {
                    path_builder.move_to(pt.x, pt.y);
                },
                OutlineSegment::LineTo(pt) => {
                    path_builder.line_to(pt.x, pt.y);
                },
                OutlineSegment::CurveTo(c1, c2, end) => {
                    path_builder.cubic_to(c1.x, c1.y, c2.x, c2.y, end.x, end.y);
                },
                OutlineSegment::QuadTo(c, end) => {
                    path_builder.quad_to(c.x, c.y, end.x, end.y);
                },
            }
        }
        
        path_builder.close();
        Ok(path_builder.build())
    }
}
```

### Advanced Path Operations

```rust
impl PathBuilder {
    pub fn apply_stroking(&self, stroke_config: &StrokeConfig) -> Result<VectorPath> {
        let mut stroked_paths = Vec::new();
        
        for subpath in self.subpaths.iter() {
            let stroked_subpath = self.stroke_subpath(subpath, stroke_config)?;
            stroked_paths.push(stroked_subpath);
        }
        
        // Combine all stroked subpaths
        let mut combined = VectorPath::new();
        for path in stroked_paths {
            combined.append(path);
        }
        
        Ok(combined)
    }
    
    pub fn apply_offsetting(&self, offset: f32) -> Result<VectorPath> {
        // Path offsetting for creating outlines, glow effects, etc.
        let mut offset_paths = Vec::new();
        
        for subpath in self.subpaths.iter() {
            let offset_path = self.offset_subpath(subpath, offset)?;
            offset_paths.push(offset_path);
        }
        
        // Resolve self-intersections and combine
        let mut combined = VectorPath::new();
        for path in offset_paths {
            combined = self.boolean_combine(&combined, &path, BooleanOp::Union)?;
        }
        
        Ok(combined)
    }
    
    pub fn simplify_paths(&self, tolerance: f32) -> VectorPath {
        // Douglas-Peucker path simplification
        let mut simplified = VectorPath::new();
        
        for subpath in self.subpaths.iter() {
            let simplified_subpath = self.simplify_subpath(subpath, tolerance);
            simplified.append(simplified_subpath);
        }
        
        simplified
    }
}
```

## Export Formats

### SVG Export

```rust
impl SvgExporter {
    pub fn export_to_svg(
        &self,
        paths: &[StyledVectorPath],
        config: &SvgExportConfig,
    ) -> Result<String> {
        let mut svg_content = String::new();
        
        // SVG header
        svg_content.push_str(&format!(
            r#"<svg width="{}" height="{}" viewBox="{} {} {} {}" xmlns="http://www.w3.org/2000/svg">"#,
            config.width,
            config.height,
            config.view_x,
            config.view_y,
            config.view_width,
            config.view_height,
        ));
        
        // Style definitions
        if !config.embedded_styles.is_empty() {
            svg_content.push_str("<style>");
            svg_content.push_str(&config.embedded_styles);
            svg_content.push_str("</style>");
        }
        
        // Export each path
        for (index, path) in paths.iter().enumerate() {
            svg_content.push_str(&self.export_path_to_svg(path, index)?);
        }
        
        svg_content.push_str("</svg>");
        
        Ok(svg_content)
    }
    
    fn export_path_to_svg(
        &self,
        path: &StyledVectorPath,
        index: usize,
    ) -> Result<String> {
        let mut path_data = String::new();
        
        for command in path.path.commands.iter() {
            match command {
                PathCommand::MoveTo(x, y) => {
                    path_data.push_str(&format!("M {} {} ", x, y));
                },
                PathCommand::LineTo(x, y) => {
                    path_data.push_str(&format!("L {} {} ", x, y));
                },
                PathCommand::CubicTo(x1, y1, x2, y2, x, y) => {
                    path_data.push_str(&format!("C {} {} {} {} {} {} ", x1, y1, x2, y2, x, y));
                },
                PathCommand::QuadTo(x1, y1, x, y) => {
                    path_data.push_str(&format!("Q {} {} {} {} ", x1, y1, x, y));
                },
                PathCommand::Close => {
                    path_data.push_str("Z ");
                },
            }
        }
        
        let style = self.generate_svg_style(&path.style);
        
        Ok(format!(
            r#"<path d="{}" class="path-{}" {} />"#,
            path_data.trim(),
            index,
            style
        ))
    }
    
    fn generate_svg_style(&self, style: &PathStyle) -> String {
        let mut style_attrs = Vec::new();
        
        if let Some(fill) = &style.fill {
            style_attrs.push(format!("fill:{}", fill.to_svg_color()));
        }
        
        if let Some(stroke) = &style.stroke {
            style_attrs.push(format!("stroke:{}", stroke.color.to_svg_color()));
            style_attrs.push(format!("stroke-width:{}", stroke.width));
            
            if stroke.dash_pattern.is_some() {
                style_attrs.push(format!("stroke-dasharray:{}", stroke.to_dash_array()));
            }
        }
        
        if style.opacity < 1.0 {
            style_attrs.push(format!("opacity:{}", style.opacity));
        }
        
        style_attrs.join("; ")
    }
}
```

### PDF Export

```rust
impl PdfExporter {
    pub fn export_to_pdf(
        &self,
        paths: &[StyledVectorPath],
        config: &PdfExportConfig,
    ) -> Result<Vec<u8>> {
        let mut pdf_writer = PdfWriter::new();
        
        // PDF header
        pdf_writer.write_header(&config.metadata)?;
        
        // Page setup
        pdf_writer.begin_page(config.width, config.height)?;
        
        // Graphics state
        pdf_writer.set_graphics_state(&config.graphics_state)?;
        
        // Draw paths
        for path in paths.iter() {
            self.draw_path_to_pdf(&mut pdf_writer, path)?;
        }
        
        pdf_writer.end_page()?;
        
        Ok(pdf_writer.finish()?)
    }
    
    fn draw_path_to_pdf(
        &self,
        pdf_writer: &mut PdfWriter,
        path: &StyledVectorPath,
    ) -> Result<()> {
        // Begin path construction
        pdf_writer.begin_path();
        
        // Add path commands
        for command in path.path.commands.iter() {
            match command {
                PathCommand::MoveTo(x, y) => {
                    pdf_writer.move_to(*x, *y);
                },
                PathCommand::LineTo(x, y) => {
                    pdf_writer.line_to(*x, *y);
                },
                PathCommand::CubicTo(x1, y1, x2, y2, x, y) => {
                    pdf_writer.cubic_to(*x1, *y1, *x2, *y2, *x, *y);
                },
                PathCommand::QuadTo(x1, y1, x, y) => {
                    pdf_writer.quad_to(*x1, *y1, *x, *y);
                },
                PathCommand::Close => {
                    pdf_writer.close_path();
                },
            }
        }
        
        // Apply styling and render
        if let Some(fill) = &path.style.fill {
            pdf_writer.set_fill_color(&fill.color);
            pdf_writer.fill_path();
        }
        
        if let Some(stroke) = &path.style.stroke {
            pdf_writer.set_stroke_color(&stroke.color);
            pdf_writer.set_line_width(stroke.width);
            pdf_writer.stroke_path();
        }
        
        Ok(())
    }
}
```

## Effects and Styling

### Gradient Fills

```rust
impl ZenoRenderer {
    pub fn apply_gradient_fill(
        &self,
        path: &VectorPath,
        gradient: &GradientDefinition,
    ) -> Result<StyledVectorPath> {
        let bounds = path.compute_bounds();
        let gradient_transform = self.compute_gradient_transform(gradient, &bounds);
        
        let style = PathStyle {
            fill: Some(PathFill::Gradient(GradientFill {
                definition: gradient.clone(),
                transform: gradient_transform,
            })),
            stroke: None,
            opacity: 1.0,
        };
        
        Ok(StyledVectorPath {
            path: path.clone(),
            style,
        })
    }
    
    fn compute_gradient_transform(
        &self,
        gradient: &GradientDefinition,
        bounds: &Rect,
    ) -> Transform {
        match gradient {
            GradientDefinition::Linear { start, end } => {
                // Map gradient coordinates to path bounds
                let scale_x = bounds.width / (end.x - start.x);
                let scale_y = bounds.height / (end.y - start.y);
                
                Transform::translation(bounds.min_x, bounds.min_y)
                    .then_scale(scale_x, scale_y)
            },
            GradientDefinition::Radial { center, radius } => {
                let scale = bounds.width.max(bounds.height) / (radius * 2.0);
                
                Transform::translation(center.x, center.y)
                    .then_scale(scale, scale)
                    .then_translation(bounds.min_x, bounds.min_y)
            },
        }
    }
}
```

### Strokes and Outlines

```rust
#[derive(Debug, Clone)]
pub struct StrokeConfig {
    pub width: f32,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f32,
    pub dash_pattern: Option<Vec<f32>>,
    pub dash_offset: f32,
}

#[derive(Debug, Clone)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

impl ZenoRenderer {
    pub fn apply_outlining(
        &self,
        path: &VectorPath,
        stroke_config: &StrokeConfig,
    ) -> Result<StyledVectorPath> {
        let stroked_path = path.apply_stroking(stroke_config)?;
        
        let style = PathStyle {
            fill: None,
            stroke: Some(PathStroke {
                color: Color::BLACK,
                width: stroke_config.width,
                line_cap: stroke_config.line_cap.clone(),
                line_join: stroke_config.line_join.clone(),
                dash_pattern: stroke_config.dash_pattern.clone(),
                dash_offset: stroke_config.dash_offset,
            }),
            opacity: 1.0,
        };
        
        Ok(StyledVectorPath {
            path: stroked_path,
            style,
        })
    }
}
```

## Performance Optimization

### Path Caching

```rust
impl ZenoRenderer {
    pub fn enable_glyph_path_caching(&mut self, cache_size: usize) {
        self.glyph_path_cache = Some(LruCache::new(
            std::num::NonZeroUsize::new(cache_size).unwrap(),
        ));
    }
    
    fn get_cached_glyph_path(&mut self, glyph_id: u32, font: &Font) -> Result<VectorPath> {
        let cache_key = GlyphPathKey::new(glyph_id, font.id());
        
        if let Some(cached_path) = self.glyph_path_cache.as_mut().and_then(|cache| {
            cache.get(&cache_key)
        }) {
            return Ok(cached_path.clone());
        }
        
        // Generate and cache path
        let path = self.extract_glyph_path(glyph_id, font)?;
        
        if let Some(ref mut cache) = self.glyph_path_cache {
            cache.put(cache_key, path.clone());
        }
        
        Ok(path)
    }
}
```

### Parallel Processing

```rust
impl ZenoRenderer {
    pub fn render_shaped_text_parallel(
        &self,
        shaped: &ShapingResult,
        font: &Font,
        viewport: ViewportConfig,
    ) -> Result<ZenoRenderResult> {
        use rayon::prelude::*;
        
        // Process glyphs in parallel
        let vector_paths: Result<Vec<_>> = shaped
            .glyphs
            .par_iter()
            .zip(shaped.positions.par_iter())
            .map(|(&glyph_id, position)| {
                let glyph_path = self.extract_glyph_path(glyph_id, font)?;
                let transformed_path = glyph_path.transform(&Transform::translation(
                    position.x_offset,
                    position.y_offset,
                ));
                
                Ok(transformed_path)
            })
            .collect();
        
        let vector_paths = vector_paths?;
        
        // Apply styling (still parallelizable)
        let styled_paths = self.apply_styling_parallel(&vector_paths, font, &viewport)?;
        
        // Generate output
        let output = match self.config.format {
            VectorFormat::SVG => self.generate_svg_output(&styled_paths)?,
            VectorFormat::PDF => self.generate_pdf_output(&styled_paths)?,
            VectorFormat::EPS => self.generate_eps_output(&styled_paths)?,
        };
        
        Ok(ZenoRenderResult {
            output,
            paths: styled_paths,
            bounds: self.compute_total_bounds(&styled_paths),
        })
    }
}
```

## Configuration

### Vector Export Configuration

```rust
#[derive(Debug, Clone)]
pub struct ZenoConfig {
    pub format: VectorFormat,
    pub precision: f32,                    // Coordinate precision
    pub optimization: OptimizationConfig,  // Path optimization settings
    pub styling: StylingConfig,            // Default styling
    pub export: ExportConfig,              // Format-specific settings
}

#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    pub simplify_paths: bool,
    pub simplify_tolerance: f32,
    pub remove_redundant_points: bool,
    pub merge_adjacent_paths: bool,
    pub optimize_curves: bool,
}

#[derive(Debug, Clone)]
pub enum VectorFormat {
    SVG,
    PDF,
    EPS,
}
```

### Python Configuration

```python
import typf

# SVG export configuration
svg_config = typf.ZenoConfig(
    format="svg",
    precision=2.0,
    optimization=typf.OptimizationConfig(
        simplify_paths=True,
        simplify_tolerance=0.1,
        remove_redundant_points=True,
        merge_adjacent_paths=False,  # Preserve glyph boundaries
        optimize_curves=True,
    ),
    styling=typf.StylingConfig(
        fill_color="black",
        stroke_width=0.0,  # No stroke by default
        opacity=1.0,
    ),
    export=typf.SvgExportConfig(
        width=800,
        height=600,
        view_box=(0, 0, 800, 600),
        embed_fonts=False,
    )
)

# PDF export configuration  
pdf_config = typf.ZenoConfig(
    format="pdf",
    precision=1.0,  # Higher precision for print
    optimization=typf.OptimizationConfig(
        simplify_paths=False,  # Preserve exact paths for print
        simplify_tolerance=0.01,
        remove_redundant_points=True,
        merge_adjacent_paths=False,
        optimize_curves=False,
    ),
    export=typf.PdfExportConfig(
        page_size=(612, 792),  # Letter size
        margin=(72, 72, 72, 72),  # 1-inch margins
        metadata=typf.PdfMetadata(
            title="TYPF Text Rendering",
            creator="TYPF Zeno Renderer",
        ),
    )
)

renderer = typf.Typf(renderer="zeno", zeno_config=svg_config)
```

## Error Handling

### Vector-Specific Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum ZenoRendererError {
    #[error("Glyph outline extraction failed for glyph {glyph_id}: {source}")]
    GlyphOutlineExtractionFailed { glyph_id: u32, source: Error },
    
    #[error("Path construction failed: {message}")]
    PathConstructionFailed { message: String },
    
    #[error("Export format {format} not supported")]
    UnsupportedFormat { format: String },
    
    #[error("Vector file generation failed: {source}")]
    ExportFailed { source: Error },
    
    #[error("Path optimization failed: {message}")]
    OptimizationFailed { message: String },
}
```

## Testing and Validation

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_svg_export() {
        let renderer = ZenoRenderer::new(test_svg_config());
        let font = load_test_font();
        let shaped = shape_simple_text("SVG", &font);
        
        let result = renderer.render_shaped_text(
            &shaped,
            &font,
            test_viewport(),
        ).unwrap();
        
        match result.output {
            ZenoOutput::SVG { content, .. } => {
                assert!(content.contains("<svg"));
                assert!(content.contains("</svg>"));
                assert!(content.contains("path d="));
            },
            _ => panic!("Expected SVG output"),
        }
    }
    
    #[test]
    fn test_path_stroking() {
        let renderer = ZenoRenderer::new(test_config());
        let font = load_test_font();
        let shaped = shape_simple_text("A", &font);
        
        let stroke_config = StrokeConfig {
            width: 2.0,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            miter_limit: 4.0,
            dash_pattern: None,
            dash_offset: 0.0,
        };
        
        let result = renderer.apply_outlining(
            &shaped.glyph_paths[0],
            &stroke_config,
        ).unwrap();
        
        assert!(result.style.stroke.is_some());
        assert_eq!(result.style.stroke.as_ref().unwrap().width, 2.0);
    }
}
```

### Integration Tests

The Zeno renderer is tested with:

- **Format Compliance**: Valid SVG/PDF/EPS output
- **Visual Validation**: Pixel-perfect comparisons with reference renderers
- **Path Accuracy**: Coordinate precision validation
- **Format Features**: Gradients, transforms, effects

## Use Cases

### Web Graphics

```python
# Generate SVG for web applications
web_config = typf.ZenoConfig(
    format="svg",
    precision=2.0,
    optimization=typf.OptimizationConfig(
        simplify_paths=True,
        simplify_tolerance=0.1,
        remove_redundant_points=True,
        merge_adjacent_paths=False,
        optimize_curves=True,
    ),
)

renderer = typf.Typf(renderer="zeno", zeno_config=web_config)
svg_output = renderer.render_text("Web Graphics", font_size=24.0)

# SVG can be embedded directly in HTML
html_content = f'<div class="text">{svg_output.content}</div>'
```

### Print Publishing

```python
# Generate RGB PDF for publishing
print_config = typf.ZenoConfig(
    format="pdf",
    precision=0.1,  # High precision for print
    optimization=typf.OptimizationConfig(
        simplify_paths=False,  # Preserve exact paths
        remove_redundant_points=False,
        merge_adjacent_paths=False,
        optimize_curves=False,
    ),
)

renderer = typf.Typf(renderer="zeno", zeno_config=print_config)
pdf_bytes = renderer.render_text("Print Quality Text", font_size=12.0)

# PDF ready for professional printing
with open("output.pdf", "wb") as f:
    f.write(pdf_bytes.content)
```

## Best Practices

### Vector Output Optimization

1. **Precision Tuning**: Balance file size vs. accuracy
2. **Path Simplification**: Remove unnecessary detail for web graphics
3. **Curve Optimization**: Convert quadratic to cubic when beneficial
4. **Path Merging**: Combine adjacent paths when appropriate
5. **Format Selection**: Choose SVG for web, PDF for print, EPS for legacy systems

### Performance Considerations

1. **Enable Caching**: Glyph path caching significantly speeds processing
2. **Parallel Processing**: Use parallel rendering for large documents
3. **Memory Management**: Reuse path builders and temporary buffers
4. **Early Bounding**: Compute bounds early for optimization

### Quality Assurance

1. **Visual Validation**: Always review vector output visually
2. **Format Compliance**: Validate against format specifications
3. **Cross-Platform**: Test output on different viewing platforms
4. **Size Limitations**: Be aware of format-specific size constraints

The Zeno renderer provides TYPF's vector graphics capabilities, enabling resolution-independent text output for web, print, and other applications where scalability and quality are paramount.