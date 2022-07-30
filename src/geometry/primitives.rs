use super::Point;

#[derive(Copy, Clone)]
pub struct Line<C: Coord>(pub C, pub f64);
impl<C: Coord> Line<C> {
    pub fn intersect(&self, from: Point, to: Point) -> Point {
        let value = self.1;

        let delta = to - from;
        let lambda = (value - C::get(from)) / C::get(delta);
        delta * lambda + from
    }
}

#[derive(Copy, Clone)]
pub struct HalfPlane<C: Coord, O: Ordering>(pub C, pub O, pub f64);
impl<C: Coord, O: Ordering> HalfPlane<C, O> {
    pub fn clip_polygon(self, input: &Vec<Point>, output: &mut Vec<Point>) {
        let value = self.2;
        for i in 0..input.len() {
            let current = input[(i + 1) % input.len()];
            let previous = input[i];

            let intersection = {
                let delta = current - previous;
                let lambda = (value - C::get(previous)) / C::get(delta);
                delta * lambda + previous
            };

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
}

pub trait Ordering {
    fn cmp(p: f64, q: f64) -> bool;
}
pub struct Gt;
impl Ordering for Gt {
    fn cmp(p: f64, q: f64) -> bool {
        p > q
    }
}
pub struct Lt;
impl Ordering for Lt {
    fn cmp(p: f64, q: f64) -> bool {
        p < q
    }
}

pub trait Coord {
    fn get(point: Point) -> f64;
}
pub struct X;
impl Coord for X {
    fn get(point: Point) -> f64 {
        point.x
    }
}
pub struct Y;
impl Coord for Y {
    fn get(point: Point) -> f64 {
        point.y
    }
}
