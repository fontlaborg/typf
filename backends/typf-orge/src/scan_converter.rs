// this_file: backends/typf-orge/src/scan_converter.rs

//! Scan converter for rasterizing vector outlines to bitmaps.
//!
//! Implements scanline rasterization with edge tables and active edge lists.
//! Supports both monochrome and grayscale rendering (via oversampling).

use crate::curves::{subdivide_cubic, subdivide_quadratic};
use crate::edge::{Edge, EdgeList};
use crate::fixed::F26Dot6;
use crate::{DropoutMode, FillRule};

use skrifa::outline::OutlinePen;

/// Main scan conversion engine.
///
/// Converts vector outlines (lines and curves) to rasterized bitmaps using
/// scanline algorithm with edge tables.
#[derive(Debug)]
pub struct ScanConverter {
    /// Edge table: one EdgeList per scanline (indexed by Y)
    edge_table: Vec<EdgeList>,

    /// Active edge list for current scanline
    active_edges: EdgeList,

    /// Current path position (for move_to/line_to)
    current_x: F26Dot6,
    current_y: F26Dot6,

    /// Start of current contour (for close)
    contour_start_x: F26Dot6,
    contour_start_y: F26Dot6,

    /// Fill rule (non-zero winding or even-odd)
    fill_rule: FillRule,

    /// Dropout control mode
    dropout_mode: DropoutMode,

    /// Bitmap dimensions (in pixels)
    width: usize,
    height: usize,
}

impl ScanConverter {
    /// Create a new scan converter for given bitmap size.
    ///
    /// # Arguments
    ///
    /// * `width` - Bitmap width in pixels
    /// * `height` - Bitmap height in pixels
    ///
    /// # Returns
    ///
    /// New `ScanConverter` initialized with default settings:
    /// - Fill rule: NonZeroWinding
    /// - Dropout mode: None
    pub fn new(width: usize, height: usize) -> Self {
        // Pre-allocate edge table with one list per scanline
        // Use with_capacity to avoid reallocations (see ALLOCATION.md)
        let mut edge_table = Vec::with_capacity(height);
        for _ in 0..height {
            edge_table.push(EdgeList::with_capacity(16)); // Estimate ~16 edges per scanline
        }

        Self {
            edge_table,
            active_edges: EdgeList::with_capacity(32), // Estimate ~32 active edges
            current_x: F26Dot6::ZERO,
            current_y: F26Dot6::ZERO,
            contour_start_x: F26Dot6::ZERO,
            contour_start_y: F26Dot6::ZERO,
            fill_rule: FillRule::NonZeroWinding,
            dropout_mode: DropoutMode::None,
            width,
            height,
        }
    }

    /// Set the fill rule.
    pub fn set_fill_rule(&mut self, rule: FillRule) {
        self.fill_rule = rule;
    }

    /// Set the dropout control mode.
    pub fn set_dropout_mode(&mut self, mode: DropoutMode) {
        self.dropout_mode = mode;
    }

    /// Get the current fill rule.
    pub fn fill_rule(&self) -> FillRule {
        self.fill_rule
    }

    /// Get the current dropout mode.
    pub fn dropout_mode(&self) -> DropoutMode {
        self.dropout_mode
    }

    /// Reset the scan converter for a new outline.
    ///
    /// Clears all edge tables and resets current position.
    pub fn reset(&mut self) {
        for list in &mut self.edge_table {
            list.clear();
        }
        self.active_edges.clear();
        self.current_x = F26Dot6::ZERO;
        self.current_y = F26Dot6::ZERO;
        self.contour_start_x = F26Dot6::ZERO;
        self.contour_start_y = F26Dot6::ZERO;
    }

    /// Move to a new position (start new contour).
    ///
    /// # Arguments
    ///
    /// * `x, y` - New position in 26.6 fixed-point format
    pub fn move_to(&mut self, x: F26Dot6, y: F26Dot6) {
        self.current_x = x;
        self.current_y = y;
        self.contour_start_x = x;
        self.contour_start_y = y;
    }

    /// Add a line from current position to (x, y).
    ///
    /// # Arguments
    ///
    /// * `x, y` - End point in 26.6 fixed-point format
    pub fn line_to(&mut self, x: F26Dot6, y: F26Dot6) {
        self.add_line(self.current_x, self.current_y, x, y);
        self.current_x = x;
        self.current_y = y;
    }

