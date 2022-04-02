use std::{env, process};

use ls::{cli, printer};

fn main() {
    let config = cli::Config::new(env::args().collect()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    printer::print(config);
}
