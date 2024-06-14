use serde::Serialize;
use serde_datalog::{DatalogExtractor, backend};
use serde_json::Value;

fn get_backend(value: Value) -> backend::vector::Backend {
    let mut backend = backend::vector::Backend::default();
    let mut extractor = DatalogExtractor::new(&mut backend);
    let res = value.serialize(&mut extractor);
    drop(extractor);
    assert!(res.is_ok());
    backend
}

#[test]
fn run_values() {
    let value: Value = 
        serde_json::Map::from_iter(
            vec![
                ("test".to_string(), Value::Number(serde_json::Number::from(10)))
            ].into_iter()
        ).into();

    let backend = get_backend(value);

    assert!(backend.map_table.len() == 1);
    assert!(backend.number_table.len() == 1);
    assert!(backend.string_table.len() == 1);
    assert!(backend.type_table.len() == 3);

    let map_key = backend.map_table.first().unwrap().1;
    let map_value = backend.map_table.first().unwrap().2;
    
    assert!(backend.string_table.first().unwrap().0 == map_key);
    assert!(backend.number_table.first().unwrap().0 == map_value);
}
