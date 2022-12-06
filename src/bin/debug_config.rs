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

    // The following prints the most interesting branch from sample.config
    if let Some(branch) = config.nodes.get(1) {
        let simple_ast = rustymon_world::features::simplify::simplify(&branch.expr);
        println!("{simple_ast:#?}");
    }
}
