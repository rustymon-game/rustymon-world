//! Different output formats this tool can produce.
//!
//! - `pytest` is a simplified version of production. It is used by a python script and rendered using matplotlib to inspect geometry errors.
//! - `production` is the version rustymon's backend will store and serve to the clients.
//!
//! Each format implements the [`Constructable`] trait which allows it to be constructed using a generic interface.
use crate::geometry::{BBox, Point};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub mod memeff;
pub mod production;

/// The version rustymon's backend will store and serve to the clients.
#[allow(dead_code)]
pub type Production = production::Tile;

/// A memory efficient format with a flat point list.
#[allow(dead_code)]
pub type MemEff = memeff::Tile;

/// Abstract interface to build a tile from the geometry's "raw" results.
///
/// Highly WIP
pub trait Constructable {
    fn new(bbox: BBox) -> Self;
    fn add_area(&mut self, area: &[Point], visual_type: AreaVisualType);
    fn add_node(&mut self, node: Point, visual_type: NodeVisualType);
    fn add_way(&mut self, way: &[Point], visual_type: WayVisualType);
}

#[repr(u64)]
#[derive(Copy, Clone, Debug, Serialize_repr, Deserialize_repr)]
pub enum AreaVisualType {
    None,
    Building,
    Water,
    Forest,
    Field,
}

#[repr(u64)]
#[derive(Copy, Clone, Debug, Serialize_repr, Deserialize_repr)]
pub enum NodeVisualType {
    None,
}

#[repr(u64)]
#[derive(Copy, Clone, Debug, Serialize_repr, Deserialize_repr)]
pub enum WayVisualType {
    None,
    MotorWay,
    Trunk,
    Primary,
    Secondary,
    Tertiary,
    Residential,
    Rail,
}
