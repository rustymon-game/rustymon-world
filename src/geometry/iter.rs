use super::bbox::BBox;
use nalgebra::Vector2;

/// High-level syntax for the `intersect` argument in [`BBox::clip_polygon_on_line`]
macro_rules! intersect_with {
    ($x:ident = $value:expr) => {
        move |from: Vector2<f64>, to: Vector2<f64>| -> Vector2<f64> {
            let delta = from - to;
            let lambda = ($value - to.$x) / delta.$x;
            delta * lambda + to
        }
    };
}
/// High-level syntax for the `is_inside` argument to [`BBox::clip_polygon_on_line`]
macro_rules! keep {
    ($x:ident < $value:expr) => {
        move |point: Vector2<f64>| -> bool { point.$x < $value }
    };
    ($x:ident > $value:expr) => {
        move |point: Vector2<f64>| -> bool { point.$x > $value }
    };
}

pub fn clip_polygon(
    polygon: impl Iterator<Item = Vector2<f64>>,
    bbox: BBox,
) -> impl Iterator<Item = Vector2<f64>> {
    let polygon = clip_polygon_on_line(
        polygon,
        keep!(y > bbox.min.y),
        intersect_with!(y = bbox.min.y),
    );
    let polygon = clip_polygon_on_line(
        polygon,
        keep!(x < bbox.max.x),
        intersect_with!(x = bbox.max.x),
    );
    let polygon = clip_polygon_on_line(
        polygon,
        keep!(y < bbox.max.y),
        intersect_with!(y = bbox.max.y),
    );
    let polygon = clip_polygon_on_line(
        polygon,
        keep!(x > bbox.min.x),
        intersect_with!(x = bbox.min.x),
    );
    polygon
}

pub fn clip_polygon_on_line(
    polygon: impl Iterator<Item = Vector2<f64>>,
    keep: impl Fn(Vector2<f64>) -> bool,
    intersect: impl Fn(Vector2<f64>, Vector2<f64>) -> Vector2<f64>,
) -> impl Iterator<Item = Vector2<f64>> {
    ClipPolygonOnLineIter::new(polygon, keep, intersect)
}

struct ClipPolygonOnLineIter<P, K, I> {
    finished: bool,
    left_yield: Option<Vector2<f64>>,
    polygon: P,
    previous: Vector2<f64>,
    last: Vector2<f64>,
    keep: K,
    intersect: I,
}
impl<P, K, I> ClipPolygonOnLineIter<P, K, I>
where
    P: Iterator<Item = Vector2<f64>>,
    K: Fn(Vector2<f64>) -> bool,
    I: Fn(Vector2<f64>, Vector2<f64>) -> Vector2<f64>,
{
    pub fn new(mut polygon: P, keep: K, intersect: I) -> ClipPolygonOnLineIter<P, K, I> {
        if let Some(previous) = polygon.next() {
            ClipPolygonOnLineIter {
                finished: false,
                left_yield: None,
                polygon,
                previous,
                last: previous,
                keep,
                intersect,
            }
        } else {
            ClipPolygonOnLineIter {
                finished: true,
                left_yield: None,
                polygon,
                previous: Vector2::new(f64::NAN, f64::NAN),
                last: Vector2::new(f64::NAN, f64::NAN),
                keep,
                intersect,
            }
        }
    }
}
impl<P, K, I> Iterator for ClipPolygonOnLineIter<P, K, I>
where
    P: Iterator<Item = Vector2<f64>>,
    K: Fn(Vector2<f64>) -> bool,
    I: Fn(Vector2<f64>, Vector2<f64>) -> Vector2<f64>,
{
    type Item = Vector2<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        let intersect = &self.intersect;
        let keep = &self.keep;

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

            let intersection = intersect(self.previous, current);
            let keep_previous = keep(self.previous);
            self.previous = current;

            if keep(current) {
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
