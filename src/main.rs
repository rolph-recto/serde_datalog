use clap::Parser;
use serde::Serialize;
use serde_json::Value;
use std::{
    io::{self, Read},
    fs,
    path::Path
};

use serde_datalog::DatalogExtractor;

#[derive(Parser, Debug)]
#[command(
    version = "0.1.0",
    about,
    long_about = Some("Converts input in a variety of formats to a database of facts.")
)]
struct Args {
    #[arg(
        index = 1,
        help = "(Optional) Input file; if absent, will read from standard input"
    )]
    filename: Option<String>,

    #[arg(short = 'o', long = "output", help = "File name of output SQLite database")]
    output: String,
}

fn main() {
    let args = Args::parse();

    let mut input: String = String::new();
    match args.filename {
        Some(filename) => {
            input = fs::read_to_string(&filename).unwrap()

        },
        None => {
            io::stdin().read_to_string(&mut input).unwrap();
        }
    };

    let value: Value = serde_json::from_str(&input).unwrap();
    let mut extractor = DatalogExtractor::new();
    value.serialize(&mut extractor).unwrap();

    let outpath = Path::new(&args.output);
    if outpath.is_file() {
        fs::remove_file(&args.output).unwrap();
    }

    extractor.dump_to_db(&args.output).unwrap();
}
