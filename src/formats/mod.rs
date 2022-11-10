use serde::{Deserialize, Serialize};

use crate::geometry::{BBox, Point};

#[derive(Serialize, Deserialize, Clone)]
pub struct Tile<Feature> {
    pub min: Point,
    pub max: Point,

    pub areas: Vec<Item<Feature, (usize, usize)>>,
    pub nodes: Vec<Item<Feature, usize>>,
    pub ways: Vec<Item<Feature, (usize, usize)>>,

    /// Common pool of points used by all areas, nodes and ways
    pub points: Vec<Point>,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Item<Feature, Index> {
    pub feature: Feature,
    pub oid: usize,

    /// Ether `usize` for nodes or `(usize, usize)` defining a range for areas and ways.
    pub points: Index,
}

/// Implements iterators hiding the flattened points
impl<Feature> Tile<Feature> {
    pub fn iter_areas(&self) -> impl Iterator<Item = Item<&Feature, &[Point]>> {
        self.areas.iter().map(
            |Item {
                 feature,
                 oid,
                 points: (start, end),
             }| Item {
                feature,
                oid: *oid,
                points: &self.points[*start..*end],
            },
        )
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = Item<&Feature, &Point>> {
        self.nodes.iter().map(
            |Item {
                 feature,
                 oid,
                 points,
             }| Item {
                feature,
                oid: *oid,
                points: &self.points[*points],
            },
        )
    }

    pub fn iter_ways(&self) -> impl Iterator<Item = Item<&Feature, &[Point]>> {
        self.ways.iter().map(
            |Item {
                 feature,
                 oid,
                 points: (start, end),
             }| Item {
                feature,
                oid: *oid,
                points: &self.points[*start..*end],
            },
        )
    }
}

/// Implement construction process
impl Tile<usize> {
    pub fn new(bbox: BBox) -> Self {
        Tile {
            min: bbox.min,
            max: bbox.max,
            points: Vec::new(),
            areas: Vec::new(),
            nodes: Vec::new(),
            ways: Vec::new(),
        }
    }

    pub fn add_area(&mut self, area: &[Point], feature: usize) {
        let start = self.points.len();
        self.points.extend_from_slice(area);
        let end = self.points.len();
        self.areas.push(Item {
            feature,
            oid: 0,
            points: (start, end),
        });
    }

    pub fn add_node(&mut self, node: Point, feature: usize) {
        let index = self.points.len();
        self.points.push(node);
        self.nodes.push(Item {
            feature,
            oid: 0,
            points: index,
        });
    }

    pub fn add_way(&mut self, way: &[Point], feature: usize) {
        let start = self.points.len();
        self.points.extend_from_slice(way);
        let end = self.points.len();
        self.ways.push(Item {
            feature,
            oid: 0,
            points: (start, end),
        });
    }
}
