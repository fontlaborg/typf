# Chapter 24: Troubleshooting and Best Practices

## Overview

Even with a well-designed system like TYPF, issues can arise during development, deployment, and production use. This chapter provides comprehensive troubleshooting guidance, common problem solutions, and best practices for optimal TYPF usage. From debugging rendering issues to optimizing performance in production, this chapter equips you with the knowledge to identify, diagnose, and resolve problems efficiently while following established best practices for text rendering applications.

## Common Issues and Solutions

### Font Loading Problems

#### Issue: Font Not Found or Failed to Load

**Symptoms:**
- `TypfError::FontNotFound` or `TypfError::FontLoadFailed`
- Empty or garbled text output
- Panic during font parsing

**Debugging Steps:**
```rust
// Font debugging utilities
pub struct FontDebugger {
    font_path: PathBuf,
}

impl FontDebugger {
    pub fn new(font_path: PathBuf) -> Self {
        Self { font_path }
    }
    
    pub fn diagnose_font(&self) -> FontDiagnosis {
        let mut diagnosis = FontDiagnosis::new();
        
        // Check file existence and permissions
        diagnosis.file_exists = self.font_path.exists();
        diagnosis.readable = self.font_path.readable();
        diagnosis.file_size = self.font_path.metadata()
            .map(|m| m.len())
            .unwrap_or(0);
        
        if !diagnosis.file_exists {
            diagnosis.errors.push("Font file does not exist".to_string());
            return diagnosis;
        }
        
        if !diagnosis.readable {
            diagnosis.errors.push("Font file is not readable".to_string());
            return diagnosis;
        }
        
        // Try to read font bytes
        match std::fs::read(&self.font_path) {
            Ok(bytes) => {
                diagnosis.bytes_read = bytes.len();
                
                // Check font magic bytes
                diagnosis.font_type = self.detect_font_type(&bytes);
                
                if let Some(font_type) = &diagnosis.font_type {
                    // Try to parse with different libraries
                    diagnosis.read_fonts_result = self.try_read_fonts(&bytes);
                    diagnosis.skrifa_result = self.try_skrifa(&bytes);
                } else {
                    diagnosis.errors.push("Unknown font format or corrupted file".to_string());
                }
                
                // Check for common font issues
                diagnosis.check_common_issues(&bytes);
            }
            Err(e) => {
                diagnosis.errors.push(format!("Failed to read font file: {}", e));
            }
        }
        
        diagnosis
    }
    
    fn detect_font_type(&self, bytes: &[u8]) -> Option<String> {
        if bytes.len() < 4 {
            return None;
        }
        
        match &bytes[0..4] {
            [0x00, 0x01, 0x00, 0x00] | [0x74, 0x72, 0x75, 0x65] => Some("TrueType".to_string()),
            [0x4F, 0x54, 0x54, 0x4F] => Some("OpenType".to_string()),
            [0x77, 0x4F, 0x46, 0x46] => Some("WOFF".to_string()),
            [0x77, 0x4F, 0x46, 0x32] => Some("WOFF2".to_string()),
            _ => None,
        }
    }
    
    fn try_read_fonts(&self, bytes: &[u8]) -> Result<String, String> {
        use read_fonts::FontRef;
        
        match FontRef::new(bytes) {
            Ok(font) => {
                let family_name = font.family_name().unwrap_or("Unknown");
                Ok(format!("Successfully parsed with read_fonts. Family: {}", family_name))
            }
            Err(e) => Err(format!("read_fonts failed: {}", e)),
        }
    }
    
    fn try_skrifa(&self, bytes: &[u8]) -> Result<String, String> {
        use skrifa::FontRef;
        
        match FontRef::from_index(bytes, 0) {
            Ok(Some(font)) => {
                let family_name = font.family_name().unwrap_or("Unknown");
                Ok(format!("Successfully parsed with skrifa. Family: {}", family_name))
            }
            Ok(None) => Err("No font found in file".to_string()),
            Err(e) => Err(format!("skrifa failed: {}", e)),
        }
    }
}

#[derive(Debug)]
pub struct FontDiagnosis {
    pub file_exists: bool,
    pub readable: bool,
    pub file_size: u64,
    pub bytes_read: usize,
    pub font_type: Option<String>,
    pub read_fonts_result: Result<String, String>,
    pub skrifa_result: Result<String, String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl FontDiagnosis {
    pub fn new() -> Self {
        Self {
            file_exists: false,
            readable: false,
            file_size: 0,
            bytes_read: 0,
            font_type: None,
            read_fonts_result: Err("Not attempted".to_string()),
            skrifa_result: Err("Not attempted".to_string()),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn check_common_issues(&mut self, bytes: &[u8]) {
        // Check for empty font
        if bytes.is_empty() {
            self.errors.push("Font file is empty".to_string());
            return;
        }
        
        // Check for minimum font size
        if bytes.len() < 1024 {
            self.warnings.push("Font file is unusually small".to_string());
        }
        
        // Check for very large fonts (might indicate embedded fonts)
        if bytes.len() > 50 * 1024 * 1024 {
            self.warnings.push("Font file is very large, consider using subset".to_string());
        }
        
        // Check for missing required tables
        if let Some(font_type) = &self.font_type {
            if font_type.contains("TrueType") || font_type.contains("OpenType") {
                // Check for essential tables
                let required_tables = vec!["cmap", "glyf", "head", "hhea", "hmtx", "loca", "maxp"];
                for table in required_tables {
                    if !bytes.windows(4).any(|w| w == table.as_bytes()) {
                        self.warnings.push(format!("Missing required table: {}", table));
                    }
                }
            }
        }
    }
    
    pub fn summary(&self) -> String {
        if self.errors.is_empty() {
            if self.warnings.is_empty() {
                "Font appears to be healthy".to_string()
            } else {
                format!("Font is valid but has warnings: {}", self.warnings.join(", "))
            }
        } else {
            format!("Font has errors: {}", self.errors.join(", "))
        }
    }
}

// Usage example
pub fn debug_font_issue(font_path: &str) -> String {
    let debugger = FontDebugger::new(PathBuf::from(font_path));
    let diagnosis = debugger.diagnose_font();
    
    format!("Font Diagnosis for {}\n{}\nSummary: {}\nDetails:\n{:#?}",
        font_path,
        "=".repeat(50),
        diagnosis.summary(),
        diagnosis
    )
}
```

