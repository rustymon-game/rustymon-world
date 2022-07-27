use nalgebra::Vector2;

pub mod pytest;

pub trait Constructable {
    fn new() -> Self;
    fn add_area(&mut self, area: Vec<Vector2<f64>>);
    fn add_node(&mut self, node: Vector2<f64>);
    fn extend_ways(&mut self, ways: impl IntoIterator<Item=Vec<Vector2<f64>>>);
}