use std::env;
use std::process;
use untitled::{Config, run};

fn main() {
    // get command line arguments with std::end::args
    let args: Vec<String> = env::args().collect();
    // parse command line arguments
    let config = Config::build(&args).unwrap_or_else(|err| {
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
