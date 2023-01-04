//! Intersecting and clipping on axis-aligned lines and half-planes.
use super::{polygon, Point};

/// A horizontal or vertical line
///
/// It is defined by a [coordinate selector] and a value which is constant along the line.
///
/// This line can be intersected with another line defined by two points (see [`intersect`]).
///
/// For example:
/// - `Line(X, 4.0)` defines the vertical line at `x = 4`
/// - `Line(Y, 0.0).intersect(p, q)` computes the intersection of the line through `p` and `q` with the x-axis.
///
/// [coordinate selector]: Coord
/// [`intersect`]: Line::intersect
#[derive(Copy, Clone)]
pub struct Line<C: Coord>(pub C, pub f64);
impl<C: Coord> Line<C> {
    /// Calculate the intersection with another line defined by two points
    ///
    /// ```
    /// # use nalgebra::Vector2;
    /// # use rustymon_world::geometry::primitives::{Line, X};
    /// let p = Vector2::new(1.0, 2.0);
    /// let q = Vector2::new(-1.0, -2.0);
    /// assert_eq!(
    ///     Line(X, 0.5).intersect(p, q),
    ///     Vector2::new(0.5, 1.0)
    /// );
    pub fn intersect(&self, from: Point, to: Point) -> Point {
        let value = self.1;

        let delta = to - from;
        let lambda = (value - C::get(from)) / C::get(delta);

        delta * lambda + from
    }

    /// Calculate the intersection of this (infinite) line with a (finite) line segment defined by its endpoints
    pub fn intersect_segment(&self, from: Point, to: Point) -> Option<Point> {
        let value = self.1;

        let delta = to - from;
        let lambda = (value - C::get(from)) / C::get(delta);

        if (0.0..=1.0).contains(&lambda) {
            Some(delta * lambda + from)
        } else {
            None
        }
    }
}

/// A half of the plane seperated by a [`Line`]
///
/// Like a [`Line`], a half-plane is defined by a [coordinate selector](Coord) and a value.
/// But additionally it takes a [comparison operator](Ordering) to select a half of the plane.
///
/// The plane can be used to clip a polygon dumping everything on the other half of the plane (see [`clip`]).
///
/// For example:
/// - `HalfPlane(X, Gt, 0.0)` defines the half plane of all points with positive x coordinates.
///
/// [`clip`]: HalfPlane::clip
#[derive(Copy, Clone)]
pub struct HalfPlane<C: Coord, O: Ordering>(pub C, pub O, pub f64);
impl<C: Coord, O: Ordering> HalfPlane<C, O> {
    /// Clip a polygon using this half-plane.
    ///
    /// This dumps all vertices ling in the half-plane and adds new vertices at the intersection.
    pub fn clip(self, input: &[Point], output: &mut Vec<Point>) {
        let value = self.2;
        for (&previous, &current) in polygon::iter_edges(input) {
            let intersection = self.intersect(current, previous);

            if O::cmp(C::get(current), value) {
                if !O::cmp(C::get(previous), value) {
                    output.push(intersection);
                }
                output.push(current);
            } else if O::cmp(C::get(previous), value) {
                output.push(intersection);
            }
        }
    }

    /// Check whether a point is contained in this half plane
    pub fn contains(self, point: Point) -> bool {
        O::cmp(C::get(point), self.2)
    }

    /// Calculate the half plane's boundary's intersection with another line defined by two points
    pub fn intersect(self, from: Point, to: Point) -> Point {
        Line(self.0, self.2).intersect(from, to)
    }
}

/// Selector for a comparison operator
///
/// Use [`Gt`] and [`Lt`] to choose a comparison operator statically
pub trait Ordering: Copy {
    fn cmp(p: f64, q: f64) -> bool;
}

/// Select points whose coordinates are greater than a certain value
///
/// ```
/// # use rustymon_world::geometry::primitives::{Gt, Ordering};
/// # let (p, q) = (0.0, 0.0);
/// assert_eq!(
///     Gt::cmp(p, q),
///     p > q
/// );
/// ```
#[derive(Copy, Clone)]
pub struct Gt;
impl Ordering for Gt {
    fn cmp(p: f64, q: f64) -> bool {
        p > q
    }
}

/// Select points whose coordinates are less than a certain value
///
/// ```
/// # use rustymon_world::geometry::primitives::{Lt, Ordering};
/// # let (p, q) = (0.0, 0.0);
/// assert_eq!(
///     Lt::cmp(p, q),
///     p < q
/// );
/// ```
#[derive(Copy, Clone)]
pub struct Lt;
impl Ordering for Lt {
    fn cmp(p: f64, q: f64) -> bool {
        p < q
    }
}

/// Selector for a point's coordinate
///
/// Use [`X`] and [`Y`] to choose a coordinates statically
pub trait Coord: Copy {
    fn get(point: Point) -> f64;
}

/// Select a point's x coordinate:
///
/// ```
/// # use rustymon_world::geometry::primitives::{X, Coord};
/// # let point = nalgebra::Vector2::zeros();
/// assert_eq!(
///     X::get(point),
///     point.x
/// );
/// ```
#[derive(Copy, Clone)]
pub struct X;
impl Coord for X {
    fn get(point: Point) -> f64 {
        point.x
    }
}

/// Select a point's y coordinate:
///
/// ```
/// # use rustymon_world::geometry::primitives::{Y, Coord};
/// # let point = nalgebra::Vector2::zeros();
/// assert_eq!(
///     Y::get(point),
///     point.y
/// );
/// ```
#[derive(Copy, Clone)]
pub struct Y;
impl Coord for Y {
    fn get(point: Point) -> f64 {
        point.y
    }
}
