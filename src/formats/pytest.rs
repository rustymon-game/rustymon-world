use nalgebra::Vector2;
use serde::Serialize;
use crate::formats::Constructable;

#[derive(Serialize, Debug)]
pub struct Tile {
    pub areas: Vec<Area>,
    pub nodes: Vec<Node>,
    pub ways: Vec<Way>,
}

pub type Area = Vec<Node>;
pub type Node = (f64, f64);
pub type Way = Vec<Node>;

impl Constructable for Tile {
    fn new() -> Self {
        Tile {
            areas: Vec::new(),
            nodes: Vec::new(),
            ways: Vec::new(),
        }
    }

    fn add_area(&mut self, area: Vec<Vector2<f64>>) {
        self.areas.push(area.into_iter().map(|v| (v.x, v.y)).collect());
    }

    fn add_node(&mut self, node: Vector2<f64>) {
        self.nodes.push((node.x, node.y));
    }

    fn extend_ways(&mut self, ways: impl IntoIterator<Item=Vec<Vector2<f64>>>) {
        for way in ways {
            self.ways.push(way.into_iter().map(|v| (v.x, v.y)).collect());
        }
    }
}