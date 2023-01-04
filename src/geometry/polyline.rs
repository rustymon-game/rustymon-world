//! Various function working with polylines

use crate::geometry::Point;

/// Create an iterator over a polygon's segments
pub fn iter_segments(polyline: &[Point]) -> impl Iterator<Item = (&Point, &Point)> {
    let len = polyline.len();
    polyline[..len - 1].iter().zip(polyline[1..].iter())
}

/// Compute a point distance to a polyline
pub fn distance_to(polyline: &[Point], point: Point) -> f64 {
    iter_segments(polyline)
        .map(|(from, to)| {
            let delta = to - from;

            // Calculate projection onto the line
            let lambda = (point - from).tr_mul(&delta).x / delta.norm();

            if 0.0 <= lambda && lambda <= 1.0 {
                (from + lambda * delta).metric_distance(&point)
            } else {
                let from_dist = point.metric_distance(from);
                let to_dist = point.metric_distance(to);

                if from_dist < to_dist {
                    from_dist
                } else {
                    to_dist
                }
            }
        })
        .min_by(|a, b| a.partial_cmp(b).expect("Distance shouldn't be NaN"))
        .expect("Polyline should contain at least 2 points to form at least one segment")
}

#[cfg(test)]
mod test {
    use crate::geometry::polyline::distance_to;
    use crate::geometry::Point;

    #[test]
    fn test_distance_to() {
        let unit_segment = &[Point::new(0.0, 0.0), Point::new(1.0, 0.0)];
        assert_eq!(
            0.0,
            distance_to(unit_segment, Point::new(0.3, 0.0)),
            "A point on the line"
        );
        assert_eq!(
            1.0,
            distance_to(unit_segment, Point::new(0.5, 1.0)),
            "A point above the line"
        );
        assert_eq!(
            3.0,
            distance_to(unit_segment, Point::new(4.0, 0.0)),
            "A point diagonal to the line"
        );
    }
}
