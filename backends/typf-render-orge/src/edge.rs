// this_file: backends/typf-orge/src/edge.rs

//! Edge list management for scan conversion.
//!
//! An edge represents a line segment in the glyph outline, stored in a format
//! optimized for scanline rasterization.

use crate::fixed::F26Dot6;

/// A single edge in the glyph outline.
///
/// Represents a line segment from (x, y_min) to (x + dx, y_max) where dx is
/// computed incrementally as we scan down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    /// Current X coordinate (at current scanline).
    pub x: F26Dot6,

    /// X increment per scanline (dx/dy slope).
    pub x_increment: F26Dot6,

    /// Winding direction: +1 for upward, -1 for downward.
    pub direction: i8,

    /// Maximum Y coordinate (inclusive) where this edge is active.
    pub y_max: i32,

    /// Minimum Y coordinate (inclusive) where this edge is active.
    pub y_min: i32,
}

impl Edge {
    /// Create a new edge from two points.
    ///
    /// Returns `None` if the edge is horizontal (y1 == y2) or doesn't cross any scanlines.
    ///
    /// # Arguments
    ///
    /// * `x1, y1` - Starting point (in F26Dot6 format)
    /// * `x2, y2` - Ending point (in F26Dot6 format)
    ///
    /// # Returns
    ///
    /// `Some(Edge)` if valid, `None` otherwise
    pub fn new(x1: F26Dot6, y1: F26Dot6, x2: F26Dot6, y2: F26Dot6) -> Option<Self> {
        let dy = y2 - y1;

        if dy == F26Dot6::ZERO {
            return None;
        }

        // Sort by Y so we always scan down
        let (x_start, y_start, x_end, y_end, direction) = if dy > F26Dot6::ZERO {
            (x1, y1, x2, y2, 1i8)
        } else {
            (x2, y2, x1, y1, -1i8)
        };

        let dy_abs = y_end - y_start;
        let dx = x_end - x_start;

        // Calculate slope = dx / dy
        let x_increment = dx.div(dy_abs);

        // Calculate scanline range
        // We use ceil() for start to ensure we are inside the edge
        // We use ceil() - 1 for end to exclude the end point (half-open interval)
        let y_min = y_start.ceil().to_int();
        let y_max = y_end.ceil().to_int() - 1;

        if y_min > y_max {
            return None;
        }

        // Calculate X at the first scanline
        // x = x_start + (y_min - y_start) * slope
        let delta_y = F26Dot6::from_int(y_min) - y_start;
        let x = x_start + delta_y.mul(x_increment);

        Some(Edge {
            x,
            x_increment,
            direction,
            y_max,
            y_min,
        })
    }

    /// Step the edge to the next scanline.
    ///
    /// Updates `x` by adding `x_increment`.
    #[inline]
    pub fn step(&mut self) {
        self.x = self.x + self.x_increment;
    }

    /// Check if edge is active at given Y coordinate.
    #[inline]
    pub fn is_active(&self, y: i32) -> bool {
        y <= self.y_max
    }
}

/// A collection of edges, maintained in sorted order.
///
/// Used for both the edge table (one list per scanline) and the active edge list.
#[derive(Debug, Clone, Default)]
pub struct EdgeList {
    edges: Vec<Edge>,
}

