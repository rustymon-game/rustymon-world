#[cfg(not(feature = "binary"))]
compile_error!("Requires feature: 'binary'");

use clap::{Parser, ValueEnum};
use rustymon_world::{features, parse, Config};

#[derive(ValueEnum, Debug, Copy, Clone, Default)]
pub enum Format {
    #[default]
    Json,

    #[cfg(feature = "message-pack")]
    MessagePack,
}
impl Format {
    pub fn write(
        &self,
        mut writer: impl std::io::Write,
        data: &impl serde::Serialize,
    ) -> Result<(), String> {
        match self {
            Format::Json => serde_json::to_writer(writer, data).map_err(|error| error.to_string()),
            #[cfg(feature = "message-pack")]
            Format::MessagePack => {
                rmp_serde::encode::write(&mut writer, data).map_err(|error| error.to_string())
            }
        }
    }
}

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

    /// Zoom level to produce tiles for
    #[clap(short, long, default_value_t = 14)]
    zoom: u8,

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
        zoom,
        center_x,
        center_y,
        visual,
        format,
    } = Args::parse();

    let visual_config = if let Some(visual) = visual {
        std::fs::read_to_string(visual).map_err(|err| err.to_string())?
    } else {
        include_str!("sample.config").to_string()
    };

    #[cfg(feature = "yada")]
    let visual: features::yada::YadaParser = features::yada::YadaParser::from_file(&visual_config)
        .ok_or_else(|| "Couldn't read config")?;

    #[cfg(not(feature = "yada"))]
    let visual = features::config::ConfigParser::borrowing()
        .parse_file(&visual_config)
        .map_err(|err| format!("{err:?}"))?;

    let config = Config {
        file,
        cols,
        rows,
        center_x,
        center_y,
        zoom,
        visual,
    };

    let tiles = parse(config).map_err(|err| err.to_string());

    format.write(std::io::stdout(), &tiles)?;

    Ok(())
}
