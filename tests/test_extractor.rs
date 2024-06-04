use std::fs;
use serde::Serialize;
use serde_datalog::DatalogExtractor;

#[test]
fn run_examples() {
    let input = fs::read_to_string("examples/test.json").unwrap();
    let value: serde_json::Value = serde_json::from_str(&input).unwrap();
    let mut extractor = DatalogExtractor::new();
    let res = value.serialize(&mut extractor);
    assert!(res.is_ok());
}
