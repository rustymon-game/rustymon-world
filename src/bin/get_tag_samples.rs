#[cfg(not(feature = "message-pack"))]
compile_error!("Requires feature: 'message-pack'");

use std::env;
use std::io::stdout;

use libosmium::handler::{AreaAssemblerConfig, Handler};
use libosmium::tag_list::OwnedTagList;
use libosmium::{Area, Node, Way};
use serde::Serialize;

#[derive(Serialize, Default)]
pub struct Samples {
    pub size: usize,
    pub areas: Vec<OwnedTagList>,
    pub nodes: Vec<OwnedTagList>,
    pub ways: Vec<OwnedTagList>,
}

impl Handler for Samples {
    fn area(&mut self, area: &Area) {
        if self.areas.len() < self.size {
            self.areas.push(area.tags().into());
        }
    }

    fn node(&mut self, node: &Node) {
        if self.nodes.len() < self.size {
            self.nodes.push(node.tags().into());
        }
    }

    fn way(&mut self, way: &Way) {
        if self.ways.len() < self.size {
            self.ways.push(way.tags().into());
        }
    }
}

fn main() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let file = args
        .next()
        .ok_or_else(|| "expected a file as argument".to_string())?;
    let size = if let Some(size) = args.next() {
        size.parse::<usize>().map_err(|err| err.to_string())?
    } else {
        100
    };

    let mut samples = Samples {
        size,
        ..Default::default()
    };
    samples
        .apply_with_areas(
            &file,
            AreaAssemblerConfig {
                ..Default::default()
            },
        )
        .map_err(|err| err.to_string_lossy().to_string())?;

    rmp_serde::encode::write(&mut stdout(), &samples).map_err(|err| err.to_string())?;

    Ok(())
}
