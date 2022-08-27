use super::Constructable;
use crate::formats::{AreaVisualType, NodeVisualType, WayVisualType};
use crate::geometry::BBox;
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Tile {
    #[serde(rename = "box")]
    pub bbox: [f64; 4],
    pub ways: Vec<Way>,
    pub nodes: Vec<Node>,
    pub areas: Vec<Area>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    pub spawns: Vec<usize>,
    #[serde(rename = "type")]
    pub ty: usize,
    pub point: Point,
    pub oid: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Way {
    #[serde(rename = "type")]
    pub ty: usize,
    pub points: Vec<Point>,
    pub oid: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Area {
    pub spawns: Vec<usize>,
    #[serde(rename = "type")]
    pub ty: usize,
    pub points: Vec<Point>,
    pub oid: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Constructable for Tile {
    fn new(bbox: BBox) -> Self {
        Tile {
            bbox: [bbox.min.x, bbox.min.y, bbox.max.x, bbox.max.y],
            ways: Vec::new(),
            nodes: Vec::new(),
            areas: Vec::new(),
        }
    }

    fn add_area(&mut self, area: Vec<Vector2<f64>>, visual_type: AreaVisualType) {
        self.areas.push(Area {
            spawns: Vec::new(),
            ty: visual_type as usize,
            points: area.into_iter().map(|v| Point { x: v.x, y: v.y }).collect(),
            oid: 0,
        });
    }

    fn add_node(&mut self, node: Vector2<f64>, visual_type: NodeVisualType) {
        self.nodes.push(Node {
            spawns: Vec::new(),
            ty: visual_type as usize,
            point: Point {
                x: node.x,
                y: node.y,
            },
            oid: 0,
        });
    }

    fn add_way(&mut self, way: Vec<crate::geometry::Point>, visual_type: WayVisualType) {
        self.ways.push(Way {
            ty: visual_type as usize,
            points: way.into_iter().map(|v| Point { x: v.x, y: v.y }).collect(),
            oid: 0,
        });
    }
}
