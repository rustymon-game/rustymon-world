use crate::generator::WorldGenerator;
use clap::Parser;
use libosmium::handler::{apply_with_areas, AreaAssemblerConfig, Handler};
use nalgebra::Vector2;
use std::ffi::CString;

mod formats;
mod generator;
mod geometry;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// PBF file to process
    #[clap(value_parser)]
    file: String,

    /// Longitude of center
    #[clap(value_parser)]
    center_y: f64,

    /// Latitude of center
    #[clap(value_parser)]
    center_x: f64,

    /// Number of columns
    #[clap(short, long, value_parser, default_value_t = 1)]
    cols: usize,

    /// Number of rows
    #[clap(short, long, value_parser, default_value_t = 1)]
    rows: usize,

    /// Tile's width in degrees
    #[clap(short, long, value_parser, default_value_t = 0.01)]
    degree: f64,
}

fn main() {
    let args = Args::parse();

    let file = CString::new(args.file).expect("File path contained NUL character");

    let step_num = (args.cols, args.rows);
    let step_size = Vector2::new(args.degree, args.degree);
    let center = Vector2::new(args.center_x, args.center_y);

    let mut handler: WorldGenerator<formats::Pytest> =
        WorldGenerator::new(center, step_num, step_size);

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

    serde_json::to_writer(std::io::stdout(), &handler.into_tiles()).expect("Couldn't output");
}
