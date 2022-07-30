use super::primitives::{Gt, HalfPlane, Lt, X, Y};
use nalgebra::Vector2;

/// An axis aligned bounding box
#[derive(Copy, Clone, Debug)]
pub struct BBox {
    pub min: Vector2<f64>,
    pub max: Vector2<f64>,
}

impl BBox {
    /// Create an "empty" bounding box which contains no point
    ///
    /// After creating use [`fit`] at least twice to get an actual bounding box.
    ///
    /// [`fit`]: BBox::fit
    #[inline]
    pub fn new() -> BBox {
        BBox {
            min: Vector2::new(f64::INFINITY, f64::INFINITY),
            max: Vector2::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }

    /// Check if a point is contained inside the bounding box
    ///
    /// If the point lies exactly on the edge it is said to be contained.
    #[inline]
    pub fn contains(&self, point: Vector2<f64>) -> bool {
        self.min.x <= point.x
            && self.min.y <= point.y
            && point.x <= self.max.x
            && point.y <= self.max.y
    }

    /// Adjust the bounding box's size to fit a given point
    #[inline]
    pub fn fit(&mut self, point: Vector2<f64>) {
        use std::cmp::Ordering::{Greater, Less};
        if matches!(self.min.x.partial_cmp(&point.x), Some(Greater)) {
            self.min.x = point.x;
        }
        if matches!(self.min.y.partial_cmp(&point.y), Some(Greater)) {
            self.min.y = point.y;
        }
        if matches!(self.max.x.partial_cmp(&point.x), Some(Less)) {
            self.max.x = point.x;
        }
        if matches!(self.max.y.partial_cmp(&point.y), Some(Less)) {
            self.max.y = point.y;
        }
    }

    /// Check if two bounding boxes intersect
    #[allow(dead_code)]
    pub fn intersects_box(&self, other: BBox) -> bool {
        self.contains(other.min)
            || self.contains(other.max)
            || self.contains(Vector2::new(other.min.x, other.max.y))
            || self.contains(Vector2::new(other.max.x, other.min.y))
    }

    /// Finds the intersection with a line segment
    #[allow(dead_code)]
    pub fn intersect_line(
        &self,
        from: Vector2<f64>,
        to: Vector2<f64>,
    ) -> Option<(Vector2<f64>, Option<Vector2<f64>>)> {
        let mut result: Option<(Vector2<f64>, Option<Vector2<f64>>)> = None;
        let delta = from - to;

        // Intersections of [from, to] with the lines x=.. and y=..
        let lambdas = [
            (self.min.x - to.x) / delta.x,
            (self.min.y - to.y) / delta.y,
            (self.max.x - to.x) / delta.x,
            (self.max.y - to.y) / delta.y,
        ];

        for lambda in lambdas {
            // Intersection actually lies in [from, to]
            if 0.0 <= lambda && lambda <= 1.0 {
                let point = delta * lambda + to;
                // Intersection doesn't lie outside of the box
                if self.contains(point) {
                    if let Some((_, second)) = result.as_mut() {
                        if second.is_none() {
                            *second = Some(point);
                        } else {
                            println!(
                                "BBox: {:?}\nLine: {:?} {:?}\nlambdas: {:?}",
                                self, from, to, lambdas
                            );
                            unreachable!("A line can't intersect a box more than twice");
                        }
                    } else {
                        result = Some((point, None));
                    }
                }
            }
        }
        result
    }

    /// Clip a path to the bounding box
    ///
    /// A path is a list of points which are thought of as joint by line segments.
    /// These segments can intersect the bounding box
    /// while the path's points might lie inside or outside of the box.
    ///
    /// This function takes such a path and cuts it into pieces
    /// where each piece lies entirely inside the box while dumping everything outside.
    #[allow(dead_code)]
    pub fn clip_path<T: IntoIterator<Item = Vector2<f64>>>(
        &self,
        path: T,
    ) -> Vec<Vec<Vector2<f64>>> {
        let mut paths: Vec<Vec<Vector2<f64>>> = Vec::new();

        let mut was_in_box = false; // Whether or not the last point was inside the box
        let mut first_iter = true; // If the first point starts in box no intersection is required
        let mut last_point = Vector2::new(0.0, 0.0);

        for point in path {
            if self.contains(point) {
                if was_in_box {
                    // A simple step inside the box
                    paths.last_mut()
                        .expect("If the last point was in the box it must have been already added to some path")
                        .push(point);
                } else if first_iter {
                    // First point of path started in the box
                    paths.push(vec![point]);
                } else {
                    // Path reentered the box
                    let entry_point = match self.intersect_line(last_point, point) {
                        Some((intersection, None)) => intersection,
                        _ => unreachable!("there should be exactly one intersection, because `last_point` lies outside and `point` inside of the box"),
                    };
                    paths.push(vec![entry_point, point]);
                }
                was_in_box = true;
            } else {
                if was_in_box {
                    // Path is leaving the box
                    let exit_point = match self.intersect_line(last_point, point) {
                        Some((intersection, None)) => intersection,
                        _ => unreachable!("there should be exactly one intersection, because `point` lies outside and `last_point` inside of the box"),
                    };
                    paths.last_mut()
                        .expect("If the last point was in the box it must have been already added to some path")
                        .push(exit_point);
                } else if !first_iter {
                    if let Some((first, Some(second))) = self.intersect_line(last_point, point) {
                        // Path enters and leaves again in a single step
                        paths.push(vec![first, second]);
                    }
                }
                was_in_box = false;
            }
            first_iter = false;
            last_point = point;
        }
        paths
    }

    /// Clip a polygon to the bounding box
    #[inline]
    pub fn clip_polygon<T: IntoIterator<Item = Vector2<f64>>>(
        &self,
        subject: T,
    ) -> Vec<Vector2<f64>> {
        let mut a = subject.into_iter().collect();
        let mut b = Vec::new();

        HalfPlane(Y, Gt, self.min.y).clip_polygon(&a, &mut b);
        a.clear();

        HalfPlane(X, Lt, self.max.x).clip_polygon(&b, &mut a);
        b.clear();

        HalfPlane(Y, Lt, self.min.y).clip_polygon(&a, &mut b);
        a.clear();

        HalfPlane(X, Gt, self.max.x).clip_polygon(&b, &mut a);

        a.shrink_to_fit();
        a
    }
}

impl FromIterator<Vector2<f64>> for BBox {
    fn from_iter<T: IntoIterator<Item = Vector2<f64>>>(iter: T) -> Self {
        let mut bbox = BBox::new();
        for v in iter {
            bbox.fit(v);
        }
        bbox
    }
}

#[cfg(test)]
mod test {
    use crate::geometry::BBox;
    use nalgebra::Vector2;

