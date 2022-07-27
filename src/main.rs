use crate::formats::{pytest, Constructable};
use crate::geometry::BBox;
use clap::Parser;
use nalgebra::Vector2;
use osmium::area::Area;
use osmium::handler::{apply_with_areas, AreaAssemblerConfig, Handler};
use osmium::node::{Node, NodeRef};
use osmium::way::Way;
use std::ffi::CString;

mod formats;
mod geometry;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// PBF file to process
    #[clap(value_parser)]
    file: String,

    #[clap(value_parser)]
    min_x: f64,
    #[clap(value_parser)]
    min_y: f64,
    #[clap(value_parser)]
    max_x: f64,
    #[clap(value_parser)]
    max_y: f64,
}

struct WorldGenerator<T: Constructable> {
    bbox: BBox,
    constructing: T,
}
impl<T: Constructable> WorldGenerator<T> {
    pub fn new(bbox: BBox) -> WorldGenerator<T> {
        WorldGenerator {
            bbox,
            constructing: T::new(),
        }
    }
}
impl<T: Constructable> Handler for WorldGenerator<T> {
    fn area(&mut self, area: &Area) {
        for ring in area.outer_rings() {
            let polygon = ring
                .iter()
                .map(NodeRef::get_location)
                .flatten()
                .map(|l| Vector2::new(l.lon(), l.lat()));
            let polygon = self.bbox.clip_polygon(polygon);
            if polygon.len() > 0 {
                self.constructing.add_area(polygon);
            }
        }
    }

    fn node(&mut self, node: &Node) {
        let location = node.location();
        if location.is_defined() && location.is_valid() {
            let point = Vector2::new(location.lon(), location.lat());
            if self.bbox.contains(point) {
                self.constructing.add_node(point);
            }
        }
    }

    fn way(&mut self, way: &Way) {
        let path = way
            .nodes()
            .iter()
            .map(NodeRef::get_location)
            .flatten()
            .map(|l| Vector2::new(l.lon(), l.lat()));
        self.constructing.extend_ways(self.bbox.clip_path(path));
    }
}

fn main() {
    let args = Args::parse();

    let file = CString::new(args.file).expect("File path contained NUL character");

    let bbox = BBox {
        min: Vector2::new(args.min_x, args.min_y),
        max: Vector2::new(args.max_x, args.max_y),
    };
    let mut handler: WorldGenerator<pytest::Tile> = WorldGenerator::new(bbox);

    unsafe {
        apply_with_areas(
            handler.as_table(),
            file.as_ptr(),
            AreaAssemblerConfig {
                create_empty_areas: false,
                ..Default::default()
            },
        );
    }

    serde_json::to_writer(std::io::stdout(), &handler.constructing).expect("Couldn't output");
}
