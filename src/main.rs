use crate::features::simple::SimpleVisual;
use crate::generator::WorldGenerator;
use clap::{Parser, Subcommand};
use libosmium::handler::{AreaAssemblerConfig, Handler};
use nalgebra::Vector2;

mod features;
mod formats;
mod generator;
mod geometry;
mod publish;
mod timer;

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(about = "Just publish already parsed tiles")]
    Publish {
        /// Json file to publish
        file: String,

        /// Url to publish to
        url: String,
    },
    #[clap(about = "Parse a PBF file")]
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
        #[clap(short, long, default_value_t = 0.01)]
        degree: f64,

        /// Publish to url instead of printing to stdout
        #[clap(short, long)]
        url: Option<String>,

        /// Config for assigning visual types
        #[clap(long)]
        visual: Option<String>,
    },
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

fn main() -> Result<(), String> {
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
            visual,
        } => {
            let step_num = (cols, rows);
            let step_size = Vector2::new(degree, degree);
            let center = Vector2::new(center_x, center_y);

            let visual = if let Some(visual) = visual {
                let file = std::fs::File::open(visual).map_err(|err| err.to_string())?;
                serde_json::from_reader(file).map_err(|err| err.to_string())?
            } else {
                SimpleVisual::default()
            };

            let mut handler: WorldGenerator<formats::Production, SimpleVisual> =
                WorldGenerator::new(center, step_num, step_size, visual);

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
            let mut handler = handler.unwrap();

            let tiles = handler.into_tiles();
            if let Some(url) = url {
                publish::publish(&url, tiles);
            } else {
                serde_json::to_writer(std::io::stdout(), &tiles)
                    .map_err(|error| error.to_string())?;
            }
        }
        Commands::Publish { file, url } => {
            let file = std::fs::File::open(file).map_err(|error| error.to_string())?;
            let tiles = serde_json::from_reader::<_, Vec<formats::Production>>(file)
                .map_err(|error| error.to_string())?;
            publish::publish(&url, tiles);
        }
    }
    Ok(())
}
