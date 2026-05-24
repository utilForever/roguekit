use crate::prelude::{Point, PointF};

/// Defines a two-dimensional curve by its control points.
///
/// This type stores the shared curve data used by higher-level curve
/// algorithms. It does not assume a specific interpolation or evaluation
/// strategy.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Curve {
    /// The points controlling the curve shape.
    pub control_points: Vec<Point>,
}

impl Curve {
    /// Creates a new curve from a set of control points.
    #[must_use]
    pub fn new(control_points: Vec<Point>) -> Self {
        Self { control_points }
    }

    /// Returns the control points as a slice.
    #[must_use]
    pub fn control_points(&self) -> &[Point] {
        &self.control_points
    }

    /// Returns the number of control points in the curve.
    #[must_use]
    pub fn len(&self) -> usize {
        self.control_points.len()
    }

    /// Returns true if the curve has no control points.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.control_points.is_empty()
    }

    /// Returns the first control point, if one is defined.
    #[must_use]
    pub fn first(&self) -> Option<Point> {
        self.control_points.first().copied()
    }

    /// Returns the last control point, if one is defined.
    #[must_use]
    pub fn last(&self) -> Option<Point> {
        self.control_points.last().copied()
    }

    /// Evaluates this curve as a Bezier curve at parameter `t`.
    ///
    /// `t` is typically in the range `0.0..=1.0`, where `0.0` returns the
    /// first control point and `1.0` returns the last control point. Values
    /// outside that range are evaluated without clamping.
    #[must_use]
    pub fn bezier_point(&self, t: f32) -> Option<PointF> {
        if self.control_points.is_empty() || !t.is_finite() {
            return None;
        }

        let mut points: Vec<PointF> = self
            .control_points
            .iter()
            .map(|point| point.to_vec2())
            .collect();

        while points.len() > 1 {
            points = points
                .windows(2)
                .map(|segment| segment[0] * (1.0 - t) + segment[1] * t)
                .collect();
        }

        Some(points[0])
    }

    /// Samples this curve as a Bezier curve using integer points.
    ///
    /// `steps` is the number of curve segments to sample. A non-empty curve
    /// sampled with `steps > 0` returns `steps + 1` points.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn bezier_points(&self, steps: usize) -> Vec<Point> {
        if self.control_points.is_empty() {
            return Vec::new();
        }

        if steps == 0 {
            return self
                .bezier_point(0.0)
                .map(Point::from_vec2)
                .into_iter()
                .collect();
        }

        (0..=steps)
            .filter_map(|i| {
                let t = i as f32 / steps as f32;
                self.bezier_point(t).map(Point::from_vec2)
            })
            .collect()
    }
}

impl From<Vec<Point>> for Curve {
    fn from(control_points: Vec<Point>) -> Self {
        Self::new(control_points)
    }
}

impl From<&[Point]> for Curve {
    fn from(control_points: &[Point]) -> Self {
        Self::new(control_points.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::{Curve, Point, PointF};

    fn assert_pointf_eq(actual: PointF, expected_x: f32, expected_y: f32) {
        const EPSILON: f32 = 0.001;

        assert!(
            (actual.x - expected_x).abs() < EPSILON,
            "expected x to be {expected_x}, got {}",
            actual.x
        );
        assert!(
            (actual.y - expected_y).abs() < EPSILON,
            "expected y to be {expected_y}, got {}",
            actual.y
        );
    }

    #[test]
    fn new_curve_stores_control_points() {
        let points = vec![Point::new(0, 0), Point::new(4, 8), Point::new(9, 2)];
        let curve = Curve::new(points.clone());

        assert_eq!(curve.control_points(), points.as_slice());
        assert_eq!(curve.len(), 3);
    }

    #[test]
    fn empty_curve_has_no_endpoints() {
        let curve = Curve::default();

        assert!(curve.is_empty());
        assert_eq!(curve.first(), None);
        assert_eq!(curve.last(), None);
    }

    #[test]
    fn curve_exposes_endpoints() {
        let curve = Curve::new(vec![Point::new(1, 2), Point::new(3, 4), Point::new(5, 6)]);

        assert_eq!(curve.first(), Some(Point::new(1, 2)));
        assert_eq!(curve.last(), Some(Point::new(5, 6)));
    }

    #[test]
    fn curve_can_be_created_from_points() {
        let points = [Point::new(2, 3), Point::new(4, 5)];
        let curve = Curve::from(points.as_slice());

        assert_eq!(curve.control_points(), &points);
    }

    #[test]
    fn empty_curve_has_no_bezier_points() {
        let curve = Curve::default();

        assert_eq!(curve.bezier_point(0.5), None);
        assert!(curve.bezier_points(10).is_empty());
    }

    #[test]
    fn single_control_point_bezier_returns_that_point() {
        let curve = Curve::new(vec![Point::new(3, 7)]);

        assert_pointf_eq(curve.bezier_point(0.0).unwrap(), 3.0, 7.0);
        assert_pointf_eq(curve.bezier_point(0.5).unwrap(), 3.0, 7.0);
        assert_pointf_eq(curve.bezier_point(1.0).unwrap(), 3.0, 7.0);
    }

    #[test]
    fn linear_bezier_interpolates_between_two_points() {
        let curve = Curve::new(vec![Point::new(0, 0), Point::new(10, 10)]);

        assert_pointf_eq(curve.bezier_point(0.5).unwrap(), 5.0, 5.0);
    }

    #[test]
    fn quadratic_bezier_evaluates_midpoint() {
        let curve = Curve::new(vec![
            Point::new(0, 0),
            Point::new(10, 10),
            Point::new(20, 0),
        ]);

        assert_pointf_eq(curve.bezier_point(0.5).unwrap(), 10.0, 5.0);
    }

    #[test]
    fn cubic_bezier_evaluates_midpoint() {
        let curve = Curve::new(vec![
            Point::new(0, 0),
            Point::new(0, 10),
            Point::new(10, 10),
            Point::new(10, 0),
        ]);

        assert_pointf_eq(curve.bezier_point(0.5).unwrap(), 5.0, 7.5);
    }

    #[test]
    fn bezier_endpoints_match_first_and_last_control_points() {
        let curve = Curve::new(vec![Point::new(1, 2), Point::new(5, 8), Point::new(9, 3)]);

        assert_pointf_eq(curve.bezier_point(0.0).unwrap(), 1.0, 2.0);
        assert_pointf_eq(curve.bezier_point(1.0).unwrap(), 9.0, 3.0);
    }

    #[test]
    fn non_finite_bezier_parameter_returns_none() {
        let curve = Curve::new(vec![Point::new(0, 0), Point::new(10, 10)]);

        assert_eq!(curve.bezier_point(f32::NAN), None);
        assert_eq!(curve.bezier_point(f32::INFINITY), None);
        assert_eq!(curve.bezier_point(f32::NEG_INFINITY), None);
    }

    #[test]
    fn zero_step_bezier_sampling_returns_start_point() {
        let curve = Curve::new(vec![Point::new(2, 3), Point::new(6, 9)]);

        assert_eq!(curve.bezier_points(0), vec![Point::new(2, 3)]);
    }

    #[test]
    fn bezier_sampling_returns_steps_plus_one_points() {
        let curve = Curve::new(vec![Point::new(0, 0), Point::new(10, 10)]);
        let points = curve.bezier_points(4);

        assert_eq!(points.len(), 5);
        assert_eq!(points.first(), Some(&Point::new(0, 0)));
        assert_eq!(points.last(), Some(&Point::new(10, 10)));
    }
}