#### Issue: Memory Leak When Loading Multiple Fonts

**Symptoms:**
- Memory usage continuously increases
- System becomes sluggish after processing many fonts
- Out-of-memory errors in long-running processes

**Solution:**
```rust
// Proper font caching with memory management
pub struct FontCacheManager {
    cache: Arc<RwLock<LruCache<String, Arc<FontData>>>>,
    memory_tracker: Arc<MemoryTracker>,
    max_memory_mb: usize,
}

impl FontCacheManager {
    pub fn new(max_memory_mb: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(
                std::num::NonZeroUsize::new(1000).unwrap()
            ))),
            memory_tracker: Arc::new(MemoryTracker::new()),
            max_memory_mb,
        }
    }
    
    pub fn load_font(&self, path: &str) -> Result<Arc<FontData>, TypfError> {
        // Check cache first
        if let Ok(cache) = self.cache.read() {
            if let Some(font_data) = cache.get(path) {
                return Ok(Arc::clone(font_data));
            }
        }
        
        // Check memory constraints
        let current_memory = self.memory_tracker.current_usage();
        if current_memory >= self.max_memory_mb * 1024 * 1024 {
            self.evict_least_used()?;
        }
        
        // Load font
        let font_data = Arc::new(self.load_font_from_disk(path)?);
        let font_size = font_data.estimated_size();
        
        // Update cache and tracking
        {
            let mut cache = self.cache.write().unwrap();
            let old_font = cache.put(path.to_string(), Arc::clone(&font_data));
            
            if let Some(old_font) = old_font {
                self.memory_tracker.deallocate(old_font.estimated_size());
            }
        }
        
        self.memory_tracker.allocate(font_size);
        Ok(font_data)
    }
    
    fn evict_least_used(&self) -> Result<(), TypfError> {
        let mut to_evict = Vec::new();
        let mut freed_bytes = 0;
        
        {
            let mut cache = self.cache.write().unwrap();
            let target_bytes = self.max_memory_mb * 1024 * 1024 / 2; // Evict to 50% of max
            
            while freed_bytes < target_bytes && !cache.is_empty() {
                if let Some((key, font_data)) = cache.pop_lru() {
                    freed_bytes += font_data.estimated_size();
                    to_evict.push((key, font_data));
                }
            }
        }
        
        // Update memory tracking
        for (_, font_data) in to_evict {
            self.memory_tracker.deallocate(font_data.estimated_size());
        }
        
        Ok(())
    }
}
```

### Text Rendering Issues

#### Issue: Text Appears Upside Down or Flipped

**Symptoms:**
- Rendered text is inverted vertically
- Text flows in wrong direction
- Glyph positions seem incorrect

