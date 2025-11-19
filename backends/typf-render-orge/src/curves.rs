// this_file: backends/typf-orge/src/curves.rs

//! Bézier curve linearization via subdivision.
//!
//! Implements de Casteljau subdivision for quadratic and cubic Bézier curves.
//! Curves are recursively subdivided until they are "flat enough" for line rendering.

use crate::fixed::F26Dot6;

/// Flatness threshold in 26.6 fixed-point format.
///
/// A curve is considered "flat" if the control points are within this distance
/// from the line connecting the endpoints. Value of 4 = 4/64 pixels = 1/16 pixel.
pub const FLATNESS_THRESHOLD: F26Dot6 = F26Dot6::from_raw(4);

/// Maximum subdivision depth to prevent infinite recursion.
const MAX_DEPTH: u32 = 16;

/// Compute flatness metric for quadratic Bézier curve.
///
/// Returns the maximum perpendicular distance from the control point to the
/// line connecting the endpoints.
///
/// # Arguments
///
/// * `x0, y0` - Start point
/// * `x1, y1` - Control point
/// * `x2, y2` - End point
pub fn compute_quadratic_flatness(
    x0: F26Dot6,
    y0: F26Dot6,
    x1: F26Dot6,
    y1: F26Dot6,
    x2: F26Dot6,
    y2: F26Dot6,
) -> F26Dot6 {
    // Distance from control point to line = |ax + by + c| / sqrt(a² + b²)
    // For simplicity, use Manhattan distance approximation:
    // flatness ≈ |x1 - (x0+x2)/2| + |y1 - (y0+y2)/2|

    let mid_x = (x0 + x2).raw() / 2;
    let mid_y = (y0 + y2).raw() / 2;

    let dx = (x1.raw() - mid_x).abs();
    let dy = (y1.raw() - mid_y).abs();

    F26Dot6::from_raw(dx + dy)
}

/// Subdivide quadratic Bézier curve and output line segments.
///
/// Uses de Casteljau subdivision algorithm. Recursively subdivides the curve
/// until it is flat enough, then outputs a line segment.
///
/// # Arguments
///
/// * `x0, y0` - Start point
/// * `x1, y1` - Control point
/// * `x2, y2` - End point
/// * `output` - Callback for line segments: `(x_end, y_end)`
/// * `depth` - Current recursion depth
#[allow(clippy::too_many_arguments)]
pub fn subdivide_quadratic<F>(
    x0: F26Dot6,
    y0: F26Dot6,
    x1: F26Dot6,
    y1: F26Dot6,
    x2: F26Dot6,
    y2: F26Dot6,
    output: &mut F,
    depth: u32,
) where
    F: FnMut(F26Dot6, F26Dot6),
{
    // Check termination conditions
    if depth >= MAX_DEPTH {
        // Max depth reached - output line to endpoint
        output(x2, y2);
        return;
    }

    let flatness = compute_quadratic_flatness(x0, y0, x1, y1, x2, y2);
    if flatness <= FLATNESS_THRESHOLD {
        // Curve is flat enough - output line to endpoint
        output(x2, y2);
        return;
    }

    // Subdivide using de Casteljau algorithm
    // Midpoints:
    //   m01 = (p0 + p1) / 2
    //   m12 = (p1 + p2) / 2
    //   m012 = (m01 + m12) / 2  (point on curve at t=0.5)

    let m01_x = F26Dot6::from_raw((x0.raw() + x1.raw()) / 2);
    let m01_y = F26Dot6::from_raw((y0.raw() + y1.raw()) / 2);

    let m12_x = F26Dot6::from_raw((x1.raw() + x2.raw()) / 2);
    let m12_y = F26Dot6::from_raw((y1.raw() + y2.raw()) / 2);

    let m012_x = F26Dot6::from_raw((m01_x.raw() + m12_x.raw()) / 2);
    let m012_y = F26Dot6::from_raw((m01_y.raw() + m12_y.raw()) / 2);

    // Recursively subdivide both halves
    subdivide_quadratic(x0, y0, m01_x, m01_y, m012_x, m012_y, output, depth + 1);
    subdivide_quadratic(m012_x, m012_y, m12_x, m12_y, x2, y2, output, depth + 1);
}

/// Compute flatness metric for cubic Bézier curve.
///
/// Returns approximate maximum perpendicular distance from control points to
/// the line connecting the endpoints.
///
/// # Arguments
///
/// * `x0, y0` - Start point
/// * `x1, y1` - First control point
/// * `x2, y2` - Second control point
/// * `x3, y3` - End point
#[allow(clippy::too_many_arguments)]
pub fn compute_cubic_flatness(
    x0: F26Dot6,
    y0: F26Dot6,
    x1: F26Dot6,
    y1: F26Dot6,
    x2: F26Dot6,
    y2: F26Dot6,
    x3: F26Dot6,
    y3: F26Dot6,
) -> F26Dot6 {
    // Similar to quadratic, use Manhattan distance from control points
    // to the line connecting endpoints

    // Check distance from both control points
    let dx1 = (x1.raw() - (x0.raw() + x3.raw()) / 2).abs();
    let dy1 = (y1.raw() - (y0.raw() + y3.raw()) / 2).abs();

    let dx2 = (x2.raw() - (x0.raw() + x3.raw()) / 2).abs();
    let dy2 = (y2.raw() - (y0.raw() + y3.raw()) / 2).abs();

    let dist1 = dx1 + dy1;
    let dist2 = dx2 + dy2;

    F26Dot6::from_raw(dist1.max(dist2))
}

