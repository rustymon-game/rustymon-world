use std::cell::RefCell;
use std::marker::PhantomData;

use crate::features::FeatureParser;
use libosmium::handler::{AreaAssemblerConfig, Handler};
use nalgebra::Vector2;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize, Serializer};

pub mod features;
pub mod formats;
pub mod generator;
pub mod geometry;
pub mod measurements;
pub mod projection;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config<Visual: FeatureParser> {
    pub file: String,
    pub cols: usize,
    pub rows: usize,
    pub center_x: f64,
    pub center_y: f64,
    pub zoom: u8,
    pub visual: Visual,
}

pub fn parse<Visual>(config: Config<Visual>) -> Result<Vec<formats::Tile<usize>>, String>
where
    Visual: FeatureParser<AreaFeature = usize, NodeFeature = usize, WayFeature = usize>,
{
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

    let mut timed_handler = measurements::TimedHandler::new(handler);
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
    let handler = timed_handler.into_handler();

    Ok(handler.into_tiles())
}

pub fn convert_format<T, F>(tiles: Vec<formats::Tile<usize>>, convert: F) -> impl Serialize
where
    T: Serialize,
    F: Fn(formats::Tile<usize>) -> T,
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