    /// Add a quadratic Bézier curve from current position.
    ///
    /// # Arguments
    ///
    /// * `x1, y1` - Control point in 26.6 fixed-point format
    /// * `x2, y2` - End point in 26.6 fixed-point format
    pub fn quadratic_to(&mut self, x1: F26Dot6, y1: F26Dot6, x2: F26Dot6, y2: F26Dot6) {
        let x0 = self.current_x;
        let y0 = self.current_y;

        // Subdivide curve and add line segments
        subdivide_quadratic(
            x0,
            y0,
            x1,
            y1,
            x2,
            y2,
            &mut |x, y| {
                self.add_line(self.current_x, self.current_y, x, y);
                self.current_x = x;
                self.current_y = y;
            },
            0,
        );
    }

    /// Add a cubic Bézier curve from current position.
    ///
    /// # Arguments
    ///
    /// * `x1, y1` - First control point in 26.6 fixed-point format
    /// * `x2, y2` - Second control point in 26.6 fixed-point format
    /// * `x3, y3` - End point in 26.6 fixed-point format
    pub fn cubic_to(
        &mut self,
        x1: F26Dot6,
        y1: F26Dot6,
        x2: F26Dot6,
        y2: F26Dot6,
        x3: F26Dot6,
        y3: F26Dot6,
    ) {
        let x0 = self.current_x;
        let y0 = self.current_y;

        // Subdivide curve and add line segments
        subdivide_cubic(
            x0,
            y0,
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
            &mut |x, y| {
                self.add_line(self.current_x, self.current_y, x, y);
                self.current_x = x;
                self.current_y = y;
            },
            0,
        );
    }

    /// Close current contour (line back to start).
    pub fn close(&mut self) {
        if self.current_x != self.contour_start_x || self.current_y != self.contour_start_y {
            self.add_line(
                self.current_x,
                self.current_y,
                self.contour_start_x,
                self.contour_start_y,
            );
        }
        self.current_x = self.contour_start_x;
        self.current_y = self.contour_start_y;
    }

    /// Add a line segment to the edge table.
    ///
    /// # Arguments
    ///
    /// * `x1, y1` - Start point in 26.6 fixed-point format
    /// * `x2, y2` - End point in 26.6 fixed-point format
    fn add_line(&mut self, x1: F26Dot6, y1: F26Dot6, x2: F26Dot6, y2: F26Dot6) {
        // Create edge (returns None for horizontal lines)
        // Edge::new() normalizes so that y_start < y_end
        if let Some(edge) = Edge::new(x1, y1, x2, y2) {
            // The edge needs to know its starting Y for insertion into edge table
            // After normalization, y_min is the starting scanline
            let y_min = y1.to_int().min(y2.to_int());

            // Clamp to bitmap bounds
            let y_start = y_min.max(0).min(self.height as i32 - 1) as usize;

            // Add edge to the edge table at its starting scanline
            if y_start < self.edge_table.len() {
                self.edge_table[y_start].push(edge);
            }
        }
    }

    /// Render outline to monochrome bitmap.
    ///
    /// # Arguments
    ///
    /// * `bitmap` - Output buffer (width * height bytes, 1 = black, 0 = white)
    ///
    /// # Panics
    ///
    /// Panics if bitmap.len() != width * height
    pub fn render_mono(&mut self, bitmap: &mut [u8]) {
        assert_eq!(
            bitmap.len(),
            self.width * self.height,
            "Bitmap size mismatch"
        );

        // Fill with white (0)
        bitmap.fill(0);

        // Scanline loop
        for y in 0..self.height {
            self.scan_line_mono(y as i32, bitmap);
        }
    }

    /// Process one scanline for monochrome rendering.
    fn scan_line_mono(&mut self, y: i32, bitmap: &mut [u8]) {
        if y < 0 || y >= self.height as i32 {
            return;
        }

        // Activate edges from edge table for this scanline
        if (y as usize) < self.edge_table.len() {
            self.active_edges.extend(&self.edge_table[y as usize]);
        }

        // Remove inactive edges (y > y_max)
        self.active_edges.remove_inactive(y);

        // Sort by X coordinate
        self.active_edges.sort_by_x();

        // Fill spans based on fill rule
        match self.fill_rule {
            FillRule::NonZeroWinding => self.fill_nonzero_winding(y, bitmap),
            FillRule::EvenOdd => self.fill_even_odd(y, bitmap),
        }

        // Step all edges to next scanline
        self.active_edges.step_all();
    }

