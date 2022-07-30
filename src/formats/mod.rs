//! Different output formats this tool can produce.
//!
//! - `pytest` is a simplified version of production. It is used by a python script and rendered using matplotlib to inspect geometry errors.
//! - `production` is the version rustymon's backend will store and serve to the clients.
//!
//! Each format implements the [`Constructable`] trait which allows it to be constructed using a generic interface.
use crate::geometry::{BBox, Point};
use nalgebra::Vector2;

mod production;
mod pytest;

/// A simplified version of production. It is used by a python script and rendered using matplotlib to inspect geometry errors.
#[allow(dead_code)]
pub type Production = production::Tile;

/// The version rustymon's backend will store and serve to the clients.
#[allow(dead_code)]
pub type Pytest = pytest::Tile;

/// Abstract interface to build a tile from the geometry's "raw" results.
///
/// Highly WIP
pub trait Constructable {
    fn new(bbox: BBox) -> Self;
    fn add_area(&mut self, area: Vec<Point>);
    fn add_node(&mut self, node: Point);
    fn add_way(&mut self, way: Vec<Point>) {
        self.extend_ways([way]);
    }
    fn extend_ways(&mut self, ways: impl IntoIterator<Item = Vec<Point>>);
}
