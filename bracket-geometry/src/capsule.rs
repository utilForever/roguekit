use crate::prelude::{Point, Rect};
use std::collections::HashSet;

/// Defines a two-dimensional capsule.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Capsule {
    /// The center point of the first rounded end.
    pub start: Point,
    /// The center point of the second rounded end.
    pub end: Point,
    /// The radius of the rounded ends and half-width of the capsule.
    pub radius: i32,
}

#[cfg(feature = "specs")]
impl specs::prelude::Component for Capsule {
    type Storage = specs::prelude::VecStorage<Self>;
}

impl Capsule {
    /// Create a new capsule from a center line and radius.
    #[must_use]
    pub fn new(start: Point, end: Point, radius: i32) -> Self {
        Self { start, end, radius }
    }

    /// Returns true if a point is inside the capsule.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn point_in_capsule(&self, point: Point) -> bool {
        let radius = self.radius.abs();
        let radius_squared = (radius * radius) as f32;

        let start_x = self.start.x as f32;
        let start_y = self.start.y as f32;
        let end_x = self.end.x as f32;
        let end_y = self.end.y as f32;
        let point_x = point.x as f32;
        let point_y = point.y as f32;

        let segment_x = end_x - start_x;
        let segment_y = end_y - start_y;
        let point_offset_x = point_x - start_x;
        let point_offset_y = point_y - start_y;
        let segment_length_squared = (segment_x * segment_x) + (segment_y * segment_y);

        if segment_length_squared <= f32::EPSILON {
            let distance_x = point_x - start_x;
            let distance_y = point_y - start_y;
            return (distance_x * distance_x) + (distance_y * distance_y) <= radius_squared;
        }

        let projection =
            ((point_offset_x * segment_x) + (point_offset_y * segment_y)) / segment_length_squared;
        let projection = projection.clamp(0.0, 1.0);
        let closest_x = start_x + (segment_x * projection);
        let closest_y = start_y + (segment_y * projection);
        let distance_x = point_x - closest_x;
        let distance_y = point_y - closest_y;

        (distance_x * distance_x) + (distance_y * distance_y) <= radius_squared
    }

    /// Returns the smallest axis-aligned rectangle that contains the capsule.
    #[must_use]
    pub fn bounding_rect(&self) -> Rect {
        let radius = self.radius.abs();
        Rect::with_exact(
            i32::min(self.start.x, self.end.x) - radius,
            i32::min(self.start.y, self.end.y) - radius,
            i32::max(self.start.x, self.end.x) + radius + 1,
            i32::max(self.start.y, self.end.y) + radius + 1,
        )
    }

    /// Calls a function for each point in the capsule.
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(Point),
    {
        self.bounding_rect().for_each(|point| {
            if self.point_in_capsule(point) {
                f(point);
            }
        });
    }

    /// Gets a set of all tiles in the capsule.
    #[must_use]
    pub fn point_set(&self) -> HashSet<Point> {
        let mut result = HashSet::new();
        self.for_each(|point| {
            result.insert(point);
        });
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::{Capsule, Point, Rect};
    use std::collections::HashSet;

    #[test]
    fn test_new() {
        let capsule = Capsule::new(Point::new(0, 0), Point::new(10, 0), 2);
        assert_eq!(capsule.start, Point::new(0, 0));
        assert_eq!(capsule.end, Point::new(10, 0));
        assert_eq!(capsule.radius, 2);
    }

    #[test]
    fn test_point_on_center_line_is_inside() {
        let capsule = Capsule::new(Point::new(0, 0), Point::new(10, 0), 2);
        assert!(capsule.point_in_capsule(Point::new(5, 0)));
    }

    #[test]
    fn test_point_within_radius_is_inside() {
        let capsule = Capsule::new(Point::new(0, 0), Point::new(10, 0), 2);
        assert!(capsule.point_in_capsule(Point::new(5, 2)));
    }

    #[test]
    fn test_point_outside_radius_is_outside() {
        let capsule = Capsule::new(Point::new(0, 0), Point::new(10, 0), 2);
        assert!(!capsule.point_in_capsule(Point::new(5, 3)));
    }

    #[test]
    fn test_point_near_rounded_end_is_inside() {
        let capsule = Capsule::new(Point::new(0, 0), Point::new(10, 0), 2);
        assert!(capsule.point_in_capsule(Point::new(-1, 1)));
    }

    #[test]
    fn test_point_past_rounded_end_is_outside() {
        let capsule = Capsule::new(Point::new(0, 0), Point::new(10, 0), 2);
        assert!(!capsule.point_in_capsule(Point::new(-3, 0)));
    }

    #[test]
    fn test_zero_length_capsule_behaves_like_circle() {
        let capsule = Capsule::new(Point::new(0, 0), Point::new(0, 0), 2);
        assert!(capsule.point_in_capsule(Point::new(1, 1)));
        assert!(!capsule.point_in_capsule(Point::new(3, 0)));
    }

    #[test]
    fn test_diagonal_capsule() {
        let capsule = Capsule::new(Point::new(0, 0), Point::new(10, 10), 2);
        assert!(capsule.point_in_capsule(Point::new(5, 5)));
        assert!(capsule.point_in_capsule(Point::new(5, 6)));
        assert!(!capsule.point_in_capsule(Point::new(5, 9)));
    }

    #[test]
    fn test_bounding_rect() {
        let capsule = Capsule::new(Point::new(0, 0), Point::new(10, 0), 2);
        assert_eq!(capsule.bounding_rect(), Rect::with_exact(-2, -2, 13, 3));
    }

    #[test]
    fn test_capsule_set() {
        let capsule = Capsule::new(Point::new(0, 0), Point::new(1, 0), 1);
        let points = capsule.point_set();
        assert!(points.contains(&Point::new(0, 0)));
        assert!(points.contains(&Point::new(1, 0)));
        assert!(points.contains(&Point::new(0, 1)));
        assert!(points.contains(&Point::new(1, -1)));
        assert!(!points.contains(&Point::new(0, 2)));
    }

    #[test]
    fn test_capsule_callback() {
        let capsule = Capsule::new(Point::new(0, 0), Point::new(1, 0), 1);
        let mut points: HashSet<Point> = HashSet::new();
        capsule.for_each(|point| {
            points.insert(point);
        });
        assert!(points.contains(&Point::new(0, 0)));
        assert!(points.contains(&Point::new(1, 0)));
        assert!(!points.contains(&Point::new(0, 2)));
    }
}
