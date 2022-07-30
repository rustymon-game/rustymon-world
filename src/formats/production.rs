use super::Constructable;
use crate::geometry::BBox;
use nalgebra::Vector2;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Tile {
    pub bbox: [f64; 4],
    pub streets: Vec<Street>,
    pub poi: Vec<PointOfInterest>,
    pub areas: Vec<Area>,
}

#[derive(Serialize, Debug)]
pub struct PointOfInterest {
    pub spawns: Vec<usize>,
    #[serde(rename = "type")]
    pub ty: usize,
    pub point: [f64; 2],
    pub oid: usize,
}

#[derive(Serialize, Debug)]
pub struct Street {
    #[serde(rename = "type")]
    pub ty: usize,
    pub points: Vec<[f64; 2]>,
    pub oid: usize,
}

#[derive(Serialize, Debug)]
pub struct Area {
    pub spawns: Vec<usize>,
    #[serde(rename = "type")]
    pub ty: usize,
    pub points: Vec<[f64; 2]>,
    pub oid: usize,
}

impl Constructable for Tile {
    fn new(bbox: BBox) -> Self {
        Tile {
            bbox: [bbox.min.x, bbox.min.y, bbox.max.x, bbox.max.y],
            streets: Vec::new(),
            poi: Vec::new(),
            areas: Vec::new(),
        }
    }

    fn add_area(&mut self, area: Vec<Vector2<f64>>) {
        self.areas.push(Area {
            spawns: Vec::new(),
            ty: 0,
            points: area.into_iter().map(|v| [v.x, v.y]).collect(),
            oid: 0,
        });
    }

    fn add_node(&mut self, node: Vector2<f64>) {
        self.poi.push(PointOfInterest {
            spawns: Vec::new(),
            ty: 0,
            point: [node.x, node.y],
            oid: 0,
        });
    }

    fn extend_ways(&mut self, ways: impl IntoIterator<Item = Vec<Vector2<f64>>>) {
        for way in ways {
            self.streets.push(Street {
                ty: 0,
                points: way.into_iter().map(|v| [v.x, v.y]).collect(),
                oid: 0,
            });
        }
    }
}
