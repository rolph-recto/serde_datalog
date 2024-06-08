use std::collections::HashMap;

use crate::{ElemId, DatalogExtractorBackend, ElemType, Result, DatalogExtractionError};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct SymbolId(usize);

/// DatalogExtractorBackend impl that stores facts in a SQLite database.
/// The database conforms to the input format for [Souffle](https://souffle-lang.github.io/),
/// a high-performance Datalog implementation.
pub struct Backend {
    cur_symbol_id: SymbolId,
    symbol_table: HashMap<String, SymbolId>,

    // (elem, elem type)
    type_table: Vec<(ElemId, SymbolId)>,

    // (elem, value)
    number_table: Vec<(ElemId, isize)>,

    // (elem, symbol)
    string_table: Vec<(ElemId, SymbolId)>,

    // (elem, key, value)
    map_table: Vec<(ElemId, ElemId, ElemId)>,

    // (elem, struct name)
    struct_type_table: Vec<(ElemId, SymbolId)>,

    // (elem, field name, value elem)
    struct_table: Vec<(ElemId, SymbolId, ElemId)>,

    // (elem, index, value)
    seq_table: Vec<(ElemId, usize, ElemId)>,

    // (elem, enum name, variant name)
    variant_type_table: Vec<(ElemId, SymbolId, SymbolId)>,

    // (elem, index, value)
    tuple_table: Vec<(ElemId, usize, ElemId)>,
}

impl Default for Backend {
    fn default() -> Self {
        Self {
            cur_symbol_id: SymbolId(1),
            symbol_table: Default::default(),
            type_table: Default::default(),
            number_table: Default::default(),
            string_table: Default::default(),
            map_table: Default::default(),
            struct_type_table: Default::default(),
            struct_table: Default::default(),
            seq_table: Default::default(),
            variant_type_table: Default::default(),
            tuple_table: Default::default()
        }
    }
}

impl Backend {
    fn intern_string(&mut self, s: &str) -> SymbolId {
        match self.symbol_table.get(s) {
            Some(id) => *id,
            None => {
                let SymbolId(id) = self.cur_symbol_id;
                self.symbol_table.insert(s.to_string(), self.cur_symbol_id);
                self.cur_symbol_id.0 += 1;
                SymbolId(id)
            }
        }
    }

    /// Print generate fact tables to standard output.
    pub fn dump(&self) {
        if !self.symbol_table.is_empty() {
            println!("{:^33}", "Symbol Table");
            println!("---------------------------------");
            println!("{:<15} | {:<15}", "String", "Symbol Id");
            println!("---------------------------------");
            for (str, sym) in self.symbol_table.iter() {
                println!("{:<15} | {:<15}", str, sym.0);
            }
            println!();
        }

        if !self.type_table.is_empty() {
            println!("{:^33}", "elem Table");
            println!("---------------------------------");
            println!("{:<15} | {:<15}", "elem Id", "elem Type");
            println!("---------------------------------");
            for (elem, elem_type) in self.type_table.iter() {
                println!("{:<15} | {:<15?}", elem.0, elem_type.0);
            }
            println!();
        }

        if !self.number_table.is_empty() {
            println!("{:^33}", "Number Table");
            println!("---------------------------------");
            println!("{:<15} | {:<15}", "elem Id", "Number");
            println!("---------------------------------");
            for (elem, number) in self.number_table.iter() {
                println!("{:<15} | {:<15?}", elem.0, number);
            }
            println!();
        }

        if !self.string_table.is_empty() {
            println!("{:^33}", "String Table");
            println!("---------------------------------");
            println!("{:<15} | {:<15}", "elem Id", "String");
            println!("---------------------------------");
            for (elem, str) in self.string_table.iter() {
                println!("{:<15} | {:<15?}", elem.0, str.0);
            }
            println!();
        }

        if !self.map_table.is_empty() {
            println!("{:^51}", "Map Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "elem Id", "Key", "Value");
            println!("---------------------------------------------------");
            for (elem, key, val) in self.map_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", elem.0, key.0, val.0);
            }
            println!();
        }

        if !self.struct_type_table.is_empty() {
            println!("{:^33}", "Struct Type Table");
            println!("---------------------------------");
            println!("{:<15} | {:<15}", "elem Id", "Struct Type");
            println!("---------------------------------");
            for (elem, struct_type) in self.string_table.iter() {
                println!("{:<15} | {:<15?}", elem.0, struct_type.0);
            }
            println!();
        }

        if !self.struct_table.is_empty() {
            println!("{:^51}", "Struct Field Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "elem Id", "Field", "Value");
            println!("---------------------------------------------------");
            for (elem, field, val) in self.struct_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", elem.0, field.0, val.0);
            }
            println!();
        }

        if !self.seq_table.is_empty() {
            println!("{:^51}", "Seq Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "elem Id", "Index", "Value");
            println!("---------------------------------------------------");
            for (elem, index, val) in self.seq_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", elem.0, index, val.0);
            }
            println!();
        }

        if !self.variant_type_table.is_empty() {
            println!("{:^51}", "Variant Type Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "elem Id", "Enum Type", "Variant Name");
            println!("---------------------------------------------------");
            for (elem, enum_type, variant_name) in self.variant_type_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", elem.0, enum_type.0, variant_name.0);
            }
            println!();
        }

        if !self.tuple_table.is_empty() {
            println!("{:^51}", "Tuple Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "elem Id", "Index", "Value");
            println!("---------------------------------------------------");
            for (elem, index, val) in self.tuple_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", elem.0, index, val.0);
            }
            println!();
        }
    }

