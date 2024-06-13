//! This crate provides a command-line utility `serdedl` for Serde Datalog that
//! converts from a variety of common data formats into an input EDB for a
//! Datalog program.

use clap::{Parser, ValueEnum, CommandFactory};
use std::{
    io::{self, Read},
    fs,
    path::Path
};

use serde_datalog::{DatalogExtractor, backends::souffle_sqlite};

#[derive(Debug, Clone, ValueEnum)]
enum InputFormat { JSON, TOML, RON, YAML, SEXPR }

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

    #[arg(short = 'f', long = "format", help = "Format of input file")]
    format: Option<InputFormat>,

    #[arg(short = 'o', long = "output", help = "File name of output SQLite database")]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut input: String = String::new();
    let mut autoformat: Option<InputFormat> = None;
    match &args.filename {
        Some(filename) => {
            let path = Path::new(&filename);

            if let Some(ext) = path.extension() {
                if ext == "json" {
                    autoformat = Some(InputFormat::JSON);

                } else if ext == "toml"{
                    autoformat = Some(InputFormat::TOML);

                } else if ext == "ron"{
                    autoformat = Some(InputFormat::RON);

                } else if ext == "yaml" || ext == "yml" {
                    autoformat = Some(InputFormat::YAML);
                }
            }

            input = fs::read_to_string(path).unwrap()
        },
        None => {
            io::stdin().read_to_string(&mut input).unwrap();
        }
    };

    let format_opt: Option<InputFormat> = 
        match (&autoformat, &args.format) {
            (None, None) => None,
            (_, None) => autoformat.clone(),
            (_, Some(_)) => args.format.clone()
        };

    if let Some(format) = format_opt {
        let mut deserializer =
            match format {
                InputFormat::JSON | InputFormat::TOML | InputFormat::RON |
                InputFormat::YAML | InputFormat::SEXPR => {
                    serde_json::Deserializer::from_str(&input)
                }
            };

        let mut souffle_sqlite = souffle_sqlite::Backend::default();

        let mut extractor = DatalogExtractor::new(&mut souffle_sqlite);
        serde_transcode::transcode(&mut deserializer, &mut extractor).unwrap();
        drop(extractor);

        match args.output {
            Some(output_file) => {
                let outpath = Path::new(&output_file);
                if outpath.is_file() {
                    fs::remove_file(&output_file).unwrap();
                }
                souffle_sqlite.dump_to_db(&output_file).unwrap();
            },

            None => {
                souffle_sqlite.dump();
            }
        }

    } else {
        println!("Unknown format for input");
    }
}
