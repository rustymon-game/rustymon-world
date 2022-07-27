use serde::Serialize;

#[derive(Serialize, Debug)]
struct Tile {
    pub bbox: [f64; 4],
    pub streets: Vec<Street>,
    pub poi: Vec<PointOfInterest>,
    pub areas: Vec<Area>,
}

#[derive(Serialize, Debug)]
struct PointOfInterest {
    pub spawns: Vec<usize>,
    #[serde(rename = "type")]
    pub ty: usize,
    pub point: [f64; 2],
    pub oid: usize,
}

#[derive(Serialize, Debug)]
struct Street {
    #[serde(rename = "type")]
    pub ty: usize,
    pub points: Vec<[f64; 2]>,
    pub oid: usize,
}

#[derive(Serialize, Debug)]
struct Area {
    pub spawns: Vec<usize>,
    #[serde(rename = "type")]
    pub ty: usize,
    pub points: Vec<[f64; 2]>,
    pub oid: usize,
}