    pub fn dump_to_db(&self, filename: &str) -> rusqlite::Result<()> {
        let conn = rusqlite::Connection::open(filename).unwrap();

        conn.execute_batch(
            "BEGIN;

            CREATE TABLE __SymbolTable (
                id INTEGER NOT NULL,
                symbol TEXT NOT NULL,
                PRIMARY KEY (id)
            );

            CREATE TABLE _type (
                id INTEGER NOT NULL,
                type INTEGER NOT NULL,
                PRIMARY KEY (id)
            );

            CREATE VIEW type AS
            SELECT _type.id AS id, __SymbolTable.symbol AS type
            FROM _type INNER JOIN __SymbolTable
            ON _type.type = __SymbolTable.id;

            CREATE TABLE _number (
                id INTEGER NOT NULL,
                value INTEGER NOT NULL,
                PRIMARY KEY (id),
                FOREIGN KEY(id) REFERENCES _type(id)
            );

            CREATE VIEW number AS
            SELECT id, value FROM _number;

            CREATE TABLE _string (
                id INTEGER NOT NULL,
                value INTEGER NOT NULL,
                PRIMARY KEY (id),
                FOREIGN KEY(id) REFERENCES _type(id),
                FOREIGN KEY(value) REFERENCES __SymbolTable(id)
            );

            CREATE VIEW string AS
            SELECT _string.id AS id, __SymbolTable.symbol AS value
            FROM _string INNER JOIN __SymbolTable
            ON _string.value = __SymbolTable.id;

            CREATE TABLE _map (
                id INTEGER NOT NULL,
                key INTEGER NOT NULL,
                value INTEGER NOT NULL,
                PRIMARY KEY (id, key),
                FOREIGN KEY(id) REFERENCES _type(id),
                FOREIGN KEY(key) REFERENCES _type(id),
                FOREIGN KEY(value) REFERENCES _type(id)
            );

            CREATE VIEW map AS
            SELECT id, key, value FROM _map;

            CREATE TABLE _struct (
                id INTEGER NOT NULL,
                field INTEGER NOT NULL,
                value INTEGER NOT NULL,
                PRIMARY KEY (id, field),
                FOREIGN KEY(id) REFERENCES _type(id),
                FOREIGN KEY(field) REFERENCES __SymbolTable(id),
                FOREIGN KEY(value) REFERENCES _type(id)
            );

            CREATE VIEW struct AS
            SELECT _struct.id AS id, __SymbolTable.symbol AS field, _struct.value AS value
            FROM _struct INNER JOIN __SymbolTable
            ON _struct.field = __SymbolTable.id;

            CREATE TABLE _seq (
                id INTEGER NOT NULL,
                pos INTEGER NOT NULL,
                value INTEGER NOT NULL,
                PRIMARY KEY (id, pos),
                FOREIGN KEY(id) REFERENCES _type(id),
                FOREIGN KEY(value) REFERENCES _type(id)
            );

            CREATE VIEW seq AS
            SELECT id, pos, value FROM _seq;

            CREATE TABLE _tuple (
                id INTEGER NOT NULL,
                pos INTEGER NOT NULL,
                value INTEGER NOT NULL,
                PRIMARY KEY (id, pos),
                FOREIGN KEY(id) REFERENCES _type(id),
                FOREIGN KEY(value) REFERENCES _type(id)
            );

            CREATE VIEW tuple AS
            SELECT id, pos, value FROM _tuple;

            CREATE TABLE _structType (
                id INTEGER NOT NULL,
                type INTEGER NOT NULL,
                PRIMARY KEY (id),
                FOREIGN KEY(id) REFERENCES _type(id),
                FOREIGN KEY(type) REFERENCES __SymbolTable(id)
            );

            CREATE VIEW structType AS
            SELECT _structType.id AS id, __SymbolTable.symbol AS type
            FROM _structType INNER JOIN __SymbolTable
            ON _structType.type = __SymbolTable.id;

            CREATE TABLE _variantType (
                id INTEGER NOT NULL,
                type INTEGER NOT NULL,
                variant INTEGER NOT NULL,
                PRIMARY KEY (id),
                FOREIGN KEY(id) REFERENCES _type(id),
                FOREIGN KEY(type) REFERENCES __SymbolTable(id),
                FOREIGN KEY(variant) REFERENCES __SymbolTable(id)
            );

            CREATE VIEW variantType AS
            SELECT _variantType.id AS id, s1.symbol AS type, s2.symbol AS variant
            FROM _variantType
                INNER JOIN __SymbolTable AS s1 ON _variantType.type = s1.id
                INNER JOIN __SymbolTable AS s2 ON _variantType.variant = s2.id;

            COMMIT;"
        ).unwrap();

        let mut insert_symbol_table =
            conn.prepare(
                "INSERT INTO __SymbolTable (id, symbol) VALUES (?1, ?2);",
            )?;

        for (str, id) in self.symbol_table.iter() {
            insert_symbol_table.execute((id.0, str))?;
        }

        let mut insert_type_table =
            conn.prepare(
                "INSERT INTO _type (id, type) VALUES (?1, ?2);",
            )?;

        for (id, elem_type) in self.type_table.iter() {
            insert_type_table.execute((id.0, elem_type.0))?;
        }

        let mut insert_number_table =
            conn.prepare(
                "INSERT INTO _number (id, value) VALUES (?1, ?2);",
            )?;

        for (id, value) in self.number_table.iter() {
            insert_number_table.execute((id.0, value))?;
        }

        let mut insert_string_table =
            conn.prepare(
                "INSERT INTO _string (id, value) VALUES (?1, ?2);",
            )?;

        for (id, value) in self.string_table.iter() {
            insert_string_table.execute((id.0, value.0))?;
        }

        let mut insert_map_table =
            conn.prepare(
                "INSERT INTO _map (id, key, value) VALUES (?1, ?2, ?3);",
            )?;

        for (id, key, value) in self.map_table.iter() {
            insert_map_table.execute((id.0, key.0, value.0))?;
        }

        let mut insert_struct_table =
            conn.prepare(
                "INSERT INTO _struct (id, field, value) VALUES (?1, ?2, ?3);",
            )?;

        for (id, field, value) in self.struct_table.iter() {
            insert_struct_table.execute((id.0, field.0, value.0))?;
        }

        let mut insert_seq_table =
            conn.prepare(
                "INSERT INTO _seq (id, pos, value) VALUES (?1, ?2, ?3);",
            )?;

        for (id, pos, value) in self.seq_table.iter() {
            insert_seq_table.execute((id.0, pos, value.0))?;
        }

        let mut insert_tuple_table =
            conn.prepare(
                "INSERT INTO _tuple (id, pos, value) VALUES (?1, ?2, ?3);",
            )?;

        for (id, pos, value) in self.tuple_table.iter() {
            insert_tuple_table.execute((id.0, pos, value.0))?;
        }

        let mut insert_struct_type_table =
            conn.prepare(
                "INSERT INTO _structType (id, type) VALUES (?1, ?2);"
            )?;

        for (id, type_sym) in self.struct_type_table.iter() {
            insert_struct_type_table.execute((id.0, type_sym.0))?;
        }

        let mut insert_variant_type_table =
            conn.prepare(
                "INSERT INTO _variantType (id, type, variant) VALUES (?1, ?2, ?3);"
            )?;

        for (id, type_sym, variant_sym) in self.variant_type_table.iter() {
            insert_variant_type_table.execute((id.0, type_sym.0, variant_sym.0))?;
        }

        rusqlite::Result::Ok(())
    }
}