**Diagnosis and Solution:**
```rust
pub struct CoordinateSystemDebugger {
    pipeline: Pipeline,
}

impl CoordinateSystemDebugger {
    pub fn new() -> Result<Self, TypfError> {
        let pipeline = PipelineBuilder::new()
            .with_debug_mode(true)
            .build()?;
        
        Ok(Self { pipeline })
    }
    
    pub fn debug_coordinate_system(&self, text: &str, font_path: &str, font_size: f32) -> CoordinateDebugInfo {
        let mut debug_info = CoordinateDebugInfo::new();
        
        // Enable coordinate debugging
        let debug_pipeline = self.pipeline.with_coordinate_debugging();
        
        // Get raw glyph positions
        debug_info.raw_positions = debug_pipeline.get_raw_glyph_positions(text, font_path, font_size);
        
        // Get rendered bounds
        debug_info.render_bounds = debug_pipeline.get_render_bounds(text, font_path, font_size);
        
        // Check for coordinate system mismatches
        debug_info.check_coordinate_issues();
        
        debug_info
    }
    
    pub fn generate_visual_debug(&self, text: &str, font_path: &str, font_size: f32) -> Result<RenderOutput, TypfError> {
        let debug_pipeline = self.pipeline.with_visual_debugging();
        
        // Render with coordinate system visualization
        let result = debug_pipeline.render_with_coordinates(text, font_path, font_size)?;
        
        Ok(result)
    }
}

#[derive(Debug)]
pub struct CoordinateDebugInfo {
    pub raw_positions: Vec<GlyphPosition>,
    pub render_bounds: Rect,
    pub detected_issues: Vec<CoordinateIssue>,
    pub suggested_fixes: Vec<String>,
}

impl CoordinateDebugInfo {
    pub fn new() -> Self {
        Self {
            raw_positions: Vec::new(),
            render_bounds: Rect::zero(),
            detected_issues: Vec::new(),
            suggested_fixes: Vec::new(),
        }
    }
    
    pub fn check_coordinate_issues(&mut self) {
        // Check for Y-axis inversion
        if self.has_y_inversion() {
            self.detected_issues.push(CoordinateIssue::YAxisInverted);
            self.suggested_fixes.push("Apply Y-axis flip in renderer".to_string());
        }
        
        // Check for incorrect text direction
        if self.has_text_direction_issue() {
            self.detected_issues.push(CoordinateIssue::TextDirectionWrong);
            self.suggested_fixes.push("Check text direction settings".to_string());
        }
        
        // Check for origin mismatch
        if self.has_origin_mismatch() {
            self.detected_issues.push(CoordinateIssue::OriginMismatch);
            self.suggested_fixes.push("Adjust rendering origin".to_string());
        }
    }
    
    fn has_y_inversion(&self) -> bool {
        // Check if glyph positions suggest Y-axis inversion
        for pos in &self.raw_positions {
            if pos.y < -1000.0 || pos.y > 1000.0 {
                return true; // Unusual Y coordinates suggest inversion
            }
        }
        false
    }
    
    fn has_text_direction_issue(&self) -> bool {
        // Check if positions suggest wrong text direction
        if self.raw_positions.len() < 2 {
            return false;
        }
        
        let baseline_direction = self.raw_positions[1].x - self.raw_positions[0].x;
        
        // If first glyph is to the right of second, might be RTL issue
        baseline_direction < 0.0
    }
    
    fn has_origin_mismatch(&self) -> bool {
        // Check if rendering origin seems incorrect
        self.render_bounds.y < -1000.0 || self.render_bounds.y > 1000.0
    }
}

#[derive(Debug)]
pub enum CoordinateIssue {
    YAxisInverted,
    TextDirectionWrong,
    OriginMismatch,
    ScaleIncorrect,
}

// Fix coordinate system issues
pub fn fix_coordinate_system(mut result: RenderOutput, issues: &[CoordinateIssue]) -> RenderOutput {
    for issue in issues {
        match issue {
            CoordinateIssue::YAxisInverted => {
                // Flip Y coordinates
                result.flip_y_axis();
            }
            CoordinateIssue::TextDirectionWrong => {
                // Reverse glyph order
                result.reverse_glyph_order();
            }
            CoordinateIssue::OriginMismatch => {
                // Adjust origin
                result.adjust_origin(0.0, result.height as f32);
            }
            CoordinateIssue::ScaleIncorrect => {
                // Adjust scale (assuming 72 DPI to 96 DPI conversion)
                result.scale(96.0 / 72.0);
            }
        }
    }
    
    result
}
```

#### Issue: Glyph Positioning Incorrect

**Symptoms:**
- Characters overlapping or spaced incorrectly
- Kerning not applied
- Text appears stretched or compressed

**Solution:**
```rust
pub struct GlyphPositionDebugger {
    shaper: Box<dyn Shaper>,
}

impl GlyphPositionDebugger {
    pub fn new(shaper: &str) -> Result<Self, TypfError> {
        let shaper = create_shaper(shaper)?;
        Ok(Self { shaper })
    }
    
    pub fn analyze_positioning(&self, text: &str, font_path: &str, font_size: f32) -> PositionAnalysis {
        let mut analysis = PositionAnalysis::new();
        
        // Get shaping result
        let shaping_result = match self.shaper.shape_text(text, font_path) {
            Ok(result) => result,
            Err(e) => {
                analysis.errors.push(format!("Shaping failed: {}", e));
                return analysis;
            }
        };
        
        analysis.glyph_count = shaping_result.glyphs.len();
        analysis.original_positions = shaping_result.positions.clone();
        
        // Analyze spacing
        analysis.analyze_spacing();
        
        // Check kerning
        analysis.check_kerning();
        
        // Verify glyph advances
        analysis.verify_advances();
        
        // Check for ligatures
        analysis.analyze_ligatures();
        
        analysis
    }
}

#[derive(Debug)]
pub struct PositionAnalysis {
    pub glyph_count: usize,
    pub original_positions: Vec<Position>,
    pub spacing_issues: Vec<SpacingIssue>,
    pub kerning_problems: Vec<KerningProblem>,
    pub advance_errors: Vec<AdvanceError>,
    pub ligature_analysis: LigatureAnalysis,
    pub errors: Vec<String>,
    pub suggestions: Vec<String>,
}

impl PositionAnalysis {
    pub fn new() -> Self {
        Self {
            glyph_count: 0,
            original_positions: Vec::new(),
            spacing_issues: Vec::new(),
            kerning_problems: Vec::new(),
            advance_errors: Vec::new(),
            ligature_analysis: LigatureAnalysis::new(),
            errors: Vec::new(),
            suggestions: Vec::new(),
        }
    }
    
    pub fn analyze_spacing(&mut self) {
        if self.original_positions.len() < 2 {
            return;
        }
        
        let mut spacing_issues = Vec::new();
        
        for i in 1..self.original_positions.len() {
            let prev_pos = self.original_positions[i - 1];
            let current_pos = self.original_positions[i];
            let spacing = current_pos.x - prev_pos.x;
            
            // Check for unusual spacing
            if spacing < 0.0 {
                spacing_issues.push(SpacingIssue {
                    glyph_index: i,
                    issue_type: SpacingIssueType::NegativeSpacing,
                    value: spacing,
                });
            } else if spacing > 1000.0 {
                spacing_issues.push(SpacingIssue {
                    glyph_index: i,
                    issue_type: SpacingIssueType::ExcessiveSpacing,
                    value: spacing,
                });
            }
        }
        
        self.spacing_issues = spacing_issues;
        
        if !self.spacing_issues.is_empty() {
            self.suggestions.push("Check font metrics and kerning settings".to_string());
        }
    }
    
    pub fn check_kerning(&mut self) {
        // Implementation would check for expected kerning pairs
        // This is a simplified version
        let expected_kerning_pairs = vec![
            ("T", "e"), ("T", "a"), ("A", "V"), // Common kerning pairs
        ];
        
        for (left_char, right_char) in expected_kerning_pairs {
            // Find corresponding glyphs and check positioning
            if let Some(issue) = self.check_kerning_pair(left_char, right_char) {
                self.kerning_problems.push(issue);
            }
        }
        
        if !self.kerning_problems.is_empty() {
            self.suggestions.push("Enable kerning in shaper configuration".to_string());
        }
    }
    
    fn check_kerning_pair(&self, left: &str, right: &str) -> Option<KerningProblem> {
        // Simplified implementation - would need actual glyph mapping
        None // Return None for now
    }
}

#[derive(Debug)]
pub struct SpacingIssue {
    pub glyph_index: usize,
    pub issue_type: SpacingIssueType,
    pub value: f32,
}

#[derive(Debug)]
pub enum SpacingIssueType {
    NegativeSpacing,
    ExcessiveSpacing,
    ZeroSpacing,
}
```

