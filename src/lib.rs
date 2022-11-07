use std::cell::RefCell;
use std::marker::PhantomData;

use libosmium::handler::{AreaAssemblerConfig, Handler};
use nalgebra::Vector2;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize, Serializer};

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

    let handler = WorldGenerator::new(center, step_num, zoom, visual, projection::Simple);

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

pub fn convert_format<T, F>(tiles: Vec<formats::MemEff>, convert: F) -> impl Serialize
where
    T: Serialize,
    F: Fn(formats::MemEff) -> T,
{
    SerializableIterator {
        iterator: RefCell::new(Some(tiles.into_iter().map(convert))),
        _result_type: PhantomData,
    }
}

pub struct SerializableIterator<T, I> {
    iterator: RefCell<Option<I>>,
    _result_type: PhantomData<T>,
}
impl<T, I> Serialize for SerializableIterator<T, I>
where
    T: Serialize,
    I: Iterator<Item = T>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // shorthand to convert any T: Display into an S::Error
        macro_rules! error {
            ($msg:expr) => {
                <S::Error as serde::ser::Error>::custom($msg)
            };
        }

        let iterator = self
            .iterator
            .try_borrow_mut()
            .map_err(|err| error!(err.to_string()))?
            .take()
            .ok_or_else(|| error!("Can't serialize a SerializableIterator twice"))?;

        let mut seq = serializer.serialize_seq(None)?;
        for item in iterator {
            seq.serialize_element(&item)?;
        }
        seq.end()
    }
}