impl<'a> DatalogExtractorBackend for &'a mut Backend {
    type Ok = ();

    fn add_elem(&mut self, elem: ElemId, elem_type: ElemType) -> Result<()> {
        let table_name: &str = match elem_type {
            ElemType::Bool |
            ElemType::I8 | ElemType::I16 | ElemType::I32 | ElemType::I64 |
            ElemType::U8 | ElemType::U16 | ElemType::U32 | ElemType::U64 => {
                "Number"
            },

            ElemType::Char | ElemType::Str => {
                "Str"
            }

            ElemType::F32 | ElemType::F64 | ElemType::Bytes => {
                return Result::Err(DatalogExtractionError::UnextractableData);
            }

            ElemType::Map => "Map",
            ElemType::Seq => "Seq",
            ElemType::Struct => "Struct",
            ElemType::StructVariant => "StructVariant",
            ElemType::Tuple => "Tuple",
            ElemType::TupleStruct => "TupleStruct",
            ElemType::TupleVariant => "TupleVariant",
            ElemType::Unit => "Unit",
            ElemType::UnitStruct => "UnitStruct",
            ElemType::UnitVariant => "UnitVariant",
            ElemType::NewtypeStruct => "NewtypeStruct",
            ElemType::NewtypeVariant => "NewtypeVariant"
        };

        let elem_type_sym = self.intern_string(table_name);
        self.type_table.push((elem, elem_type_sym));
        Result::Ok(())
    }