### Performance Issues

#### Issue: Slow Rendering Performance

**Symptoms:**
- Rendering takes longer than expected
- High CPU usage during text processing
- Memory usage grows continuously

**Performance Diagnostic:**
```rust
pub struct PerformanceDiagnostic {
    metrics: Vec<PerformanceMetric>,
    analysis: PerformanceAnalysis,
}

impl PerformanceDiagnostic {
    pub fn run_benchmark(&mut self, test_cases: Vec<RenderTestCase>) -> Result<(), TypfError> {
        self.metrics.clear();
        
        for test_case in test_cases {
            let metric = self.run_single_benchmark(&test_case)?;
            self.metrics.push(metric);
        }
        
        self.analyze_performance();
        Ok(())
    }
    
    fn run_single_benchmark(&self, test_case: &RenderTestCase) -> Result<PerformanceMetric, TypfError> {
        let mut metric = PerformanceMetric::new(test_case.name.clone());
        
        // Warm up
        for _ in 0..5 {
            let _ = self.render_test_case(test_case)?;
        }
        
        // Benchmark
        let mut times = Vec::new();
        let mut memory_samples = Vec::new();
        
        for iteration in 0..test_case.iterations {
            let start_time = Instant::now();
            let memory_before = self.get_memory_usage();
            
            let result = self.render_test_case(test_case)?;
            
            let memory_after = self.get_memory_usage();
            let duration = start_time.elapsed();
            
            times.push(duration);
            memory_samples.push(memory_after.saturating_sub(memory_before));
            
            if iteration % 10 == 0 {
                log::debug!("Iteration {}: {:?}", iteration, duration);
            }
        }
        
        metric.calculate_statistics(times, memory_samples);
        
        // Analyze bottlenecks
        metric.identify_bottlenecks(test_case)?;
        
        Ok(metric)
    }
    
    fn analyze_performance(&mut self) {
        self.analysis = PerformanceAnalysis::new();
        
        // Analyze overall performance
        self.analysis.overall_performance = self.calculate_overall_performance();
        
        // Identify patterns
        self.analysis.identify_performance_patterns(&self.metrics);
        
        // Generate recommendations
        self.analysis.generate_recommendations(&self.metrics);
    }
    
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# TYPF Performance Diagnostic Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", chrono::Utc::now()));
        
        // Overall performance summary
        report.push_str("## Overall Performance\n\n");
        report.push_str(&format!("- Average render time: {:.2}ms\n", 
            self.analysis.overall_performance.avg_render_time));
        report.push_str(&format!("- Peak memory usage: {:.2}MB\n", 
            self.analysis.overall_performance.peak_memory_mb));
        report.push_str(&format!("- Renders per second: {:.2}\n\n", 
            self.analysis.overall_performance.renders_per_second));
        
        // Detailed metrics
        report.push_str("## Detailed Metrics\n\n");
        for metric in &self.metrics {
            report.push_str(&format!("### {}\n", metric.test_name));
            report.push_str(&format!("- Average: {:.2}ms\n", metric.avg_time));
            report.push_str(&format!("- Min: {:.2}ms\n", metric.min_time));
            report.push_str(&format!("- Max: {:.2}ms\n", metric.max_time));
            report.push_str(&format!("- Std Dev: {:.2}ms\n", metric.std_deviation));
            report.push_str(&format!("- Memory: {:.2}MB\n", metric.avg_memory_mb));
            
            if !metric.bottlenecks.is_empty() {
                report.push_str("- Bottlenecks:\n");
                for bottleneck in &metric.bottlenecks {
                    report.push_str(&format!("  - {}\n", bottleneck));
                }
            }
            
            report.push_str("\n");
        }
        
        // Recommendations
        report.push_str("## Recommendations\n\n");
        for recommendation in &self.analysis.recommendations {
            report.push_str(&format!("- {}\n", recommendation));
        }
        
        report
    }
}

#[derive(Debug)]
pub struct PerformanceMetric {
    pub test_name: String,
    pub avg_time: f64,
    pub min_time: f64,
    pub max_time: f64,
    pub std_deviation: f64,
    pub avg_memory_mb: f64,
    pub peak_memory_mb: f64,
    pub bottlenecks: Vec<String>,
}

impl PerformanceMetric {
    pub fn new(test_name: String) -> Self {
        Self {
            test_name,
            avg_time: 0.0,
            min_time: f64::INFINITY,
            max_time: 0.0,
            std_deviation: 0.0,
            avg_memory_mb: 0.0,
            peak_memory_mb: 0.0,
            bottlenecks: Vec::new(),
        }
    }
    
    pub fn calculate_statistics(&mut self, times: Vec<Duration>, memory_samples: Vec<usize>) {
        let times_ms: Vec<f64> = times.iter()
            .map(|d| d.as_millis() as f64)
            .collect();
        
        self.avg_time = times_ms.iter().sum::<f64>() / times_ms.len() as f64;
        self.min_time = times_ms.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        self.max_time = times_ms.iter().fold(0.0, |a, &b| a.max(b));
        
        // Calculate standard deviation
        let variance = times_ms.iter()
            .map(|x| (x - self.avg_time).powi(2))
            .sum::<f64>() / times_ms.len() as f64;
        self.std_deviation = variance.sqrt();
        
        // Memory statistics
        let memory_mb: Vec<f64> = memory_samples.iter()
            .map(|&bytes| bytes as f64 / (1024.0 * 1024.0))
            .collect();
        
        self.avg_memory_mb = memory_mb.iter().sum::<f64>() / memory_mb.len() as f64;
        self.peak_memory_mb = memory_mb.iter().fold(0.0, |a, &b| a.max(b));
    }
    
    pub fn identify_bottlenecks(&mut self, test_case: &RenderTestCase) -> Result<(), TypfError> {
        // Identify performance bottlenecks based on timing patterns
        if self.avg_time > 100.0 {
            self.bottlenecks.push("Slow overall rendering time".to_string());
            
            if test_case.text.len() > 1000 {
                self.bottlenecks.push("Large text size may be causing slowdown".to_string());
            }
            
            if test_case.font_size > 72.0 {
                self.bottlenecks.push("Large font size increases rendering time".to_string());
            }
        }
        
        if self.peak_memory_mb > 100.0 {
            self.bottlenecks.push("High memory usage during rendering".to_string());
        }
        
        if self.std_deviation > self.avg_time * 0.5 {
            self.bottlenecks.push("Inconsistent performance - possible caching issues".to_string());
        }
        
        Ok(())
    }
}
```