    /// Fill spans using non-zero winding rule.
    fn fill_nonzero_winding(&self, y: i32, bitmap: &mut [u8]) {
        let mut winding = 0i32;
        let mut fill_start: Option<i32> = None;

        for edge in self.active_edges.iter() {
            let x = edge.x.to_int();
            let old_winding = winding;

            // Update winding number
            winding += edge.direction as i32;

            // Check for transitions
            if old_winding == 0 && winding != 0 {
                // Start fill span (entering filled region)
                fill_start = Some(x);
            } else if old_winding != 0 && winding == 0 {
                // End fill span (leaving filled region)
                if let Some(start) = fill_start {
                    self.fill_span(start, x, y, bitmap);
                    fill_start = None;
                }
            }
        }
    }

    /// Fill spans using even-odd rule.
    fn fill_even_odd(&self, y: i32, bitmap: &mut [u8]) {
        let mut inside = false;
        let mut fill_start = 0i32;

        for edge in self.active_edges.iter() {
            let x = edge.x.to_int();

            if inside {
                // End span
                self.fill_span(fill_start, x, y, bitmap);
                inside = false;
            } else {
                // Start span
                fill_start = x;
                inside = true;
            }
        }
    }

    /// Fill a horizontal span of pixels.
    fn fill_span(&self, x1: i32, x2: i32, y: i32, bitmap: &mut [u8]) {
        if y < 0 || y >= self.height as i32 {
            return;
        }

        let x_start = x1.max(0).min(self.width as i32) as usize;
        let x_end = x2.max(0).min(self.width as i32) as usize;

        let row_offset = y as usize * self.width;
        for x in x_start..x_end {
            bitmap[row_offset + x] = 1; // Black
        }
    }
}

/// Implement skrifa OutlinePen for ScanConverter.
///
/// This allows direct rendering of font outlines from skrifa.
/// Coordinates are expected in pixels (already scaled from font units).
impl OutlinePen for ScanConverter {
    fn move_to(&mut self, x: f32, y: f32) {
        // Y-flip: font space (Y-up) → graphics space (Y-down)
        let y_flipped = self.height as f32 - y;
        self.move_to(F26Dot6::from_float(x), F26Dot6::from_float(y_flipped));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let y_flipped = self.height as f32 - y;
        self.line_to(F26Dot6::from_float(x), F26Dot6::from_float(y_flipped));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let y1_flipped = self.height as f32 - y1;
        let y_flipped = self.height as f32 - y;
        self.quadratic_to(
            F26Dot6::from_float(x1),
            F26Dot6::from_float(y1_flipped),
            F26Dot6::from_float(x),
            F26Dot6::from_float(y_flipped),
        );
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let y1_flipped = self.height as f32 - y1;
        let y2_flipped = self.height as f32 - y2;
        let y_flipped = self.height as f32 - y;
        self.cubic_to(
            F26Dot6::from_float(x1),
            F26Dot6::from_float(y1_flipped),
            F26Dot6::from_float(x2),
            F26Dot6::from_float(y2_flipped),
            F26Dot6::from_float(x),
            F26Dot6::from_float(y_flipped),
        );
    }

