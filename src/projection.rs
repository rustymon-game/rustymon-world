use std::f64::consts::PI;

use libosmium::{Location, Node, NodeRef};
use nalgebra::Vector2;

pub trait GetLocation {
    fn get_location(&self) -> Option<Location>;
}
impl GetLocation for Node {
    #[inline]
    fn get_location(&self) -> Option<Location> {
        Some(self.location())
    }
}
impl GetLocation for NodeRef {
    #[inline]
    fn get_location(&self) -> Option<Location> {
        NodeRef::get_location(self)
    }
}

pub trait Projection: Copy + 'static {
    fn project(&self, point: &impl GetLocation) -> Option<Vector2<f64>> {
        point.get_location().map(|location| {
            let lambda = location.lon().to_radians();
            let phi = location.lat().to_radians();
            let (x, y) = self._project(lambda, phi);
            Vector2::new(x, y)
        })
    }

    fn project_nalgebra(&self, point: Vector2<f64>) -> Vector2<f64> {
        let [[lambda, phi]] = point.data.0;
        let (x, y) = self._project(lambda.to_radians(), phi.to_radians());
        Vector2::new(x, y)
    }

    fn _project(&self, lambda: f64, phi: f64) -> (f64, f64);
}

#[derive(Copy, Clone)]
pub struct Simple;
impl Projection for Simple {
    #[inline]
    fn _project(&self, lambda: f64, phi: f64) -> (f64, f64) {
        (lambda, phi)
    }
}

#[derive(Copy, Clone)]
pub struct WebMercator;
impl Projection for WebMercator {
    #[inline]
    fn _project(&self, lambda: f64, phi: f64) -> (f64, f64) {
        let x = (lambda + PI) / (2.0 * PI);
        let y = PI - (PI / 4.0 + phi / 2.0).tan().ln().clamp(0.0, 1.0);
        (x, y)
    }
}