## Debugging Tools and Utilities

### Visual Debugging

```rust
// Visual debugging utilities for rendering issues
pub struct VisualDebugger {
    pipeline: Pipeline,
}

impl VisualDebugger {
    pub fn new() -> Result<Self, TypfError> {
        let pipeline = PipelineBuilder::new()
            .with_debug_mode(true)
            .build()?;
        
        Ok(Self { pipeline })
    }
    
    pub fn create_debug_overlay(&self, 
        text: &str, 
        font_path: &str, 
        font_size: f32,
        debug_options: &DebugOptions,
    ) -> Result<RenderOutput, TypfError> {
        let mut renderer = DebugRenderer::new(debug_options.clone());
        
        // Render base text
        let base_result = self.pipeline.render_text(text, font_path, font_size)?;
        
        // Get glyph positions for debugging
        let shaping_result = self.pipeline.shape_text(text, font_path)?;
        
        // Create debug overlay
        let debug_result = renderer.render_debug_overlay(&base_result, &shaping_result, debug_options)?;
        
        // Combine base result with debug overlay
        self.combine_debug_layers(base_result, debug_result)
    }
    
    pub fn export_debug_info(&self, 
        text: &str, 
        font_path: &str, 
        font_size: f32,
    ) -> Result<DebugExport, TypfError> {
        let shaping_result = self.pipeline.shape_text(text, font_path)?;
        let render_result = self.pipeline.render_text(text, font_path, font_size)?;
        
        Ok(DebugExport {
            text: text.to_string(),
            font_path: font_path.to_string(),
            font_size,
            shaping_result: serde_json::to_value(&shaping_result).unwrap(),
            render_info: RenderInfo {
                width: render_result.width,
                height: render_result.height,
                format: render_result.format,
                data_size: render_result.data.len(),
            },
            glyph_details: self.extract_glyph_details(&shaping_result),
            performance_metrics: self.get_performance_metrics(),
            timestamp: chrono::Utc::now(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DebugOptions {
    pub show_glyph_boxes: bool,
    pub show_baseline: bool,
    pub show_advance_widths: bool,
    pub show_kerning: bool,
    pub show_coordinate_grid: bool,
    pub highlight_overflow: bool,
    pub color_scheme: DebugColorScheme,
}

#[derive(Debug, Clone)]
pub enum DebugColorScheme {
    Default,
    HighContrast,
    ColorBlind,
    PrintFriendly,
}

pub struct DebugRenderer {
    options: DebugOptions,
}

impl DebugRenderer {
    pub fn new(options: DebugOptions) -> Self {
        Self { options }
    }
    
    pub fn render_debug_overlay(&self, 
        base_result: &RenderOutput,
        shaping_result: &ShapingResult,
        debug_options: &DebugOptions,
    ) -> Result<RenderOutput, TypfError> {
        let mut overlay = RenderOutput::create_empty(base_result.width, base_result.height);
        
        // Draw baseline
        if debug_options.show_baseline {
            self.draw_baseline(&mut overlay, shaping_result);
        }
        
        // Draw glyph boxes
        if debug_options.show_glyph_boxes {
            self.draw_glyph_boxes(&mut overlay, shaping_result);
        }
        
        // Draw advance widths
        if debug_options.show_advance_widths {
            self.draw_advance_widths(&mut overlay, shaping_result);
        }
        
        // Draw kerning indicators
        if debug_options.show_kerning {
            self.draw_kerning(&mut overlay, shaping_result);
        }
        
        // Draw coordinate grid
        if debug_options.show_coordinate_grid {
            self.draw_coordinate_grid(&mut overlay);
        }
        
        Ok(overlay)
    }
    
    fn draw_baseline(&self, overlay: &mut RenderOutput, shaping_result: &ShapingResult) {
        let baseline_y = shaping_result.baseline_y;
        
        // Draw horizontal baseline
        for x in 0..overlay.width {
            let y = baseline_y as u32;
            if y < overlay.height {
                overlay.set_pixel(x, y, Color::red());
            }
        }
    }
    
    fn draw_glyph_boxes(&self, overlay: &mut RenderOutput, shaping_result: &ShapingResult) {
        for (i, glyph) in shaping_result.glyphs.iter().enumerate() {
            let position = &shaping_result.positions[i];
            let bounds = glyph.bounds;
            
            let x0 = (position.x + bounds.x_min) as u32;
            let y0 = (position.y + bounds.y_min) as u32;
            let x1 = (position.x + bounds.x_max) as u32;
            let y1 = (position.y + bounds.y_max) as u32;
            
            // Draw rectangle outline
            self.draw_rectangle(overlay, x0, y0, x1, y1, Color::blue());
        }
    }
    
    fn draw_rectangle(&self, overlay: &mut RenderOutput, x0: u32, y0: u32, x1: u32, y1: u32, color: Color) {
        // Draw top and bottom edges
        for x in x0..=x1.min(overlay.width - 1) {
            if y0 < overlay.height {
                overlay.set_pixel(x, y0, color);
            }
            if y1 < overlay.height {
                overlay.set_pixel(x, y1, color);
            }
        }
        
        // Draw left and right edges
        for y in y0..=y1.min(overlay.height - 1) {
            if x0 < overlay.width {
                overlay.set_pixel(x0, y, color);
            }
            if x1 < overlay.width {
                overlay.set_pixel(x1, y, color);
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DebugExport {
    pub text: String,
    pub font_path: String,
    pub font_size: f32,
    pub shaping_result: serde_json::Value,
    pub render_info: RenderInfo,
    pub glyph_details: Vec<GlyphDetail>,
    pub performance_metrics: PerformanceMetrics,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

### Logging and Tracing

```rust
// Comprehensive logging system for TYPF
pub struct TypfLogger {
    logger: tracing::Subscriber,
    metrics_collector: Arc<MetricsCollector>,
}

