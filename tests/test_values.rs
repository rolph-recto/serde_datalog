#[cfg(feature = "json")]
mod test {
    use std::ops::ControlFlow;

    use arbitrary::{Arbitrary, Unstructured};
    use arbitrary_json::ArbitraryValue;
    use rand::RngCore;
    use serde::Serialize;
    use serde_datalog::{backend, DatalogExtractionError, DatalogExtractor, ElemId};
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
        fn new(
            null: usize,
            bool: usize,
            number: usize,
            string: usize,
            array: usize,
            array_elements: usize,
            object: usize,
            object_fields: usize,
        ) -> Self {
            Self {
                null,
                bool,
                number,
                string,
                array,
                array_elements,
                object,
                object_fields,
            }
        }

        fn get(value: &Value) -> Self {
            match value {
                Value::Null => ValueCount::new(1, 0, 0, 0, 0, 0, 0, 0),

                Value::Bool(_) => ValueCount::new(0, 1, 0, 0, 0, 0, 0, 0),

                Value::Number(_) => ValueCount::new(0, 0, 1, 0, 0, 0, 0, 0),

                Value::String(_) => ValueCount::new(0, 0, 0, 1, 0, 0, 0, 0),

                Value::Array(arr) => {
                    arr.iter()
                        .fold(ValueCount::new(0, 0, 0, 0, 1, 0, 0, 0), |acc, v| {
                            let c = ValueCount::get(v);
                            let mut res = acc + c;
                            res.array_elements += 1;
                            res
                        })
                }

                Value::Object(map) => {
                    map.iter()
                        .fold(ValueCount::new(0, 0, 0, 0, 0, 0, 1, 0), |acc, (_, v)| {
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

    fn extract(value: &Value) -> Option<backend::vector::BackendData<ElemId>> {
        let mut extractor = DatalogExtractor::new(backend::vector::Backend::default());
        let res = value.serialize(&mut extractor);
        let data = extractor.get_backend().get_data();

        return match res {
            Ok(_) => {
                let map_sym = data.symbol_table.get_by_left("Map").unwrap();
                let seq_sym = data.symbol_table.get_by_left("Seq").unwrap();

                let (map_count, seq_count) = data.type_table.iter().fold((0, 0), |acc, row| {
                    let map_inc = if row.1 == map_sym { 1 } else { 0 };
                    let seq_inc = if row.1 == seq_sym { 1 } else { 0 };
                    (acc.0 + map_inc, acc.1 + seq_inc)
                });

                let c = ValueCount::get(value);
                assert!(map_count == c.object);
                assert!(seq_count == c.array);
                assert!(data.map_table.len() == c.object_fields);
                assert!(data.seq_table.len() == c.array_elements);
                assert!(data.bool_table.len() == c.bool);
                assert!(data.number_table.len() == c.number);
                assert!(data.string_table.len() == c.string);
                assert!(data.type_table.len() == c.total());
                Some(data)
            }

            Err(DatalogExtractionError::UnextractableData)
            | Err(DatalogExtractionError::NonuniqueRootElement(_))
            | Err(DatalogExtractionError::NonuniqueIdentifier(_)) => None,

            Err(DatalogExtractionError::Custom(msg)) => {
                assert!(false, "{}", msg);
                None
            }
        };
    }

    #[test]
    fn run_value1() {
        let value: Value = serde_json::Map::from_iter(vec![(
            "test".to_string(),
            Value::Number(serde_json::Number::from(10)),
        )])
        .into();

        let data = extract(&value).unwrap();
        let map_kv = data.map_table.iter().next().unwrap();

        assert!(*data.string_table.keys().next().unwrap() == map_kv.0 .1);
        assert!(*data.number_table.keys().next().unwrap() == *map_kv.1);
    }

    #[test]
    fn run_value2() {
        let value: Value = Value::Array(vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ]);

        let data = extract(&value).unwrap();

        let a_sym = data
            .symbol_table
            .iter()
            .find_map(|(s, id)| if s == "a" { Some(*id) } else { None })
            .unwrap();

        let b_sym = data
            .symbol_table
            .iter()
            .find_map(|(s, id)| if s == "b" { Some(*id) } else { None })
            .unwrap();

        let a_id = data
            .string_table
            .iter()
            .find_map(|(id, sym)| if *sym == a_sym { Some(*id) } else { None })
            .unwrap();

        let b_id = data
            .string_table
            .iter()
            .find_map(|(id, sym)| if *sym == b_sym { Some(*id) } else { None })
            .unwrap();

        let seq_first_elem = data
            .seq_table
            .iter()
            .find_map(|((_, index), val)| if *index == 0 { Some(*val) } else { None })
            .unwrap();

        let seq_second_elem = data
            .seq_table
            .iter()
            .find_map(|((_, index), val)| if *index == 1 { Some(*val) } else { None })
            .unwrap();

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
            match ArbitraryValue::arbitrary(u) {
                Ok(value) => {
                    if extract(&value).is_some() {
                        extracted += 1;
                    }
                    total += 1;
                }

                Err(_) => {}
            };
            Ok(ControlFlow::Continue(()))
        })
        .unwrap();

        println!(
            "generated {} arbitrary JSON values in total, tested {}",
            total, extracted
        );
    }
}
