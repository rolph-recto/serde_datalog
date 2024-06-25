use serde::Serialize;
use serde_datalog::{backend::souffle_sqlite, DatalogExtractor};
use std::{fs, path::PathBuf};

fn get_example_files(extension: &str) -> Vec<PathBuf> {
    fs::read_dir("./examples/")
        .unwrap()
        .map(|path| path.unwrap().path())
        .filter(|path| match path.extension() {
            Some(path_ext) => path_ext == extension,
            None => false,
        })
        .collect()
}

fn run_example<T: Serialize>(value: T) {
    let souffle_sqlite = souffle_sqlite::StringKeyBackend::default();
    let mut extractor = DatalogExtractor::new(souffle_sqlite);
    let res = value.serialize(&mut extractor);
    assert!(res.is_ok());
}

fn run_examples<T: Serialize>(extension: &str, value_builder: fn(String) -> T) {
    let files = get_example_files(extension);
    if !files.is_empty() {
        println!("discovered {} example .{} file(s)", files.len(), extension);

        for file in get_example_files(extension) {
            println!("running test for file {}", file.display());
            let input = fs::read_to_string(file).unwrap();
            let value = value_builder(input);
            run_example(value);
        }
    } else {
        println!("discovered no .{} tests :(", extension);
    }
}

#[test]
#[cfg(feature = "json")]
fn run_json_examples() {
    run_examples("json", |input| -> serde_json::Value {
        serde_json::from_str(&input).unwrap()
    });
}

#[test]
#[cfg(feature = "ron")]
fn run_ron_examples() {
    run_examples("ron", |input| -> ron::Value {
        ron::from_str(&input).unwrap()
    });
}

#[test]
#[cfg(feature = "toml")]
fn run_toml_examples() {
    run_examples("toml", |input| -> toml::Value {
        toml::from_str(&input).unwrap()
    });
}

#[test]
#[cfg(feature = "yaml")]
fn run_yaml_examples() {
    run_examples("yaml", |input| -> serde_yaml::Value {
        serde_yaml::from_str(&input).unwrap()
    });
}
