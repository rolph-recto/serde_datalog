#[cfg(feature = "json")]
mod test {
    use std::ops::ControlFlow;

    use arbitrary::{Unstructured, Arbitrary};
    use arbitrary_json::ArbitraryValue;
    use rand::RngCore;
    use serde::Serialize;
    use serde_datalog::{DatalogExtractor, backend, DatalogExtractionError};
    use serde_json::Value;

    struct ValueCount {
        null: usize,
        bool: usize,
        number: usize,
        string: usize,
        array: usize,
        object: usize,
        array_elements: usize,
        object_fields: usize,
    }

    impl std::ops::Add<ValueCount> for ValueCount {
        type Output = ValueCount;

        fn add(self, rhs: ValueCount) -> Self::Output {
            ValueCount {
                null: self.null + rhs.null,
                bool: self.bool + rhs.bool,
                number: self.number + rhs.number,
                string: self.string + rhs.string,
                array: self.array + rhs.array,
                object: self.object + rhs.object,
                array_elements: self.array_elements + rhs.array_elements,
                object_fields: self.object_fields + rhs.object_fields,
            }
        }
    }

    impl ValueCount {
        fn new(null: usize, bool: usize, number: usize, string: usize, array: usize, array_elements: usize, object: usize, object_fields: usize) -> Self {
            Self { null, bool, number, string, array, array_elements, object, object_fields }
        }

        fn get(value: &Value) -> Self {
            match value {
                Value::Null => ValueCount::new(1, 0, 0, 0, 0, 0, 0, 0),

                Value::Bool(_) => ValueCount::new(0, 1, 0, 0, 0, 0, 0, 0),

                Value::Number(_) => ValueCount::new(0, 0, 1, 0, 0, 0, 0, 0),

                Value::String(_) => ValueCount::new(0, 0, 0, 1, 0, 0, 0, 0),

                Value::Array(arr) => {
                    arr.iter().fold(ValueCount::new(0, 0, 0, 0, 1, 0, 0, 0), |acc, v| {
                        let c = ValueCount::get(v);
                        let mut res = acc + c;
                        res.array_elements += 1;
                        res
                    })
                }

                Value::Object(map) => {
                    map.iter().fold(ValueCount::new(0, 0, 0, 0, 0, 0, 1, 0), |acc, (_, v)| {
                        let c = ValueCount::get(v);

                        // add 1 string for the key
                        let mut res = acc + c;
                        res.string += 1;
                        res.object_fields += 1;
                        res
                    })
                }
            }
        }

        fn total(&self) -> usize {
            self.null + self.bool + self.number + self.string + self.array + self.object
        }
    }

    fn extract(value: &Value) -> Option<backend::vector::Backend> {
        let mut backend = backend::vector::Backend::default();
        let mut extractor = DatalogExtractor::new(&mut backend);
        let res = value.serialize(&mut extractor);
        drop(extractor);

        return match res {
            Ok(_) => {
                let map_sym = backend.symbol_table.get("Map").unwrap();
                let seq_sym = backend.symbol_table.get("Seq").unwrap();

                let (map_count, seq_count) = 
                    backend.type_table.iter().fold((0, 0), |acc, row| {
                        let map_inc = if row.1 == *map_sym { 1 } else { 0 };
                        let seq_inc = if row.1 == *seq_sym { 1 } else { 0 };
                        (acc.0 + map_inc, acc.1 + seq_inc)
                    });

                let c = ValueCount::get(&value);
                assert!(map_count == c.object);
                assert!(seq_count == c.array);
                assert!(backend.map_table.len() == c.object_fields);
                assert!(backend.seq_table.len() == c.array_elements);
                assert!(backend.bool_table.len() == c.bool);
                assert!(backend.number_table.len() == c.number);
                assert!(backend.string_table.len() == c.string);
                assert!(backend.type_table.len() == c.total());
                Some(backend)
            }

            Err(DatalogExtractionError::UnextractableData) => {
                None
            }

            Err(DatalogExtractionError::Custom(msg)) => {
                assert!(false, "{}", msg);
                None
            }
        };
    }

    #[test]
    fn run_value1() {
        let value: Value = 
            serde_json::Map::from_iter(
                vec![
                    ("test".to_string(), Value::Number(serde_json::Number::from(10)))
                ].into_iter()
            ).into();

        let backend = extract(&value).unwrap();

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

        let backend = extract(&value).unwrap();

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

    #[test]
    fn run_fuzzer() {
        let mut data = [0u8; 16384];
        rand::thread_rng().fill_bytes(&mut data);
        let mut u = Unstructured::new(&data);
        let mut total = 0;
        let mut extracted = 0;

        u.arbitrary_loop(Some(10000), Some(10000), |u| {
            let value = ArbitraryValue::arbitrary(u).unwrap().take();
            if extract(&value).is_some() {
                extracted += 1;
            }
            total += 1;
            Ok(ControlFlow::Continue(()))
        }).unwrap();

        println!("generated {} arbitrary JSON values in total, tested {}", total, extracted);
    }
}