impl TypfLogger {
    pub fn new(config: &LoggingConfig) -> Result<Self, TypfError> {
        let logger = self.create_tracing_subscriber(config)?;
        let metrics_collector = Arc::new(MetricsCollector::new());
        
        Ok(Self {
            logger,
            metrics_collector,
        })
    }
    
    fn create_tracing_subscriber(&self, config: &LoggingConfig) -> Result<tracing::Subscriber, TypfError> {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
        
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(config.include_target)
            .with_thread_ids(config.include_thread_id)
            .with_file(config.include_file)
            .with_line_number(config.include_line_number)
            .with_level(config.include_level);
        
        let filter = tracing_subscriber::filter::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::filter::EnvFilter::new(&config.default_level));
        
        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer);
        
        if config.enable_json_logging {
            let json_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_target(false);
            
            subscriber.with(json_layer).init();
        } else {
            subscriber.init();
        }
        
        // Return a dummy subscriber for the struct
        Ok(tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(filter)
            .boxed())
    }
    
    pub fn log_render_start(&self, text: &str, font: &str, font_size: f32) {
        tracing::info!(
            text_length = text.len(),
            font = font,
            font_size = font_size,
            "Starting text rendering"
        );
        
        self.metrics_collector.increment_counter("renders_started");
    }
    
    pub fn log_render_complete(&self, result: &RenderOutput, duration: Duration) {
        tracing::info!(
            width = result.width,
            height = result.height,
            data_size = result.data.len(),
            duration_ms = duration.as_millis(),
            "Text rendering completed"
        );
        
        self.metrics_collector.record_timing("render_duration", duration);
        self.metrics_collector.record_memory("render_data_size", result.data.len());
    }
    
    pub fn log_shaping_complete(&self, glyph_count: usize, duration: Duration) {
        tracing::debug!(
            glyph_count = glyph_count,
            duration_ms = duration.as_millis(),
            "Text shaping completed"
        );
        
        self.metrics_collector.record_timing("shaping_duration", duration);
    }
    
    pub fn log_cache_operation(&self, cache_type: &str, operation: &str, hit: bool) {
        tracing::trace!(
            cache_type = cache_type,
            operation = operation,
            hit = hit,
            "Cache operation"
        );
        
        if hit {
            self.metrics_collector.increment_counter(&format!("{}_cache_hits", cache_type));
        } else {
            self.metrics_collector.increment_counter(&format!("{}_cache_misses", cache_type));
        }
    }
    
    pub fn log_error(&self, error: &TypfError, context: &str) {
        tracing::error!(
            error = %error,
            context = context,
            "TYPF error occurred"
        );
        
        self.metrics_collector.increment_counter("errors");
        self.metrics_collector.increment_counter(&format!("errors_{}", error.error_type()));
    }
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub default_level: String,
    pub include_target: bool,
    pub include_thread_id: bool,
    pub include_file: bool,
    pub include_line_number: bool,
    pub include_level: bool,
    pub enable_json_logging: bool,
    pub log_to_file: Option<PathBuf>,
    pub enable_performance_logging: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            default_level: "info".to_string(),
            include_target: true,
            include_thread_id: false,
            include_file: true,
            include_line_number: true,
            include_level: true,
            enable_json_logging: false,
            log_to_file: None,
            enable_performance_logging: true,
        }
    }
}
```

## Best Practices

### Performance Best Practices

```rust
// Performance optimization guidelines
pub struct PerformanceGuidelines;

