#[cfg(feature = "json")]
mod test {
    use std::fs;
    use serde::Serialize;
    use serde_datalog::{DatalogExtractor, backend::souffle_sqlite};

    #[test]
    fn run_examples() {
        let input = fs::read_to_string("examples/test.json").unwrap();
        let value: serde_json::Value = serde_json::from_str(&input).unwrap();
        let mut souffle_sqlite = souffle_sqlite::Backend::default();
        let mut extractor = DatalogExtractor::new(&mut souffle_sqlite);
        let res = value.serialize(&mut extractor);
        assert!(res.is_ok());
    }
}
