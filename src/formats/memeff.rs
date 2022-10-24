use crate::formats::{AreaVisualType, Constructable, NodeVisualType, WayVisualType};
use crate::geometry::{BBox, Point};
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Tile {
    pub min: Vector2<f64>,
    pub max: Vector2<f64>,

    pub areas: Vec<Item<(usize, usize)>>,
    pub nodes: Vec<Item<usize>>,
    pub ways: Vec<Item<(usize, usize)>>,

    /// Common pool of points used by all areas, nodes and ways
    pub points: Vec<Vector2<f64>>,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Item<Index> {
    pub visual: usize,
    pub oid: usize,

    /// Ether `usize` for nodes or `(usize, usize)` defining a range for areas and ways.
    pub points: Index,
}

/// Implements iterators hiding the flattened points
impl Tile {
    pub fn iter_areas(&self) -> impl Iterator<Item = Item<&[Point]>> {
        self.areas.iter().cloned().map(
            |Item {
                 visual,
                 oid,
                 points: (start, end),
             }| Item {
                visual,
                oid,
                points: &self.points[start..end],
            },
        )
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = Item<&Point>> {
        self.nodes.iter().cloned().map(
            |Item {
                 visual,
                 oid,
                 points,
             }| Item {
                visual,
                oid,
                points: &self.points[points],
            },
        )
    }

    pub fn iter_ways(&self) -> impl Iterator<Item = Item<&[Point]>> {
        self.ways.iter().cloned().map(
            |Item {
                 visual,
                 oid,
                 points: (start, end),
             }| Item {
                visual,
                oid,
                points: &self.points[start..end],
            },
        )
    }
}

impl Constructable for Tile {
    fn new(bbox: BBox) -> Self {
        Tile {
            min: bbox.min,
            max: bbox.max,
            points: Vec::new(),
            areas: Vec::new(),
            nodes: Vec::new(),
            ways: Vec::new(),
        }
    }

    fn add_area(&mut self, area: &[Point], visual_type: AreaVisualType) {
        let start = self.points.len();
        self.points.extend_from_slice(area);
        let end = self.points.len();
        self.areas.push(Item {
            visual: visual_type as usize,
            oid: 0,
            points: (start, end),
        });
    }

    fn add_node(&mut self, node: Point, visual_type: NodeVisualType) {
        let index = self.points.len();
        self.points.push(node);
        self.nodes.push(Item {
            visual: visual_type as usize,
            oid: 0,
            points: index,
        });
    }

    fn add_way(&mut self, way: &[Point], visual_type: WayVisualType) {
        let start = self.points.len();
        self.points.extend_from_slice(way);
        let end = self.points.len();
        self.ways.push(Item {
            visual: visual_type as usize,
            oid: 0,
            points: (start, end),
        });
    }
}