    static ORIGIN: Vector2<f64> = Vector2::new(0.0, 0.0);

    /// Set of points "randomly" created by a human
    static POINTS: [Vector2<f64>; 5] = [
        Vector2::new(0.0, 0.0),
        Vector2::new(12.3, 4.56),
        Vector2::new(7.0, 8.0),
        Vector2::new(-1.3, -3.7),
        Vector2::new(-3.0, -5.0),
    ];

    #[test]
    fn bbox_fit_contains() {
        let mut b = BBox::new();
        for p in POINTS {
            b.fit(p);
        }
        for p in POINTS {
            assert!(b.contains(p));
        }
    }

    #[test]
    fn bbox_intersect_line() {
        let b = BBox {
            min: Vector2::new(-1.0, -1.0),
            max: Vector2::new(1.0, 1.0),
        };

        // Check lines starting on the origin along an axis
        {
            assert_eq!(
                b.intersect_line(ORIGIN, Vector2::new(2.0, 0.0)),
                Some((Vector2::new(1.0, 0.0), None))
            );
            assert_eq!(
                b.intersect_line(ORIGIN, Vector2::new(-2.0, 0.0)),
                Some((Vector2::new(-1.0, 0.0), None))
            );
            assert_eq!(
                b.intersect_line(ORIGIN, Vector2::new(0.0, 2.0)),
                Some((Vector2::new(0.0, 1.0), None))
            );
            assert_eq!(
                b.intersect_line(ORIGIN, Vector2::new(0.0, -2.0)),
                Some((Vector2::new(0.0, -1.0), None))
            );
        }

        // Check "whole" axis'
        {
            assert_eq!(
                b.intersect_line(Vector2::new(2.0, 0.0), Vector2::new(-2.0, 0.0)),
                Some((Vector2::new(-1.0, 0.0), Some(Vector2::new(1.0, 0.0))))
            );
            assert_eq!(
                b.intersect_line(Vector2::new(0.0, 2.0), Vector2::new(0.0, -2.0)),
                Some((Vector2::new(0.0, -1.0), Some(Vector2::new(0.0, 1.0))))
            );
        }

        // Check error cases found in debugging
        {
            let b = BBox {
                min: Vector2::new(9.55263, 47.11752),
                max: Vector2::new(9.55637, 47.12132),
            };
            assert_eq!(
                b.intersect_line(
                    Vector2::new(9.5560283, 47.121235),
                    Vector2::new(9.556378, 47.1214064), // x slightly larger
                ),
                Some((Vector2::new(9.556201721820301, 47.12132), None))
            );
        }
        // TODO more "complex" lines
    }

    #[test]
    fn bbox_clip_path() {
        // TODO
    }
}
