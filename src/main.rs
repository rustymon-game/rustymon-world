#[cfg(not(feature = "binary"))]
compile_error!("Please compile the main.rs with the \"binary\" feature");

use crate::features::simple::SimpleVisual;
use crate::generator::WorldGenerator;
use clap::{Parser, Subcommand};
use libosmium::handler::{AreaAssemblerConfig, Handler};
use nalgebra::Vector2;

mod features;
mod formats;
mod generator;
mod geometry;
mod projection;
mod timer;


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// PBF file to parse
    file: String,

    /// Longitude of center
    center_y: f64,

    /// Latitude of center
    center_x: f64,

    /// Number of columns
    #[clap(short, long, value_parser, default_value_t = 1)]
    cols: usize,

    /// Number of rows
    #[clap(short, long, value_parser, default_value_t = 1)]
    rows: usize,

    /// Tile's width in degrees
    #[clap(short, long, default_value_t = 0.01)]
    degree: f64,

    /// Data format when writing to stdout
    #[clap(value_enum, short, long, default_value_t = Default::default())]
    format: Format,

    /// Config for assigning visual types
    #[clap(long)]
    visual: Option<String>,
}

fn main() -> Result<(), String> {
    env_logger::init();

    let Args {
        file,
        cols,
        rows,
        degree,
        center_x,
        center_y,
        url,
        visual,
        format,
    } = Args::parse();

    let step_num = (cols, rows);
    let step_size = Vector2::new(degree, degree);
    let center = Vector2::new(center_x, center_y);

    let visual = if let Some(visual) = visual {
        let file = std::fs::File::open(visual).map_err(|err| err.to_string())?;
        serde_json::from_reader(file).map_err(|err| err.to_string())?
    } else {
        SimpleVisual::default()
    };

    let handler: WorldGenerator<_, formats::MemEff, _> =
        WorldGenerator::new(center, step_num, step_size, visual, projection::Simple);

    // start timer
    let mut handler = timer::Timer::wrap(handler);

    handler
        .apply_with_areas(
            &file,
            AreaAssemblerConfig {
                create_empty_areas: false,
                ..Default::default()
            },
        )
        .map_err(|error| error.into_string().unwrap())?;

    // end timer
    handler.print();
    let handler = handler.unwrap();

    let tiles = handler.into_tiles();

    serde_json::to_writer(std::io::stdout(), &tiles).map_err(|error| error.to_string())?;

    Ok(())
}