    fn close(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_converter_new() {
        let sc = ScanConverter::new(64, 64);
        assert_eq!(sc.width, 64);
        assert_eq!(sc.height, 64);
        assert_eq!(sc.fill_rule(), FillRule::NonZeroWinding);
        assert_eq!(sc.dropout_mode(), DropoutMode::None);
    }

    #[test]
    fn test_scan_converter_set_fill_rule() {
        let mut sc = ScanConverter::new(64, 64);
        sc.set_fill_rule(FillRule::EvenOdd);
        assert_eq!(sc.fill_rule(), FillRule::EvenOdd);
    }

    #[test]
    fn test_scan_converter_set_dropout_mode() {
        let mut sc = ScanConverter::new(64, 64);
        sc.set_dropout_mode(DropoutMode::Simple);
        assert_eq!(sc.dropout_mode(), DropoutMode::Simple);
    }

    #[test]
    fn test_scan_converter_move_to() {
        let mut sc = ScanConverter::new(64, 64);
        sc.move_to(F26Dot6::from_int(10), F26Dot6::from_int(20));
        assert_eq!(sc.current_x.to_int(), 10);
        assert_eq!(sc.current_y.to_int(), 20);
        assert_eq!(sc.contour_start_x.to_int(), 10);
        assert_eq!(sc.contour_start_y.to_int(), 20);
    }

    #[test]
    fn test_scan_converter_line_to() {
        let mut sc = ScanConverter::new(64, 64);
        sc.move_to(F26Dot6::from_int(0), F26Dot6::from_int(0));
        sc.line_to(F26Dot6::from_int(10), F26Dot6::from_int(10));
        assert_eq!(sc.current_x.to_int(), 10);
        assert_eq!(sc.current_y.to_int(), 10);
    }

    #[test]
    fn test_render_simple_rectangle() {
        let mut sc = ScanConverter::new(10, 10);

        // Draw rectangle from (2,2) to (8,8)
        sc.move_to(F26Dot6::from_int(2), F26Dot6::from_int(2));
        sc.line_to(F26Dot6::from_int(8), F26Dot6::from_int(2));
        sc.line_to(F26Dot6::from_int(8), F26Dot6::from_int(8));
        sc.line_to(F26Dot6::from_int(2), F26Dot6::from_int(8));
        sc.close();

        let mut bitmap = vec![0u8; 100];
        sc.render_mono(&mut bitmap);

        // Check that interior pixels are filled
        // Row 4 (middle), columns 2-7 should be black (1)
        for x in 2..8 {
            assert_eq!(bitmap[4 * 10 + x], 1, "Pixel ({}, 4) should be black", x);
        }

        // Check that exterior pixels are white (0)
        assert_eq!(bitmap[4 * 10 + 0], 0, "Pixel (0, 4) should be white");
        assert_eq!(bitmap[4 * 10 + 9], 0, "Pixel (9, 4) should be white");
    }

    #[test]
    fn test_render_triangle() {
        let mut sc = ScanConverter::new(20, 20);

        // Draw larger triangle: (5,5) -> (15,5) -> (10,15)
        sc.move_to(F26Dot6::from_int(5), F26Dot6::from_int(5));
        sc.line_to(F26Dot6::from_int(15), F26Dot6::from_int(5));
        sc.line_to(F26Dot6::from_int(10), F26Dot6::from_int(15));
        sc.close();

        let mut bitmap = vec![0u8; 400];
        sc.render_mono(&mut bitmap);

        // Count total filled pixels
        let filled_count: usize = bitmap.iter().filter(|&&p| p == 1).count();

        // Triangle should have filled pixels
        // Area ~ 0.5 * base * height = 0.5 * 10 * 10 = 50 pixels
        assert!(
            filled_count > 20,
            "Triangle should have filled pixels (got {})",
            filled_count
        );
    }

    #[test]
    fn test_even_odd_fill_rule() {
        let mut sc = ScanConverter::new(10, 10);
        sc.set_fill_rule(FillRule::EvenOdd);

        // Simple rectangle
        sc.move_to(F26Dot6::from_int(2), F26Dot6::from_int(2));
        sc.line_to(F26Dot6::from_int(8), F26Dot6::from_int(2));
        sc.line_to(F26Dot6::from_int(8), F26Dot6::from_int(8));
        sc.line_to(F26Dot6::from_int(2), F26Dot6::from_int(8));
        sc.close();

        let mut bitmap = vec![0u8; 100];
        sc.render_mono(&mut bitmap);

        // Should fill interior
        assert_eq!(bitmap[5 * 10 + 5], 1, "Center should be filled");
    }

    #[test]
    fn test_reset() {
        let mut sc = ScanConverter::new(10, 10);

        sc.move_to(F26Dot6::from_int(5), F26Dot6::from_int(5));
        sc.line_to(F26Dot6::from_int(10), F26Dot6::from_int(10));

        sc.reset();

        assert_eq!(sc.current_x.to_int(), 0);
        assert_eq!(sc.current_y.to_int(), 0);
    }
}
