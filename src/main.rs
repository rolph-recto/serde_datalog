//! This crate provides a command-line utility `serdedl` for Serde Datalog that
//! converts from a variety of common data formats into an input EDB for a
//! Datalog program.

pub mod input_format;

use clap::Parser;
use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    str::FromStr,
};

use serde_datalog::{backend, DatalogExtractor, DatalogExtractorBackend};

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
        help = "List of input files; if absent, will read from standard input.\nFiles must all have the same input format."
    )]
    filenames: Vec<String>,

    #[arg(
        short = 'f',
        long = "format",
        help = "Format of input file; if absent, will guess format from file extensions"
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

fn process_file<B: DatalogExtractorBackend>(
    extractor: &mut DatalogExtractor<B>,
    format: &Box<dyn InputFormat>,
    filename_opt: Option<String>,
    input: String,
) -> Result<(), String> {
    let mut format_data = format.create(&input);
    let mut deserializer = format_data.deserializer();
    let path: String = match &filename_opt {
        Some(file) => PathBuf::from_str(file)
            .unwrap()
            .canonicalize()
            .unwrap()
            .display()
            .to_string(),
        None => "stdin".to_string(),
    };
    extractor.set_file(&path).unwrap();
    serde_transcode::transcode(deserializer.as_mut(), extractor).unwrap();
    Result::Ok(())
}

fn process_files<B: backend::souffle_sqlite::AbstractBackend>(
    backend: B,
    format: &Box<dyn InputFormat>,
    filenames: &Vec<String>,
    output: &Option<String>,
) {
    let mut extractor: DatalogExtractor<B> = DatalogExtractor::new(backend);
    if filenames.len() > 0 {
        for filename in filenames.iter() {
            let path = Path::new(filename);
            let buf = fs::read_to_string(path).unwrap();
            process_file(&mut extractor, format, Some(filename.to_string()), buf).unwrap();
        }
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf).unwrap();

        process_file(&mut extractor, format, None, buf).unwrap();
    };

    if let Some(output_file) = output {
        let outpath = Path::new(&output_file);
        if outpath.is_file() {
            fs::remove_file(output_file).unwrap();
        }
    }

    let souffle_sqlite = extractor.get_backend();
    match output {
        Some(output_file) => {
            souffle_sqlite.dump_to_db(output_file).unwrap();
        }

        None => souffle_sqlite.dump(),
    }
}

fn main() {
    let args = Args::parse();
    let formats: Vec<Box<dyn InputFormat>> = get_input_formats();

    if args.list_formats {
        print_formats(&formats);
        return;
    }

    // assume that all input files are the same format
    let format_res: Result<&Box<dyn InputFormat>, String> = match args.format {
        Some(name) => formats.iter().find(|fmt| fmt.name() == &name).map_or(
            Result::Err(format!("Unknown input format {}", &name)),
            |fmt| Result::Ok(fmt),
        ),

        None => {
            let exts: Vec<Option<String>> = args
                .filenames
                .iter()
                .map(|filename| {
                    Path::new(filename)
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|s| s.to_string())
                })
                .collect();

            let mut ext_opt: Result<String, String> =
                Result::Err("Unknown or missing file extension".to_string());

            for file_ext_opt in exts {
                match file_ext_opt {
                    Some(file_ext) => {
                        if let Result::Ok(ext) = &ext_opt {
                            if ext != &file_ext {
                                let err: String =
                                    format!("Input format must be unique; found extensions {} and {} for different formats",
                                        ext, &file_ext);

                                ext_opt = Result::Err(err);
                                break;
                            }
                        } else {
                            ext_opt = Result::Ok(file_ext);
                        }
                    }

                    None => {
                        ext_opt = Result::Err("Missing file extension".to_string());
                        break;
                    }
                }
            }

            ext_opt.map_or_else(
                |err| Result::Err(err),
                |ext| {
                    formats
                        .iter()
                        .find(|fmt| fmt.file_extensions().contains(&ext.as_str()))
                        .map_or(
                            Result::Err(format!("Unknown file extension {}", &ext)),
                            |fmt| Result::Ok(fmt),
                        )
                },
            )
        }
    };

    match format_res {
        Err(err) => {
            println!("{}", err);
        }

        Ok(format) => {
            if format.has_string_keys() {
                process_files(
                    backend::souffle_sqlite::StringKeyBackend::default(),
                    format,
                    &args.filenames,
                    &args.output,
                );
            } else {
                process_files(
                    backend::souffle_sqlite::Backend::default(),
                    format,
                    &args.filenames,
                    &args.output,
                );
            }
        }
    }
}
