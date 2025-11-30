//! Taming Bézier curves, one subdivision at a time
//!
//! Font outlines are beautiful curves, but computers think in straight lines.
//! We bridge that gap using de Casteljau's subdivision algorithm—splitting curves
//! until they're practically straight. The result? Smooth glyphs rendered with
//! mathematical precision.

use crate::fixed::F26Dot6;

/// When good enough is perfect: our flatness tolerance
///
/// A curve gets the "flat enough" stamp when its control point strays less than
/// 1/16th of a pixel from the straight line between endpoints. At that point,
/// the human eye can't tell the difference—time to stop subdividing.
pub const FLATNESS_THRESHOLD: F26Dot6 = F26Dot6::from_raw(4);

/// The recursion guardrail that prevents infinite loops
///
/// Even the most complex curves surrender after 16 subdivisions. This safety
/// net protects us from malformed font data that could otherwise send us
/// spinning forever.
const MAX_DEPTH: u32 = 16;

/// How rebellious is your quadratic curve?
///
/// We measure flatness by checking how far the control point strays from the
/// straight line between start and end. The farther it wanders, the more
/// subdivision we'll need to tame it.
///
/// # The Geometry Dance
///
/// * `x0, y0` - Where the curve begins its journey
/// * `x1, y1` - The control point that pulls the curve off course
/// * `x2, y2` - Where the curve finally settles
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

/// The curve whisperer: transforms rebellious Béziers into obedient lines
///
/// De Casteljau's algorithm is our secret weapon. We keep splitting curves
/// in half until they surrender and become straight lines. Each split creates
/// two better-behaved curves that are easier to render.
///
/// # The Subdivision Spell
///
/// * `x0, y0` - Where we start drawing
/// * `x1, y1` - The curve's influence point
/// * `x2, y2` - Where we end up
/// * `output` - Where we send each conquered line segment
/// * `depth` - How many times we've cast this spell
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

/// Cubic curves: double the control points, double the complexity
///
/// With four points to manage instead of three, cubic Béziers can create
/// incredibly smooth shapes. We measure flatness by checking both control
/// points—the farther they drift from the straight path, the more work we have.
///
/// # The Four-Point Symphony
///
/// * `x0, y0` - The opening note
/// * `x1, y1` - First control point's influence
/// * `x2, y2` - Second control point's touch
/// * `x3, y3` - The grand finale
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

/// Taming the wild cubic beast, one split at a time
///
/// Cubic Béziers are the divas of font design—beautiful but demanding.
/// Our de Casteljau subdivision treats them with respect, carefully splitting
/// until they behave like proper straight lines. Each split creates eight new
/// points, but the result is worth it: curves that render perfectly.
///
/// # The Cubic Ballet
///
/// * `x0, y0` - Stage entrance
/// * `x1, y1` - First controller's guidance
/// * `x2, y2` - Second controller's wisdom
/// * `x3, y3` - Graceful exit
/// * `output` - Receives each perfected line segment
/// * `depth` - How deep into the subdivision rabbit hole we've gone
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
    subdivide_cubic(
        x0,
        y0,
        m01_x,
        m01_y,
        m012_x,
        m012_y,
        m0123_x,
        m0123_y,
        output,
        depth + 1,
    );
    subdivide_cubic(
        m0123_x,
        m0123_y,
        m123_x,
        m123_y,
        m23_x,
        m23_y,
        x3,
        y3,
        output,
        depth + 1,
    );
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
