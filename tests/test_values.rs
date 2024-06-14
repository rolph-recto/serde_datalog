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
fn run_value1() {
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

#[test]
fn run_value2() {
    let value: Value = 
        Value::Array(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ]);

    let backend = get_backend(value);

    assert!(backend.seq_table.len() == 2);
    assert!(backend.string_table.len() == 2);
    assert!(backend.type_table.len() == 3);

    let a_sym =
        backend.symbol_table.iter()
        .find_map(|(s, id)|
            if s == "a" { Some(*id) } else { None }
        ).unwrap();

    let b_sym =
        backend.symbol_table.iter()
        .find_map(|(s, id)|
            if s == "b" { Some(*id) } else { None }
        ).unwrap();

    let a_id = 
        backend.string_table.iter()
        .find_map(|(id, sym)|
            if *sym == a_sym { Some(*id) } else { None }
        ).unwrap();

    let b_id = 
        backend.string_table.iter()
        .find_map(|(id, sym)|
            if *sym == b_sym { Some(*id) } else { None }
        ).unwrap();

    let seq_first_elem = 
        backend.seq_table.iter()
        .find_map(|(_, index, val)|
            if *index == 0 { Some(*val) } else { None }
        ).unwrap();

    let seq_second_elem = 
        backend.seq_table.iter()
        .find_map(|(_, index, val)|
            if *index == 1 { Some(*val) } else { None }
        ).unwrap();

    assert!(seq_first_elem == a_id);
    assert!(seq_second_elem == b_id);
}
