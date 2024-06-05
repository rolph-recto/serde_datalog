use clap::Parser;
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
    output: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut input: String = String::new();
    match args.filename {
        Some(filename) => {
            input = fs::read_to_string(filename).unwrap()

        },
        None => {
            io::stdin().read_to_string(&mut input).unwrap();
        }
    };

    let mut deserializer = serde_json::Deserializer::from_str(&input);
    let mut extractor = DatalogExtractor::default();
    serde_transcode::transcode(&mut deserializer, &mut extractor).unwrap();

    match args.output {
        Some(output_file) => {
            let outpath = Path::new(&output_file);
            if outpath.is_file() {
                fs::remove_file(&output_file).unwrap();
            }
            extractor.dump_to_db(&output_file).unwrap();
        },

        None => {
            extractor.dump();
        }
    }
}