/// Subdivide cubic Bézier curve and output line segments.
///
/// Uses de Casteljau subdivision algorithm. Recursively subdivides the curve
/// until it is flat enough, then outputs a line segment.
///
/// # Arguments
///
/// * `x0, y0` - Start point
/// * `x1, y1` - First control point
/// * `x2, y2` - Second control point
/// * `x3, y3` - End point
/// * `output` - Callback for line segments: `(x_end, y_end)`
/// * `depth` - Current recursion depth
#[allow(clippy::too_many_arguments)]
pub fn subdivide_cubic<F>(
    x0: F26Dot6,
    y0: F26Dot6,
    x1: F26Dot6,
    y1: F26Dot6,
    x2: F26Dot6,
    y2: F26Dot6,
    x3: F26Dot6,
    y3: F26Dot6,
    output: &mut F,
    depth: u32,
) where
    F: FnMut(F26Dot6, F26Dot6),
{
    // Check termination conditions
    if depth >= MAX_DEPTH {
        // Max depth reached - output line to endpoint
        output(x3, y3);
        return;
    }

    let flatness = compute_cubic_flatness(x0, y0, x1, y1, x2, y2, x3, y3);
    if flatness <= FLATNESS_THRESHOLD {
        // Curve is flat enough - output line to endpoint
        output(x3, y3);
        return;
    }

    // Subdivide using de Casteljau algorithm
    // First level midpoints:
    let m01_x = F26Dot6::from_raw((x0.raw() + x1.raw()) / 2);
    let m01_y = F26Dot6::from_raw((y0.raw() + y1.raw()) / 2);

    let m12_x = F26Dot6::from_raw((x1.raw() + x2.raw()) / 2);
    let m12_y = F26Dot6::from_raw((y1.raw() + y2.raw()) / 2);

    let m23_x = F26Dot6::from_raw((x2.raw() + x3.raw()) / 2);
    let m23_y = F26Dot6::from_raw((y2.raw() + y3.raw()) / 2);

    // Second level midpoints:
    let m012_x = F26Dot6::from_raw((m01_x.raw() + m12_x.raw()) / 2);
    let m012_y = F26Dot6::from_raw((m01_y.raw() + m12_y.raw()) / 2);

    let m123_x = F26Dot6::from_raw((m12_x.raw() + m23_x.raw()) / 2);
    let m123_y = F26Dot6::from_raw((m12_y.raw() + m23_y.raw()) / 2);

    // Third level midpoint (point on curve at t=0.5):
    let m0123_x = F26Dot6::from_raw((m012_x.raw() + m123_x.raw()) / 2);
    let m0123_y = F26Dot6::from_raw((m012_y.raw() + m123_y.raw()) / 2);

    // Recursively subdivide both halves
    subdivide_cubic(x0, y0, m01_x, m01_y, m012_x, m012_y, m0123_x, m0123_y, output, depth + 1);
    subdivide_cubic(m0123_x, m0123_y, m123_x, m123_y, m23_x, m23_y, x3, y3, output, depth + 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadratic_flatness_straight_line() {
        // Straight line: control point on the line
        let flatness = compute_quadratic_flatness(
            F26Dot6::from_int(0),
            F26Dot6::from_int(0),
            F26Dot6::from_int(5),
            F26Dot6::from_int(5),
            F26Dot6::from_int(10),
            F26Dot6::from_int(10),
        );

        // Should be very flat
        assert!(flatness.raw() < FLATNESS_THRESHOLD.raw());
    }

    #[test]
    fn test_quadratic_subdivision() {
        let mut points = Vec::new();

        subdivide_quadratic(
            F26Dot6::from_int(0),
            F26Dot6::from_int(0),
            F26Dot6::from_int(5),
            F26Dot6::from_int(10),
            F26Dot6::from_int(10),
            F26Dot6::from_int(0),
            &mut |x, y| points.push((x.to_int(), y.to_int())),
            0,
        );

        // Should produce multiple line segments
        assert!(points.len() > 1, "Curve should be subdivided");

        // Last point should be the endpoint
        assert_eq!(points.last().unwrap(), &(10, 0));
    }

    #[test]
    fn test_cubic_flatness_curved() {
        // Curved cubic - control points off the line
        let flatness = compute_cubic_flatness(
            F26Dot6::from_int(0),
            F26Dot6::from_int(0),
            F26Dot6::from_int(0),
            F26Dot6::from_int(20),
            F26Dot6::from_int(10),
            F26Dot6::from_int(20),
            F26Dot6::from_int(10),
            F26Dot6::from_int(0),
        );

        // Should detect curvature (flatness > threshold)
        assert!(flatness.raw() > FLATNESS_THRESHOLD.raw());
    }

    #[test]
    fn test_cubic_subdivision() {
        let mut points = Vec::new();

        subdivide_cubic(
            F26Dot6::from_int(0),
            F26Dot6::from_int(0),
            F26Dot6::from_int(0),
            F26Dot6::from_int(10),
            F26Dot6::from_int(10),
            F26Dot6::from_int(10),
            F26Dot6::from_int(10),
            F26Dot6::from_int(0),
            &mut |x, y| points.push((x.to_int(), y.to_int())),
            0,
        );

        // Should produce multiple line segments
        assert!(points.len() > 1, "Curve should be subdivided");

        // Last point should be the endpoint
        assert_eq!(points.last().unwrap(), &(10, 0));
    }

    #[test]
    fn test_max_depth_termination() {
        let mut points = Vec::new();

        // Create a highly curved quadratic
        subdivide_quadratic(
            F26Dot6::from_int(0),
            F26Dot6::from_int(0),
            F26Dot6::from_int(100),
            F26Dot6::from_int(100),
            F26Dot6::from_int(10),
            F26Dot6::from_int(0),
            &mut |x, y| points.push((x, y)),
            0,
        );

        // Should terminate (not infinite loop)
        assert!(points.len() < 10000, "Should terminate subdivision");
    }
}
