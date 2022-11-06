use libosmium::handler::{AreaAssemblerConfig, Handler};
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

use crate::features::simple::SimpleVisual;
use crate::generator::WorldGenerator;

pub mod features;
pub mod formats;
pub mod generator;
pub mod geometry;
pub mod projection;
pub mod timer;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub file: String,
    pub cols: usize,
    pub rows: usize,
    pub center_x: f64,
    pub center_y: f64,
    pub zoom: u8,
    pub visual: SimpleVisual,
}

pub fn parse(config: Config) -> Result<Vec<formats::MemEff>, String> {
    let Config {
        file,
        cols,
        rows,
        zoom,
        center_x,
        center_y,
        visual,
    } = config;
    let step_num = (cols, rows);
    let center = Vector2::new(center_x, center_y);

    let handler: WorldGenerator<_, formats::MemEff, _> =
        WorldGenerator::new(center, step_num, zoom, visual, projection::Simple);

    let mut timed_handler = timer::Timer::wrap(handler);
    timed_handler
        .apply_with_areas(
            &file,
            AreaAssemblerConfig {
                create_empty_areas: false,
                ..Default::default()
            },
        )
        .map_err(|error| error.into_string().unwrap())?;
    timed_handler.print();

    let handler = timed_handler.unwrap();

    Ok(handler.into_tiles())
}
