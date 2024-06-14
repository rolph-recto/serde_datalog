use std::ops::Add;

use serde::Serialize;
use serde_datalog::{DatalogExtractor, backend};
use serde_json::Value;

struct ValueCount {
    bool: usize,
    number: usize,
    string: usize,
    array: usize,
    object: usize,
}

impl std::ops::Add<ValueCount> for ValueCount {
    type Output = ValueCount;

    fn add(self, rhs: ValueCount) -> Self::Output {
        ValueCount {
            bool: self.bool + rhs.bool,
            number: self.number + rhs.number,
            string: self.string + rhs.string,
            array: self.array + rhs.array,
            object: self.object + rhs.object,
        }
    }
}

impl ValueCount {
    fn new(bool: usize, number: usize, string: usize, array: usize, object: usize) -> Self {
        Self { bool, number, string, array, object }
    }

    fn get(value: &Value) -> Self {
        match value {
            Value::Null => ValueCount::new(0, 0, 0, 0, 0),

            Value::Bool(_) => ValueCount::new(1, 0, 0, 0, 0),

            Value::Number(_) => ValueCount::new(0, 1, 0, 0, 0),

            Value::String(_) => ValueCount::new(0, 0, 1, 0, 0),

            Value::Array(arr) => {
                arr.iter().fold(ValueCount::new(0, 0, 0, 1, 0), |acc, v| {
                    let c = ValueCount::get(v);
                    acc + c
                })
            }

            Value::Object(map) => {
                map.iter().fold(ValueCount::new(0, 0, 0, 0, 1), |acc, (_, v)| {
                    let c = ValueCount::get(v);

                    // add 1 string for the key
                    let mut res = acc + c;
                    res.string += 1;
                    res
                })
            }
        }
    }

    fn total(&self) -> usize {
        self.bool + self.number + self.string + self.array + self.object
    }
}

fn get_backend(value: &Value) -> backend::vector::Backend {
    let mut backend = backend::vector::Backend::default();
    let mut extractor = DatalogExtractor::new(&mut backend);
    let res = value.serialize(&mut extractor);
    drop(extractor);
    assert!(res.is_ok());

    let c = ValueCount::get(&value);

    assert!(backend.map_table.len() == c.object);
    assert!(backend.number_table.len() == c.number + c.bool);
    assert!(backend.string_table.len() == c.string);
    assert!(backend.type_table.len() == c.total());

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

    let backend = get_backend(&value);

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

    let backend = get_backend(&value);

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
