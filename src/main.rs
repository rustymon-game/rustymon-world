use crate::generator::WorldGenerator;
use clap::Parser;
use libosmium::handler::{apply_with_areas, AreaAssemblerConfig, Handler};
use nalgebra::Vector2;
use std::ffi::CString;

mod formats;
mod generator;
mod geometry;
mod threading;

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

    /// Number of columns [default: 1]
    #[clap(short, long, value_parser)]
    cols: Option<usize>,

    /// Number of rows [default: 1]
    #[clap(short, long, value_parser)]
    rows: Option<usize>,

    /// Tile's width in degrees [default: 0.01]
    #[clap(short, long, value_parser)]
    degree: Option<f64>,
}

fn main() {
    let args = Args::parse();

    let file = CString::new(args.file).expect("File path contained NUL character");

    let step_num = (args.cols.unwrap_or(1), args.rows.unwrap_or(1));
    let step_size = args.degree.unwrap_or(0.01);
    let step_size = Vector2::new(step_size, step_size);
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
