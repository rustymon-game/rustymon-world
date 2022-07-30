pub mod bbox;
pub mod grid;
pub mod iter;

pub use bbox::BBox;
use nalgebra::Vector2;

#[derive(Copy, Clone, PartialEq, Debug)]
enum Line {
    Horizontal {
        y: f64,
    },
    Vertical {
        x: f64,
    },
    #[allow(dead_code)]
    Arbitrary {
        delta: Vector2<f64>,
        point: Vector2<f64>,
    },
}

impl Line {
    pub fn intersect(&self, from: Point, to: Point) -> Point {
        let delta = to - from;
        let lambda = match self {
            Line::Horizontal { y } => (y - from.y) / delta.y,
            Line::Vertical { x } => (x - from.x) / delta.x,
            Line::Arbitrary { .. } => todo!(),
        };
        delta * lambda + from
    }
}

pub type Point = Vector2<f64>;
