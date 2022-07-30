pub mod bbox;
pub mod grid;
pub mod iter;
pub mod primitives;

pub use bbox::BBox;

pub type Point = nalgebra::Vector2<f64>;
