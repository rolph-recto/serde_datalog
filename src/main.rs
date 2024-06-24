//! This crate provides a command-line utility `serdedl` for Serde Datalog that
//! converts from a variety of common data formats into an input EDB for a
//! Datalog program.

pub mod input_format;

use clap::Parser;
use std::{
    fs,
    io::{self, Read},
    path::Path,
};

use serde_datalog::{backend::souffle_sqlite, DatalogExtractor};

use crate::input_format::InputFormat;

#[derive(Parser, Debug)]
#[command(
    version = "0.1.0",
    about,
    long_about = Some("Converts input in a variety of formats to a database of facts.")
)]
struct Args {
    #[arg(
        index = 1,
        help = "List of input files; if absent, will read from standard input"
    )]
    filenames: Vec<String>,

    #[arg(
        short = 'f',
        long = "format",
        help = "Format of input file; if absent, will guess format from file extension"
    )]
    format: Option<String>,

    #[arg(
        short = 'o',
        long = "output",
        help = "File name of output SQLite database"
    )]
    output: Option<String>,

    #[arg(
        short = 'l',
        long = "list-formats",
        help = "Generate a list of supported file formats"
    )]
    list_formats: bool,
}

fn get_input_formats() -> Vec<Box<dyn InputFormat>> {
    let mut formats: Vec<Box<dyn InputFormat>> = Vec::new();

    #[cfg(feature = "json")]
    {
        formats.push(Box::new(input_format::json::InputFormatJSON));
    }

    #[cfg(feature = "ron")]
    {
        formats.push(Box::new(crate::input_format::ron::InputFormatRON));
    }

    #[cfg(feature = "toml")]
    {
        formats.push(Box::new(crate::input_format::toml::InputFormatTOML));
    }

    #[cfg(feature = "yaml")]
    {
        formats.push(Box::new(crate::input_format::yaml::InputFormatYAML));
    }

    formats
}

fn print_formats(formats: &Vec<Box<dyn InputFormat>>) {
    println!("Supported input formats:");
    for fmt in formats.iter() {
        print!("- {} (extensions: ", fmt.name());

        let exts = fmt.file_extensions();
        let mut iter = exts.iter().peekable();

        while let Some(ext) = iter.next() {
            print!(".{}", ext);
            if iter.peek().is_some() {
                print!(", ");
            }
        }

        println!(")");
    }
}

fn main() {
    let args = Args::parse();

    let mut formats: Vec<Box<dyn InputFormat>> = get_input_formats();

    if args.list_formats {
        print_formats(&formats);
        return;
    }

    let input: String =
        if args.filenames.len() > 0 {
            let filename = args.filenames.first().unwrap();
            let path = Path::new(filename);
            fs::read_to_string(path).unwrap()

        } else {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap();
            buf
        };

    let format_auto: Option<String> =
        if args.filenames.len() > 0 {
            Path::new(args.filenames.first().unwrap())
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string())

        } else {
            None
        };

    let format_opt: Option<&mut dyn InputFormat> = match (&format_auto, &args.format) {
        (None, None) => None,

        // format specified with -f overrides format from file extension
        (_, Some(name)) => formats
            .iter_mut()
            .find(|fmt| fmt.name() == *name)
            .map(|fmt| fmt.as_mut() as &mut dyn InputFormat),

        (Some(ext), None) => formats
            .iter_mut()
            .find(|fmt| fmt.file_extensions().iter().any(|fmt_ext| fmt_ext == ext))
            .map(|fmt| fmt.as_mut() as &mut dyn InputFormat),
    };

    if let Some(format) = format_opt {
        let mut format_data = format.create(&input);
        let mut deserializer = format_data.deserializer();
        let mut souffle_sqlite = souffle_sqlite::StringKeyBackend::default();

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
            }

            None => {
                souffle_sqlite.dump();
            }
        }
    } else {
        println!("Unknown format for input.");
        print_formats(&formats);
    }
}