    fn add_bool(&mut self, elem: ElemId, value: bool) -> Result<Self::Ok> {
        self.number_table.push((elem, if value { 1 } else { 0 }));
        Result::Ok(())
    }

    fn add_i8(&mut self, elem: ElemId, value: i8) -> Result<Self::Ok> {
        self.number_table.push((elem, value as isize));
        Result::Ok(())
    }

    fn add_i16(&mut self, elem: ElemId, value: i16) -> Result<Self::Ok> {
        self.number_table.push((elem, value as isize));
        Result::Ok(())
    }

    fn add_i32(&mut self, elem: ElemId, value: i32) -> Result<Self::Ok> {
        self.number_table.push((elem, value as isize));
        Result::Ok(())
    }

    fn add_i64(&mut self, elem: ElemId, value: i64) -> Result<Self::Ok> {
        self.number_table.push((elem, value as isize));
        Result::Ok(())
    }

    fn add_u8(&mut self, elem: ElemId, value: u8) -> Result<Self::Ok> {
        self.number_table.push((elem, value as isize));
        Result::Ok(())
    }

    fn add_u16(&mut self, elem: ElemId, value: u16) -> Result<Self::Ok> {
        self.number_table.push((elem, value as isize));
        Result::Ok(())
    }

    fn add_u32(&mut self, elem: ElemId, value: u32) -> Result<Self::Ok> {
        self.number_table.push((elem, value as isize));
        Result::Ok(())
    }

    fn add_u64(&mut self, elem: ElemId, value: u64) -> Result<Self::Ok> {
        self.number_table.push((elem, value as isize));
        Result::Ok(())
    }

    fn add_f32(&mut self, _elem: ElemId, _value: f32) -> Result<Self::Ok> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    fn add_f64(&mut self, _elem: ElemId, _value: f64) -> Result<Self::Ok> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    fn add_bytes(&mut self, _elem: ElemId, _value: &[u8]) -> Result<Self::Ok> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    fn add_char(&mut self, elem: ElemId, value: char) -> Result<Self::Ok> {
        self.add_str(elem, &value.to_string())
    }

    fn add_str(&mut self, elem: ElemId, value: &str) -> Result<()> {
        let value_sym = self.intern_string(value);
        self.string_table.push((elem, value_sym));
        Result::Ok(())
    }

    fn add_map_entry(&mut self, elem: ElemId, key: ElemId, value: ElemId) -> Result<()> {
        self.map_table.push((elem, key, value));
        Result::Ok(())
    }

    fn add_struct_type(&mut self, elem: ElemId, struct_name: &str) -> Result<()> {
        let struct_name_sym = self.intern_string(struct_name);
        self.struct_type_table.push((elem, struct_name_sym));
        Result::Ok(())
    }

    fn add_struct_entry(&mut self, elem: ElemId, key: &str, value: ElemId) -> Result<()> {
        let key_sym = self.intern_string(key);
        self.struct_table.push((elem, key_sym, value));
        Result::Ok(())
    }

    fn add_seq_entry(&mut self, elem: ElemId, pos: usize, value: ElemId) -> Result<()> {
        self.seq_table.push((elem, pos, value));
        Result::Ok(())
    }

    fn add_variant_type(&mut self, elem: ElemId, type_name: &str, variant_name: &str) -> Result<()> {
        let type_name_sym = self.intern_string(type_name);
        let variant_name_sym = self.intern_string(variant_name);
        self.variant_type_table.push((elem, type_name_sym, variant_name_sym));
        Result::Ok(())
    }

    fn add_tuple_entry(&mut self, elem: ElemId, pos: usize, value: ElemId) -> Result<()> {
        self.tuple_table.push((elem, pos, value));
        Result::Ok(())
    }
}
