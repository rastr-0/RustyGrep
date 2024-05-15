use std::env;
use std::process;
use RustyGrep::{Config, run};

fn main() {
    // parse command line arguments
    // passing env::args to build function is possible, because it takes as a parameter
    // impl Iterator<Item = String> which is a trait of Iterator
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments | {}", err);
        process::exit(1);
    });
    print!("searching for \"{}\" ", config.query);
    println!("in the file: {}", config.file_path);

    if let Err(e) = run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    };
}
