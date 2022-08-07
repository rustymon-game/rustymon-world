use super::Constructable;
use crate::geometry::BBox;
use nalgebra::Vector2;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Tile {
    #[serde(rename = "box")]
    pub bbox: [f64; 4],
    pub ways: Vec<Way>,
    pub nodes: Vec<Node>,
    pub areas: Vec<Area>,
}

#[derive(Serialize, Debug)]
pub struct Node {
    pub spawns: Vec<usize>,
    #[serde(rename = "type")]
    pub ty: usize,
    pub point: Point,
    pub oid: usize,
}

#[derive(Serialize, Debug)]
pub struct Way {
    #[serde(rename = "type")]
    pub ty: usize,
    pub points: Vec<Point>,
    pub oid: usize,
}

#[derive(Serialize, Debug)]
pub struct Area {
    pub spawns: Vec<usize>,
    #[serde(rename = "type")]
    pub ty: usize,
    pub points: Vec<Point>,
    pub oid: usize,
}

#[derive(Serialize, Debug)]
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

    fn add_area(&mut self, area: Vec<Vector2<f64>>) {
        self.areas.push(Area {
            spawns: Vec::new(),
            ty: 0,
            points: area.into_iter().map(|v| Point { x: v.x, y: v.y }).collect(),
            oid: 0,
        });
    }

    fn add_node(&mut self, node: Vector2<f64>) {
        self.nodes.push(Node {
            spawns: Vec::new(),
            ty: 0,
            point: Point {
                x: node.x,
                y: node.y,
            },
            oid: 0,
        });
    }

    fn extend_ways(&mut self, ways: impl IntoIterator<Item = Vec<Vector2<f64>>>) {
        for way in ways {
            self.ways.push(Way {
                ty: 0,
                points: way.into_iter().map(|v| Point { x: v.x, y: v.y }).collect(),
                oid: 0,
            });
        }
    }
}
