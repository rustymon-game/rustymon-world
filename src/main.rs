use crate::generator::WorldGenerator;
use clap::{Parser, Subcommand};
use libosmium::handler::{apply_with_areas, AreaAssemblerConfig, Handler};
use log::error;
use nalgebra::Vector2;
use std::ffi::CString;

mod formats;
mod generator;
mod geometry;
mod publish;

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(about = "Just publish already parsed tiles")]
    Publish {
        /// Json file to publish
        file: String,

        /// Url to publish to
        url: String,
    },
    Parse {
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
        #[clap(short, long, value_parser, default_value_t = 0.01)]
        degree: f64,

        /// Publish to url instead of printing to stdout
        #[clap(short, long)]
        url: Option<String>,
    },
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    match args.command {
        Commands::Parse {
            file,
            cols,
            rows,
            degree,
            center_x,
            center_y,
            url,
        } => {
            let file = CString::new(file).expect("File path contained NUL character");

            let step_num = (cols, rows);
            let step_size = Vector2::new(degree, degree);
            let center = Vector2::new(center_x, center_y);

            let mut handler: WorldGenerator<formats::Production> =
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

            let tiles = handler.into_tiles();
            if let Some(url) = url {
                publish::publish(&url, tiles);
            } else {
                serde_json::to_writer(std::io::stdout(), &tiles).expect("Couldn't output");
            }
        }
        Commands::Publish { file, url } => {
            let file = match std::fs::File::open(file) {
                Ok(file) => file,
                Err(error) => {
                    error!("Couldn't open json file: {}", error);
                    return;
                }
            };
            match serde_json::from_reader::<_, Vec<formats::Production>>(file) {
                Ok(tiles) => publish::publish(&url, tiles),
                Err(error) => {
                    error!("Couldn't parse json file: {}", error);
                    return;
                }
            }
        }
    }
}
