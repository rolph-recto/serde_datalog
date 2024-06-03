use clap::Parser;
use serde::Serialize;
use serde_json::Value;
use std::{io::{self, Read}, fs};

use serde_datalog::DatalogExtractor;

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let value: Value = serde_json::from_str(&input).unwrap();
    let mut extractor = DatalogExtractor::new();
    value.serialize(&mut extractor).unwrap();
    let _ = fs::remove_file("json2.db");
    extractor.dump();
    extractor.dump_to_db("json2.db").unwrap();
}
