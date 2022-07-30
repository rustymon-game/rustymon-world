#![allow(dead_code)]

use super::{bbox::BBox, Point};
use crate::geometry::primitives::{Coord, Gt, HalfPlane, Lt, Ordering, X, Y};

pub fn clip_polygon(
    polygon: impl Iterator<Item = Point>,
    bbox: BBox,
) -> impl Iterator<Item = Point> {
    let polygon = HalfPlane(Y, Gt, bbox.min.y).iter_clip_polygon(polygon);
    let polygon = HalfPlane(X, Lt, bbox.max.x).iter_clip_polygon(polygon);
    let polygon = HalfPlane(Y, Lt, bbox.max.y).iter_clip_polygon(polygon);
    let polygon = HalfPlane(X, Gt, bbox.min.x).iter_clip_polygon(polygon);
    polygon
}

struct ClipPolygonOnLineIter<P, C: Coord, O: Ordering> {
    finished: bool,
    left_yield: Option<Point>,
    polygon: P,
    previous: Point,
    last: Point,
    halfplane: HalfPlane<C, O>,
}
impl<C: Coord, O: Ordering> HalfPlane<C, O> {
    pub fn iter_clip_polygon(
        self,
        mut polygon: impl Iterator<Item = Point>,
    ) -> impl Iterator<Item = Point> {
        if let Some(previous) = polygon.next() {
            ClipPolygonOnLineIter {
                finished: false,
                left_yield: None,
                polygon,
                previous,
                last: previous,
                halfplane: self,
            }
        } else {
            ClipPolygonOnLineIter {
                finished: true,
                left_yield: None,
                polygon,
                previous: Point::new(f64::NAN, f64::NAN),
                last: Point::new(f64::NAN, f64::NAN),
                halfplane: self,
            }
        }
    }
}
impl<P, C, O> Iterator for ClipPolygonOnLineIter<P, C, O>
where
    P: Iterator<Item = Point>,
    C: Coord,
    O: Ordering,
{
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(left_yield) = self.left_yield.take() {
            Some(left_yield)
        } else if self.finished {
            None
        } else {
            let current = if let Some(current) = self.polygon.next() {
                self.finished = false;
                current
            } else {
                self.finished = true;
                self.last
            };

            let intersection = self.halfplane.intersect(self.previous, current);
            let keep_previous = self.halfplane.contains(self.previous);
            let keep_current = self.halfplane.contains(current);
            self.previous = current;

            if keep_current {
                if !keep_previous {
                    self.left_yield = Some(current);
                    Some(intersection)
                } else {
                    Some(current)
                }
            } else if keep_previous {
                Some(intersection)
            } else {
                self.next()
            }
        }
    }
}
