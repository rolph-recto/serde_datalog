//! This crate provides a command-line utility `serdedl` for Serde Datalog that
//! converts from a variety of common data formats into an input EDB for a
//! Datalog program.

use erased_serde::Deserializer as ErasedDeserializer;
use clap::{Parser, ValueEnum};
use serde_json::de::StrRead;
use std::{
    io::{self, Read},
    fs,
    path::Path
};

use serde_datalog::{DatalogExtractor, backends::souffle_sqlite};

trait InputFormat<'a> {
    fn name(&self) -> String;
    fn file_extensions(&self) -> Vec<String>;
    fn create_deserializer<'b>(&'b mut self, contents: &'a str) -> Box<dyn ErasedDeserializer<'a> + 'b>;
}

#[derive(Default)]
struct InputFormatJSON<'a> {
    deserializer: Option<serde_json::de::Deserializer<StrRead<'a>>>
}

impl<'a> InputFormat<'a> for InputFormatJSON<'a> {
    fn name(&self) -> String {
        "json".to_string()
    }

    fn file_extensions(&self) -> Vec<String> {
        vec!["json".to_string()]
    }

    fn create_deserializer<'b>(&'b mut self, contents: &'a str) -> Box<dyn ErasedDeserializer<'a> + 'b> {
        self.deserializer = Some(serde_json::Deserializer::from_str(contents));
        Box::new(<dyn ErasedDeserializer<'a>>::erase(self.deserializer.as_mut().unwrap()))
    }
}

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
    format: Option<String>,

    #[arg(short = 'o', long = "output", help = "File name of output SQLite database")]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut formats = vec![
        InputFormatJSON::default()
    ];

    let (format_auto, input): (Option<String>, String) =
        match &args.filename {
            Some(filename) => {
                let path = Path::new(&filename);
                let ext_opt =
                    path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|s| s.to_string());

                (ext_opt, fs::read_to_string(path).unwrap())
            },

            None => {
                let mut buf = String::new();
                io::stdin().read_to_string(&mut buf).unwrap();
                (None, buf)
            }
        };

    let format_opt: Option<&mut dyn InputFormat<'_>> =
        match (&format_auto, &args.format) {
            (None, None) => None,

            // format specified with -f overrides format from file extension
            (_, Some(name)) => {
                formats.iter_mut()
                .find(|fmt| fmt.name() == *name)
                .map(|fmt| fmt as &mut dyn InputFormat<'_>)
            },

            (Some(ext), None) => {
                formats.iter_mut()
                .find(|fmt| {
                    fmt.file_extensions().iter()
                    .any(|fmt_ext| fmt_ext == ext)
                })
                .map(|fmt| fmt as &mut dyn InputFormat<'_>)
            },
        };

    if let Some(format) = format_opt {
        let mut deserializer = format.create_deserializer(&input);
        let mut souffle_sqlite = souffle_sqlite::Backend::default();

        let mut extractor = DatalogExtractor::new(&mut souffle_sqlite);
        serde_transcode::transcode(deserializer.as_mut(), &mut extractor).unwrap();
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
        println!("Unknown format for input. Accepted input formats:");
        for fmt in formats.iter() {
            println!("- {}", fmt.name());
        }
    }
}