impl PerformanceGuidelines {
    pub fn font_caching_best_practices() -> Vec<String> {
        vec![
            "Implement LRU caching for fonts with configurable size limits".to_string(),
            "Use memory compression for font data when memory is constrained".to_string(),
            "Preload commonly used fonts during application startup".to_string(),
            "Clear font cache when memory usage exceeds thresholds".to_string(),
            "Use background threads for font loading to avoid blocking UI".to_string(),
        ]
    }
    
    pub fn rendering_optimization_tips() -> Vec<String> {
        vec![
            "Choose appropriate shaper based on script complexity (None for Latin ICU/HarfBuzz for complex scripts)".to_string(),
            "Use Skia for complex rendering needs, Orge for minimal embedded systems".to_string(),
            "Batch multiple text renders when possible to reduce overhead".to_string(),
            "Reuse render targets and avoid unnecessary allocations".to_string(),
            "Consider GPU acceleration for large-scale rendering operations".to_string(),
        ]
    }
    
    pub fn memory_management_guidelines() -> Vec<String> {
        vec![
            "Always use Arc<Font> to avoid duplicating font data".to_string(),
            "Implement proper cleanup for temporary font data".to_string(),
            "Monitor memory usage and implement memory pressure responses".to_string(),
            "Use arena allocators for temporary glyph data".to_string(),
            "Avoid holding large render buffers longer than necessary".to_string(),
        ]
    }
}

// Performance monitoring and alerting
pub struct PerformanceMonitor {
    thresholds: PerformanceThresholds,
    alert_handler: Box<dyn AlertHandler>,
}

impl PerformanceMonitor {
    pub fn new(thresholds: PerformanceThresholds) -> Self {
        Self {
            thresholds,
            alert_handler: Box::new(LoggingAlertHandler::new()),
        }
    }
    
    pub fn check_performance(&self, metrics: &PerformanceMetrics) -> Vec<PerformanceAlert> {
        let mut alerts = Vec::new();
        
        // Check render time
        if metrics.avg_render_time > self.thresholds.max_render_time {
            alerts.push(PerformanceAlert::RenderTimeExceeded {
                actual: metrics.avg_render_time,
                threshold: self.thresholds.max_render_time,
            });
        }
        
        // Check memory usage
        if metrics.peak_memory_mb > self.thresholds.max_memory_mb {
            alerts.push(PerformanceAlert::MemoryExceeded {
                actual: metrics.peak_memory_mb,
                threshold: self.thresholds.max_memory_mb,
            });
        }
        
        // Check error rate
        if metrics.error_rate > self.thresholds.max_error_rate {
            alerts.push(PerformanceAlert::ErrorRateExceeded {
                actual: metrics.error_rate,
                threshold: self.thresholds.max_error_rate,
            });
        }
        
        // Send alerts
        for alert in &alerts {
            self.alert_handler.handle_alert(alert);
        }
        
        alerts
    }
}

#[derive(Debug)]
pub enum PerformanceAlert {
    RenderTimeExceeded { actual: f64, threshold: f64 },
    MemoryExceeded { actual: f64, threshold: f64 },
    ErrorRateExceeded { actual: f64, threshold: f64 },
    CacheHitRateLow { actual: f64, threshold: f64 },
}

pub trait AlertHandler {
    fn handle_alert(&self, alert: &PerformanceAlert);
}

pub struct LoggingAlertHandler;

impl LoggingAlertHandler {
    pub fn new() -> Self {
        Self
    }
}

impl AlertHandler for LoggingAlertHandler {
    fn handle_alert(&self, alert: &PerformanceAlert) {
        match alert {
            PerformanceAlert::RenderTimeExceeded { actual, threshold } => {
                tracing::warn!(
                    actual_ms = actual,
                    threshold_ms = threshold,
                    "Render time performance alert"
                );
            }
            PerformanceAlert::MemoryExceeded { actual, threshold } => {
                tracing::warn!(
                    actual_mb = actual,
                    threshold_mb = threshold,
                    "Memory usage performance alert"
                );
            }
            _ => {
                tracing::warn!("Performance alert: {:?}", alert);
            }
        }
    }
}
```

### Development Workflow Best Practices

```rust
// Development workflow utilities
pub struct DevelopmentWorkflow;

