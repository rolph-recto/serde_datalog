//! This crate provides a command-line utility `serdedl` for Serde Datalog that
//! converts from a variety of common data formats into an input EDB for a
//! Datalog program.

use clap::Parser;
use std::{
    io::{self, Read},
    fs,
    path::Path
};

use serde_datalog::{DatalogExtractor, backend::souffle_sqlite};

use input_format::InputFormat;

pub mod input_format {
    use erased_serde::Deserializer as ErasedDeserializer;

    pub trait InputFormat {
        fn name(&self) -> &'static str;
        fn file_extensions(&self) -> Vec<&'static str>;
        fn create<'input>(&self, contents: &'input str) -> Box<dyn InputFormatData<'input> + 'input>;
    }

    pub trait InputFormatData<'input> {
        fn deserializer<'de>(&'de mut self) -> Box<dyn ErasedDeserializer<'input> + 'de>;
    }

    #[cfg(feature = "json")]
    pub mod json {
        use erased_serde::Deserializer as ErasedDeserializer;
        use serde_json::de::StrRead;
        use super::{InputFormat, InputFormatData};

        pub struct InputFormatJSON;

        impl InputFormat for InputFormatJSON {
            fn name(&self) -> &'static str {
                "json"
            }

            fn file_extensions(&self) -> Vec<&'static str> {
                vec!["json"]
            }

            fn create<'input>(&self, contents: &'input str) -> Box<dyn InputFormatData<'input> + 'input> {
                Box::new(InputFormatJSONData {
                    deserializer: serde_json::Deserializer::from_str(contents)
                })
            }
        }

        struct InputFormatJSONData<'input> {
            deserializer: serde_json::de::Deserializer<StrRead<'input>>
        }

        impl<'input> InputFormatData<'input> for InputFormatJSONData<'input> {
            fn deserializer<'de>(&'de mut self) -> Box<dyn ErasedDeserializer<'input> + 'de> {
                Box::new(<dyn ErasedDeserializer<'input>>::erase(&mut self.deserializer))
            }
        }
    }
    
    #[cfg(feature = "toml")]
    pub mod toml {
        use erased_serde::Deserializer as ErasedDeserializer;
        use super::{InputFormat, InputFormatData};

        pub struct InputFormatTOML;

        impl InputFormat for InputFormatTOML {
            fn name(&self) -> &'static str {
                "toml"
            }

            fn file_extensions(&self) -> Vec<&'static str> {
                vec!["toml"]
            }

            fn create<'input>(&self, contents: &'input str) -> Box<dyn InputFormatData<'input> + 'input> {
                Box::new(InputFormatDataTOML { contents })
            }
        }

        pub struct InputFormatDataTOML<'a> {
            contents: &'a str
        }

        impl<'input> InputFormatData<'input> for InputFormatDataTOML<'input> {
            fn deserializer<'de>(&'de mut self) -> Box<dyn ErasedDeserializer<'input> + 'de> {
                Box::new(<dyn ErasedDeserializer<'input>>::erase(toml::Deserializer::new(self.contents)))
            }
        }
    }

    #[cfg(feature = "ron")]
    pub mod ron {
        use erased_serde::Deserializer as ErasedDeserializer;
        use super::{InputFormat, InputFormatData};

        pub struct InputFormatRON;

        impl InputFormat for InputFormatRON {
            fn name(&self) -> &'static str {
                "ron"
            }

            fn file_extensions(&self) -> Vec<&'static str> {
                vec!["ron"]
            }

            fn create<'input>(&self, contents: &'input str) -> Box<dyn InputFormatData<'input> + 'input> {
                Box::new(InputFormatDataRON { 
                    deserializer: ron::Deserializer::from_str(contents).unwrap()
                })
            }
        }

        pub struct InputFormatDataRON<'input> {
            deserializer: ron::Deserializer<'input>
        }

        impl<'input> InputFormatData<'input> for InputFormatDataRON<'input> {
            fn deserializer<'de>(&'de mut self) -> Box<dyn ErasedDeserializer<'input> + 'de> {
                Box::new(<dyn ErasedDeserializer<'input>>::erase(&mut self.deserializer))
            }
        }
    }

    #[cfg(feature = "yaml")]
    pub mod yaml {
        use erased_serde::Deserializer as ErasedDeserializer;
        use super::{InputFormat, InputFormatData};

        pub struct InputFormatYAML;

        impl InputFormat for InputFormatYAML {
            fn name(&self) -> &'static str {
                "yaml"
            }

            fn file_extensions(&self) -> Vec<&'static str> {
                vec!["yaml", "yml"]
            }

            fn create<'input>(&self, contents: &'input str) -> Box<dyn InputFormatData<'input> + 'input> {
                Box::new(InputFormatDataYAML { contents })
            }
        }

        pub struct InputFormatDataYAML<'input> {
            contents: &'input str
        }

        impl<'input> InputFormatData<'input> for InputFormatDataYAML<'input> {
            fn deserializer<'de>(&'de mut self) -> Box<dyn ErasedDeserializer<'input> + 'de> {
                Box::new(<dyn ErasedDeserializer<'input>>::erase(serde_yaml::Deserializer::from_str(self.contents)))
            }
        }
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

fn get_input_formats() -> Vec<Box<dyn InputFormat>> {
    let mut formats: Vec<Box<dyn InputFormat>> = Vec::new();

    #[cfg(feature = "json")]
    {
        formats.push(Box::new(input_format::json::InputFormatJSON));
    }

    #[cfg(feature = "ron")]
    {
        formats.push(Box::new(input_format::ron::InputFormatRON));
    }
    
    #[cfg(feature = "toml")]
    {
        formats.push(Box::new(input_format::toml::InputFormatTOML));
    }

    #[cfg(feature = "yaml")]
    {
        formats.push(Box::new(input_format::yaml::InputFormatYAML));
    }

    formats
}

fn main() {
    let args = Args::parse();

    let input: String =
        match &args.filename {
            Some(filename) => {
                let path = Path::new(filename);
                fs::read_to_string(path).unwrap()
            }

            None => {
                let mut buf = String::new();
                io::stdin().read_to_string(&mut buf).unwrap();
                buf
            }
        };

    let mut formats: Vec<Box<dyn InputFormat>> = get_input_formats();

    let format_auto: Option<String> =
        match &args.filename {
            Some(filename) => {
                Path::new(filename).extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s.to_string())
            },

            None => None,
        };

    let format_opt: Option<&mut dyn InputFormat> =
        match (&format_auto, &args.format) {
            (None, None) => None,

            // format specified with -f overrides format from file extension
            (_, Some(name)) => {
                formats.iter_mut()
                .find(|fmt| fmt.name() == *name)
                .map(|fmt| fmt.as_mut() as &mut dyn InputFormat)
            },

            (Some(ext), None) => {
                formats.iter_mut()
                .find(|fmt| {
                    fmt.file_extensions().iter()
                    .any(|fmt_ext| fmt_ext == ext)
                })
                .map(|fmt| fmt.as_mut() as &mut dyn InputFormat)
            },
        };

    if let Some(format) = format_opt {
        let mut format_data = format.create(&input);
        let mut deserializer = format_data.deserializer();
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
