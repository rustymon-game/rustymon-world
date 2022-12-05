use rustymon_world::features::config;

fn main() {
    let Some(file) = std::env::args().skip(1).next() else {
        eprintln!("Missing config file");
        return;
    };
    let content = match std::fs::read_to_string(file) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("Unable to read file:\n{err}");
            return;
        }
    };
    let config = match config::ConfigParser::borrowing().parse_file(&content) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("Unable to parse file:\n{err}");
            return;
        }
    };
    println!("{config:#?}");
}