impl DevelopmentWorkflow {
    pub fn setup_development_environment() -> Result<(), TypfError> {
        // Set up logging for development
        let config = LoggingConfig {
            default_level: "debug".to_string(),
            include_file: true,
            include_line_number: true,
            include_thread_id: true,
            enable_performance_logging: true,
            ..Default::default()
        };
        
        let _logger = TypfLogger::new(&config)?;
        
        // Set up development cache
        Self::setup_development_cache()?;
        
        // Enable development features
        Self::enable_development_features()?;
        
        Ok(())
    }
    
    fn setup_development_cache() -> Result<(), TypfError> {
        // Create development cache directories
        let cache_dirs = vec![
            "cache/fonts",
            "cache/shapes",
            "cache/renders",
        ];
        
        for dir in cache_dirs {
            std::fs::create_dir_all(dir)?;
        }
        
        Ok(())
    }
    
    fn enable_development_features() -> Result<(), TypfError> {
        // Enable debug modes in various components
        std::env::set_var("TYPF_DEBUG", "1");
        std::env::set_var("TYPF_LOG_LEVEL", "debug");
        std::env::set_var("TYPF_PERFORMANCE_LOGGING", "1");
        
        Ok(())
    }
    
    pub fn run_development_tests() -> Result<TestResults, TypfError> {
        let mut test_results = TestResults::new();
        
        // Run unit tests
        test_results.add_result("unit_tests", Self::run_unit_tests()?);
        
        // Run integration tests
        test_results.add_result("integration_tests", Self::run_integration_tests()?);
        
        // Run performance tests
        test_results.add_result("performance_tests", Self::run_performance_tests()?);
        
        // Run memory leak tests
        test_results.add_result("memory_tests", Self::run_memory_leak_tests()?);
        
        Ok(test_results)
    }
    
    fn run_unit_tests() -> Result<bool, TypfError> {
        // Implementation would run cargo test --lib
        Ok(true) // Placeholder
    }
    
    fn run_integration_tests() -> Result<bool, TypfError> {
        // Implementation would run integration test suite
        Ok(true) // Placeholder
    }
    
    fn run_performance_tests() -> Result<bool, TypfError> {
        // Implementation would run performance benchmarks
        Ok(true) // Placeholder
    }
    
    fn run_memory_leak_tests() -> Result<bool, TypfError> {
        // Implementation would run memory leak detection
        Ok(true) // Placeholder
    }
}

#[derive(Debug)]
pub struct TestResults {
    results: HashMap<String, bool>,
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }
    
    pub fn add_result(&mut self, test_name: &str, passed: bool) {
        self.results.insert(test_name.to_string(), passed);
    }
    
    pub fn all_passed(&self) -> bool {
        self.results.values().all(|&passed| passed)
    }
    
    pub fn summary(&self) -> String {
        let passed = self.results.values().filter(|&&passed| passed).count();
        let total = self.results.len();
        
        format!("{}/{} tests passed", passed, total)
    }
}

// Pre-commit hooks and quality checks
pub struct QualityChecker;

impl QualityChecker {
    pub fn run_all_checks() -> Result<QualityReport, TypfError> {
        let mut report = QualityReport::new();
        
        // Code formatting check
        report.add_check("formatting", Self::check_code_formatting()?);
        
        // Linting check
        report.add_check("linting", Self::check_linting()?);
        
        // Documentation check
        report.add_check("documentation", Self::check_documentation()?);
        
        // Security check
        report.add_check("security", Self::check_security()?);
        
        // License check
        report.add_check("licenses", Self::check_licenses()?);
        
        Ok(report)
    }
    
    fn check_code_formatting() -> Result<bool, TypfError> {
        let output = std::process::Command::new("cargo")
            .args(&["fmt", "--check"])
            .output()?;
        
        Ok(output.status.success())
    }
    
    fn check_linting() -> Result<bool, TypfError> {
        let output = std::process::Command::new("cargo")
            .args(&["clippy", "--", "-D", "warnings"])
            .output()?;
        
        Ok(output.status.success())
    }
    
    fn check_documentation() -> Result<bool, TypfError> {
        // Check that all public items have documentation
        // This would be a more sophisticated implementation
        Ok(true)
    }
    
    fn check_security() -> Result<bool, TypfError> {
        let output = std::process::Command::new("cargo")
            .args(&["audit"])
            .output()?;
        
        Ok(output.status.success())
    }
    
    fn check_licenses() -> Result<bool, TypfError> {
        let output = std::process::Command::new("cargo")
            .args(&["deny", "check", "licenses"])
            .output()?;
        
        Ok(output.status.success())
    }
}

#[derive(Debug)]
pub struct QualityReport {
    checks: HashMap<String, bool>,
}

impl QualityReport {
    pub fn new() -> Self {
        Self {
            checks: HashMap::new(),
        }
    }
    
    pub fn add_check(&mut self, check_name: &str, passed: bool) {
        self.checks.insert(check_name.to_string(), passed);
    }
    
    pub fn all_passed(&self) -> bool {
        self.checks.values().all(|&passed| passed)
    }
}
```

By using these comprehensive troubleshooting tools and following best practices, developers can quickly identify and resolve issues with TYPF while maintaining optimal performance and code quality. The debugging utilities, logging systems, and performance monitoring capabilities provide deep insights into the text rendering pipeline, enabling efficient problem resolution and proactive performance optimization.