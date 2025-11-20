//! The edge detectives: line segments that build glyph foundations
//!
//! Every glyph starts as a collection of edges—straight lines that trace
//! the character's outline. These edges become the roadmap for our rasterizer,
//! telling it exactly which pixels should be filled. Think of them as the
//! scaffolding that holds up beautiful text.

use crate::fixed::F26Dot6;

/// One edge, infinite possibilities
///
/// An edge is more than just a line—it's a promise. It promises that for every
/// scanline it crosses, it knows exactly where to be. We store both its current
/// position and its slope, making rasterization a simple march down the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    /// Where we are right now (X position at current scanline)
    pub x: F26Dot6,

    /// How we move each step down (slope in X per scanline)
    pub x_increment: F26Dot6,

    /// Which way we're going (upward fills vs downward carves)
    pub direction: i8,

    /// The last scanline where we matter (inclusive)
    pub y_max: i32,

    /// The first scanline where we appear (inclusive)
    pub y_min: i32,
}

impl Edge {
    /// Birth of an edge: two points become a rasterization warrior
    ///
    /// We transform any two points into a scan-ready edge. Horizontal edges get
    /// discarded (they don't cross scanlines), and everything else gets sorted
    /// so we always scan from top to bottom. The result is an edge that knows
    /// its purpose from the moment it's created.
    ///
    /// # The Point Partnership
    ///
    /// * `x1, y1` - Where the edge begins its journey
    /// * `x2, y2` - Where the edge completes its mission
    ///
    /// # Returns
    ///
    /// A battle-ready edge, or `None` if the points refuse to cooperate
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

    /// One step closer to completion
    ///
    /// Each scanline brings us one step down the edge. We simply add our slope
    /// to our current position—no complex math, just elegant progression.
    #[inline]
    pub fn step(&mut self) {
        self.x = self.x + self.x_increment;
    }

    /// Are we still relevant at this height?
    ///
    /// An edge knows when it's time to retire. Once we've passed our maximum
    /// Y coordinate, we're done contributing to the glyph.
    #[inline]
    pub fn is_active(&self, y: i32) -> bool {
        y <= self.y_max
    }
}

/// The edge orchestra: many lines working in harmony
///
/// Managing one edge is simple, but glyphs have hundreds. This collection
/// keeps them sorted by X position, making scanline traversal efficient.
/// Whether we're building the global edge table or managing active edges,
/// this structure ensures every edge finds its place.
#[derive(Debug, Clone, Default)]
pub struct EdgeList {
    edges: Vec<Edge>,
}

impl EdgeList {
    /// Start fresh: a clean slate for new edges
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }

    /// Room for everyone: pre-allocate space for efficiency
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            edges: Vec::with_capacity(capacity),
        }
    }

    /// Find the perfect spot: insert while maintaining order
    ///
    /// We use binary search to locate exactly where each edge belongs.
    /// No messy linear searches—just surgical precision.
    pub fn insert_sorted(&mut self, edge: Edge) {
        let pos = self
            .edges
            .binary_search_by(|e| e.x.cmp(&edge.x))
            .unwrap_or_else(|e| e);
        self.edges.insert(pos, edge);
    }

    /// Restore order: sort everyone by their X position
    pub fn sort_by_x(&mut self) {
        self.edges.sort_by(|a, b| a.x.cmp(&b.x));
    }

    /// Spring cleaning: remove edges that have finished their journey
    pub fn remove_inactive(&mut self, y: i32) {
        self.edges.retain(|edge| edge.is_active(y));
    }

    /// March together: advance every edge one scanline down
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
