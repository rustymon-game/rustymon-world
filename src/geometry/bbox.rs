//! An axis aligned bounding box
//!
//! It implements simple algorithms for clipping paths and polygons.
//! But their slightly more complicated and more efficient versions from [`Grid`] are actually used.
//!
//! [`Grid`]: super::grid::Grid
use super::primitives::{Gt, HalfPlane, Lt, X, Y};
use super::Point;
use nalgebra::{Scalar, Vector2};

/// An axis aligned bounding box
#[derive(Copy, Clone, Debug)]
pub struct GenericBox<T: Scalar> {
    pub min: Vector2<T>,
    pub max: Vector2<T>,
}

/// The most commonly used bounding box
pub type BBox = GenericBox<f64>;

impl<T: PartialOrd + Scalar + Copy> GenericBox<T> {
    /// Check if a point is contained inside the bounding box
    ///
    /// If the point lies exactly on the edge it is said to be contained.
    #[inline]
    pub fn contains(&self, point: Vector2<T>) -> bool {
        self.min.x <= point.x
            && self.min.y <= point.y
            && point.x <= self.max.x
            && point.y <= self.max.y
    }

    /// Adjust the bounding box's size to fit a given point
    #[inline]
    pub fn fit(&mut self, point: Vector2<T>) {
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
    pub fn intersects_box(&self, other: GenericBox<T>) -> bool {
        self.contains(other.min)
            || self.contains(other.max)
            || self.contains(Vector2::new(other.min.x, other.max.y))
            || self.contains(Vector2::new(other.max.x, other.min.y))
    }
}

impl<T: PartialOrd + Scalar + Copy> FromIterator<Vector2<T>> for GenericBox<T>
where
    GenericBox<T>: Default,
{
    fn from_iter<I: IntoIterator<Item = Vector2<T>>>(iter: I) -> Self {
        let mut bbox: GenericBox<T> = Default::default();
        for v in iter {
            bbox.fit(v);
        }
        bbox
    }
}

impl Default for GenericBox<i32> {
    fn default() -> Self {
        GenericBox {
            min: Vector2::new(i32::MAX, i32::MAX),
            max: Vector2::new(i32::MIN, i32::MIN),
        }
    }
}

impl Default for GenericBox<isize> {
    fn default() -> Self {
        GenericBox {
            min: Vector2::new(isize::MAX, isize::MAX),
            max: Vector2::new(isize::MIN, isize::MIN),
        }
    }
}

impl Default for BBox {
    fn default() -> Self {
        BBox::new()
    }
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
            min: Point::new(f64::INFINITY, f64::INFINITY),
            max: Point::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }

    /// Finds the intersection with a line segment
    #[allow(dead_code)]
    pub fn intersect_line(&self, from: Point, to: Point) -> Option<(Point, Option<Point>)> {
        let mut result: Option<(Point, Option<Point>)> = None;
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
    #[deprecated = "Use Grid::clip_path instead because it scales way better."]
    pub fn clip_path<T: IntoIterator<Item = Point>>(&self, path: T) -> Vec<Vec<Point>> {
        let mut paths: Vec<Vec<Point>> = Vec::new();

        let mut was_in_box = false; // Whether or not the last point was inside the box
        let mut first_iter = true; // If the first point starts in box no intersection is required
        let mut last_point = Point::new(0.0, 0.0);

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
    #[allow(dead_code)]
    #[deprecated = "Use Grid::clip_polygon instead because it scales way better."]
    pub fn clip_polygon<T: IntoIterator<Item = Point>>(&self, subject: T) -> Vec<Point> {
        let mut a = subject.into_iter().collect();
        let mut b = Vec::new();

        HalfPlane(Y, Gt, self.min.y).clip(&a, &mut b);
        a.clear();

        HalfPlane(X, Lt, self.max.x).clip(&b, &mut a);
        b.clear();

        HalfPlane(Y, Lt, self.min.y).clip(&a, &mut b);
        a.clear();

        HalfPlane(X, Gt, self.max.x).clip(&b, &mut a);

        a.shrink_to_fit();
        a
    }
}

#[cfg(test)]
mod test {
    use crate::geometry::{BBox, Point};

    static ORIGIN: Point = Point::new(0.0, 0.0);

    /// Set of points "randomly" created by a human
    static POINTS: [Point; 5] = [
        Point::new(0.0, 0.0),
        Point::new(12.3, 4.56),
        Point::new(7.0, 8.0),
        Point::new(-1.3, -3.7),
        Point::new(-3.0, -5.0),
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
            min: Point::new(-1.0, -1.0),
            max: Point::new(1.0, 1.0),
        };

        // Check lines starting on the origin along an axis
        {
            assert_eq!(
                b.intersect_line(ORIGIN, Point::new(2.0, 0.0)),
                Some((Point::new(1.0, 0.0), None))
            );
            assert_eq!(
                b.intersect_line(ORIGIN, Point::new(-2.0, 0.0)),
                Some((Point::new(-1.0, 0.0), None))
            );
            assert_eq!(
                b.intersect_line(ORIGIN, Point::new(0.0, 2.0)),
                Some((Point::new(0.0, 1.0), None))
            );
            assert_eq!(
                b.intersect_line(ORIGIN, Point::new(0.0, -2.0)),
                Some((Point::new(0.0, -1.0), None))
            );
        }

        // Check "whole" axis'
        {
            assert_eq!(
                b.intersect_line(Point::new(2.0, 0.0), Point::new(-2.0, 0.0)),
                Some((Point::new(-1.0, 0.0), Some(Point::new(1.0, 0.0))))
            );
            assert_eq!(
                b.intersect_line(Point::new(0.0, 2.0), Point::new(0.0, -2.0)),
                Some((Point::new(0.0, -1.0), Some(Point::new(0.0, 1.0))))
            );
        }

        // Check error cases found in debugging
        {
            let b = BBox {
                min: Point::new(9.55263, 47.11752),
                max: Point::new(9.55637, 47.12132),
            };
            assert_eq!(
                b.intersect_line(
                    Point::new(9.5560283, 47.121235),
                    Point::new(9.556378, 47.1214064), // x slightly larger
                ),
                Some((Point::new(9.556201721820301, 47.12132), None))
            );
        }
        // TODO more "complex" lines
    }

    #[test]
    fn bbox_clip_path() {
        // TODO
    }
}