impl EdgeList {
    /// Create a new empty edge list.
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }

    /// Create with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            edges: Vec::with_capacity(capacity),
        }
    }

    /// Insert an edge in sorted order by X coordinate.
    ///
    /// Uses binary search to find insertion point.
    pub fn insert_sorted(&mut self, edge: Edge) {
        let pos = self
            .edges
            .binary_search_by(|e| e.x.cmp(&edge.x))
            .unwrap_or_else(|e| e);
        self.edges.insert(pos, edge);
    }

    /// Sort all edges by X coordinate.
    pub fn sort_by_x(&mut self) {
        self.edges.sort_by(|a, b| a.x.cmp(&b.x));
    }

    /// Remove inactive edges (where y > y_max).
    pub fn remove_inactive(&mut self, y: i32) {
        self.edges.retain(|edge| edge.is_active(y));
    }

    /// Step all edges to next scanline.
    pub fn step_all(&mut self) {
        for edge in &mut self.edges {
            edge.step();
        }
    }

    /// Clear all edges.
    pub fn clear(&mut self) {
        self.edges.clear();
    }

    /// Get number of edges.
    #[inline]
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Get edges as slice.
    #[inline]
    pub fn as_slice(&self) -> &[Edge] {
        &self.edges
    }

    /// Get edges as mutable slice.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [Edge] {
        &mut self.edges
    }

    /// Add an edge without sorting (append).
    pub fn push(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Extend with edges from another list.
    pub fn extend(&mut self, other: &EdgeList) {
        self.edges.extend_from_slice(&other.edges);
    }

    /// Get iterator over edges.
    pub fn iter(&self) -> std::slice::Iter<'_, Edge> {
        self.edges.iter()
    }

    /// Get mutable iterator over edges.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Edge> {
        self.edges.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_new_vertical() {
        let e = Edge::new(
            F26Dot6::from_int(10),
            F26Dot6::from_int(5),
            F26Dot6::from_int(10),
            F26Dot6::from_int(15),
        )
        .unwrap();

        assert_eq!(e.x, F26Dot6::from_int(10));
        assert_eq!(e.x_increment, F26Dot6::ZERO);
        assert_eq!(e.direction, 1);
        assert_eq!(e.y_min, 5);
        assert_eq!(e.y_max, 14); // 15 is excluded
    }

    #[test]
    fn test_edge_new_horizontal_returns_none() {
        let e = Edge::new(
            F26Dot6::from_int(10),
            F26Dot6::from_int(5),
            F26Dot6::from_int(20),
            F26Dot6::from_int(5),
        );

        assert!(e.is_none());
    }

    #[test]
    fn test_edge_new_diagonal() {
        let e = Edge::new(
            F26Dot6::from_int(0),
            F26Dot6::from_int(0),
            F26Dot6::from_int(10),
            F26Dot6::from_int(10),
        )
        .unwrap();

        assert_eq!(e.x, F26Dot6::from_int(0));
        assert_eq!(e.direction, 1);
        assert_eq!(e.y_min, 0);
        assert_eq!(e.y_max, 9); // 10 is excluded
                                // Slope = 10/10 = 1.0 in 26.6 = 64
        assert_eq!(e.x_increment.raw(), 64);
    }

    #[test]
    fn test_edge_new_swaps_if_downward() {
        let e = Edge::new(
            F26Dot6::from_int(20),
            F26Dot6::from_int(15),
            F26Dot6::from_int(10),
            F26Dot6::from_int(5),
        )
        .unwrap();

        // Should be swapped so y1 < y2
        assert_eq!(e.x, F26Dot6::from_int(10));
        assert_eq!(e.y_min, 5);
        assert_eq!(e.y_max, 14); // 15 is excluded
        assert_eq!(e.direction, -1);
    }

    #[test]
    fn test_edge_step() {
        let mut e = Edge::new(
            F26Dot6::from_int(0),
            F26Dot6::from_int(0),
            F26Dot6::from_int(10),
            F26Dot6::from_int(10),
        )
        .unwrap();

        assert_eq!(e.x.to_int(), 0);
        e.step();
        assert_eq!(e.x.to_int(), 1);
        e.step();
        assert_eq!(e.x.to_int(), 2);
    }

    #[test]
    fn test_edge_is_active() {
        let e = Edge::new(
            F26Dot6::from_int(0),
            F26Dot6::from_int(5),
            F26Dot6::from_int(10),
            F26Dot6::from_int(15),
        )
        .unwrap();

        assert!(e.is_active(5));
        assert!(e.is_active(10));
        assert!(e.is_active(14));
        assert!(!e.is_active(15));
    }

    #[test]
    fn test_edgelist_new() {
        let list = EdgeList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_edgelist_push() {
        let mut list = EdgeList::new();
        let e = Edge::new(
            F26Dot6::from_int(10),
            F26Dot6::from_int(0),
            F26Dot6::from_int(10),
            F26Dot6::from_int(10),
        )
        .unwrap();

        list.push(e);
        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());
    }

    #[test]
    fn test_edgelist_insert_sorted() {
        let mut list = EdgeList::new();

        list.insert_sorted(
            Edge::new(
                F26Dot6::from_int(20),
                F26Dot6::ZERO,
                F26Dot6::from_int(20),
                F26Dot6::from_int(10),
            )
            .unwrap(),
        );

        list.insert_sorted(
            Edge::new(
                F26Dot6::from_int(10),
                F26Dot6::ZERO,
                F26Dot6::from_int(10),
                F26Dot6::from_int(10),
            )
            .unwrap(),
        );

        list.insert_sorted(
            Edge::new(
                F26Dot6::from_int(15),
                F26Dot6::ZERO,
                F26Dot6::from_int(15),
                F26Dot6::from_int(10),
            )
            .unwrap(),
        );

        assert_eq!(list.len(), 3);
        assert_eq!(list.as_slice()[0].x.to_int(), 10);
        assert_eq!(list.as_slice()[1].x.to_int(), 15);
        assert_eq!(list.as_slice()[2].x.to_int(), 20);
    }

    #[test]
    fn test_edgelist_sort_by_x() {
        let mut list = EdgeList::new();

        list.push(
            Edge::new(
                F26Dot6::from_int(20),
                F26Dot6::ZERO,
                F26Dot6::from_int(20),
                F26Dot6::from_int(10),
            )
            .unwrap(),
        );

        list.push(
            Edge::new(
                F26Dot6::from_int(10),
                F26Dot6::ZERO,
                F26Dot6::from_int(10),
                F26Dot6::from_int(10),
            )
            .unwrap(),
        );

        list.sort_by_x();

        assert_eq!(list.as_slice()[0].x.to_int(), 10);
        assert_eq!(list.as_slice()[1].x.to_int(), 20);
    }

    #[test]
    fn test_edgelist_remove_inactive() {
        let mut list = EdgeList::new();

        list.push(
            Edge::new(
                F26Dot6::from_int(10),
                F26Dot6::ZERO,
                F26Dot6::from_int(10),
                F26Dot6::from_int(5),
            )
            .unwrap(),
        );

        list.push(
            Edge::new(
                F26Dot6::from_int(20),
                F26Dot6::ZERO,
                F26Dot6::from_int(20),
                F26Dot6::from_int(10),
            )
            .unwrap(),
        );

        assert_eq!(list.len(), 2);

        list.remove_inactive(6);
        assert_eq!(list.len(), 1);
        assert_eq!(list.as_slice()[0].x.to_int(), 20);
    }

    #[test]
    fn test_edgelist_step_all() {
        let mut list = EdgeList::new();

        list.push(
            Edge::new(
                F26Dot6::from_int(0),
                F26Dot6::from_int(0),
                F26Dot6::from_int(10),
                F26Dot6::from_int(10),
            )
            .unwrap(),
        );

        list.push(
            Edge::new(
                F26Dot6::from_int(0),
                F26Dot6::from_int(0),
                F26Dot6::from_int(20),
                F26Dot6::from_int(10),
            )
            .unwrap(),
        );

        assert_eq!(list.as_slice()[0].x.to_int(), 0);
        assert_eq!(list.as_slice()[1].x.to_int(), 0);

        list.step_all();

        assert_eq!(list.as_slice()[0].x.to_int(), 1);
        assert_eq!(list.as_slice()[1].x.to_int(), 2);
    }

    #[test]
    fn test_edgelist_clear() {
        let mut list = EdgeList::new();
        list.push(
            Edge::new(
                F26Dot6::from_int(10),
                F26Dot6::ZERO,
                F26Dot6::from_int(10),
                F26Dot6::from_int(10),
            )
            .unwrap(),
        );

        assert!(!list.is_empty());
        list.clear();
        assert!(list.is_empty());
    }

    #[test]
    fn test_edgelist_extend() {
        let mut list1 = EdgeList::new();
        let mut list2 = EdgeList::new();

        list1.push(
            Edge::new(
                F26Dot6::from_int(10),
                F26Dot6::ZERO,
                F26Dot6::from_int(10),
                F26Dot6::from_int(10),
            )
            .unwrap(),
        );

        list2.push(
            Edge::new(
                F26Dot6::from_int(20),
                F26Dot6::ZERO,
                F26Dot6::from_int(20),
                F26Dot6::from_int(10),
            )
            .unwrap(),
        );

        list1.extend(&list2);
        assert_eq!(list1.len(), 2);
    }

    #[test]
    fn test_edge_fractional_slope() {
        // Edge with slope 0.5 (moves 5 pixels over 10 scanlines)
        let e = Edge::new(
            F26Dot6::from_int(0),
            F26Dot6::from_int(0),
            F26Dot6::from_int(5),
            F26Dot6::from_int(10),
        )
        .unwrap();

        // Slope = 5/10 = 0.5, in 26.6 format = 32
        assert_eq!(e.x_increment.raw(), 32);

        let mut x = e.x;
        // After 2 steps, should be at x=1
        x = x + e.x_increment;
        x = x + e.x_increment;
        assert_eq!(x.to_int(), 1);
    }

    #[test]
    fn test_edge_negative_slope() {
        // Edge going left-down (negative slope)
        let e = Edge::new(
            F26Dot6::from_int(10),
            F26Dot6::from_int(0),
            F26Dot6::from_int(0),
            F26Dot6::from_int(10),
        )
        .unwrap();

        // Slope = -10/10 = -1.0, in 26.6 format = -64
        assert_eq!(e.x_increment.raw(), -64);

        let mut test_e = e;
        assert_eq!(test_e.x.to_int(), 10);
        test_e.step();
        assert_eq!(test_e.x.to_int(), 9);
    }

    #[test]
    fn test_edgelist_with_capacity() {
        let list = EdgeList::with_capacity(100);
        assert_eq!(list.len(), 0);
        assert!(list.edges.capacity() >= 100);
    }

    #[test]
    fn test_edge_winding_direction() {
        // Upward edge (y1 < y2)
        let e_up = Edge::new(
            F26Dot6::from_int(0),
            F26Dot6::from_int(0),
            F26Dot6::from_int(10),
            F26Dot6::from_int(10),
        )
        .unwrap();
        assert_eq!(e_up.direction, 1);

        // Downward edge (y1 > y2)
        let e_down = Edge::new(
            F26Dot6::from_int(10),
            F26Dot6::from_int(10),
            F26Dot6::from_int(0),
            F26Dot6::from_int(0),
        )
        .unwrap();
        assert_eq!(e_down.direction, -1);
    }

    #[test]
    fn test_edgelist_iter() {
        let mut list = EdgeList::new();
        list.push(
            Edge::new(
                F26Dot6::from_int(10),
                F26Dot6::ZERO,
                F26Dot6::from_int(10),
                F26Dot6::from_int(10),
            )
            .unwrap(),
        );

        let count = list.iter().count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_edgelist_iter_mut() {
        let mut list = EdgeList::new();
        list.push(
            Edge::new(
                F26Dot6::from_int(10),
                F26Dot6::ZERO,
                F26Dot6::from_int(10),
                F26Dot6::from_int(10),
            )
            .unwrap(),
        );

        for edge in list.iter_mut() {
            edge.step();
        }

        // No actual change since x_increment is 0 for vertical edge
        assert_eq!(list.as_slice()[0].x.to_int(), 10);
    }
}
