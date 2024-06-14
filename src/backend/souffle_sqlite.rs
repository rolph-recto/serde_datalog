use crate::{
    ElemId, DatalogExtractorBackend, ElemType, Result,
    backend::vector
};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct SymbolId(usize);

/// DatalogExtractorBackend impl that stores facts in a SQLite database.
/// The database conforms to the input format for [Souffle](https://souffle-lang.github.io/),
/// a high-performance Datalog implementation.
pub struct Backend {
    vector_backend: vector::Backend,
}

impl Default for Backend {
    fn default() -> Self {
        Self {
            vector_backend: Default::default()
        }
    }
}

impl Backend {
    /// Print generate fact tables to standard output.
    pub fn dump(&self) {
        self.vector_backend.dump()
    }

    pub fn dump_to_db(&self, filename: &str) -> rusqlite::Result<()> {
        let conn = rusqlite::Connection::open(filename)?;
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
        )?;

        let mut insert_symbol_table =
            conn.prepare(
                "INSERT INTO __SymbolTable (id, symbol) VALUES (?1, ?2);",
            )?;

        for (sym, id) in self.vector_backend.symbol_table.iter() {
            insert_symbol_table.execute((id.0, sym))?;
        }

        let mut insert_type_table =
            conn.prepare(
                "INSERT INTO _type (id, type) VALUES (?1, ?2);",
            )?;

        for (id, sym) in self.vector_backend.type_table.iter() {
            insert_type_table.execute((id.0, sym.0))?;
        }

        let mut insert_number_table =
            conn.prepare(
                "INSERT INTO _number (id, value) VALUES (?1, ?2);",
            )?;

        for (id, value) in self.vector_backend.number_table.iter() {
            insert_number_table.execute((id.0, *value))?;
        }

        let mut insert_string_table =
            conn.prepare(
                "INSERT INTO _string (id, value) VALUES (?1, ?2);",
            )?;

        for (id, value) in self.vector_backend.string_table.iter() {
            insert_string_table.execute((id.0, value.0))?;
        }

        let mut insert_map_table =
            conn.prepare(
                "INSERT INTO _map (id, key, value) VALUES (?1, ?2, ?3);",
            )?;

        for (id, key, value) in self.vector_backend.map_table.iter() {
            insert_map_table.execute((id.0, key.0, value.0))?;
        }

        let mut insert_struct_table =
            conn.prepare(
                "INSERT INTO _struct (id, field, value) VALUES (?1, ?2, ?3);",
            )?;

        for (id, field, value) in self.vector_backend.struct_table.iter() {
            insert_struct_table.execute((id.0, field.0, value.0))?;
        }

        let mut insert_seq_table =
            conn.prepare(
                "INSERT INTO _seq (id, pos, value) VALUES (?1, ?2, ?3);",
            )?;

        for (id, pos, value) in self.vector_backend.seq_table.iter() {
            insert_seq_table.execute((id.0, pos, value.0))?;
        }

        let mut insert_tuple_table =
            conn.prepare(
                "INSERT INTO _tuple (id, pos, value) VALUES (?1, ?2, ?3);",
            )?;

        for (id, pos, value) in self.vector_backend.tuple_table.iter() {
            insert_tuple_table.execute((id.0, pos, value.0))?;
        }

        let mut insert_struct_type_table =
            conn.prepare(
                "INSERT INTO _structType (id, type) VALUES (?1, ?2);"
            )?;

        for (id, type_name) in self.vector_backend.struct_type_table.iter() {
            insert_struct_type_table.execute((id.0, type_name.0))?;
        }

        let mut insert_variant_type_table =
            conn.prepare(
                "INSERT INTO _variantType (id, type, variant) VALUES (?1, ?2, ?3);"
            )?;

        for (id, type_name, variant_name) in self.vector_backend.variant_type_table.iter() {
            insert_variant_type_table.execute((id.0, type_name.0, variant_name.0))?;
        }

        rusqlite::Result::Ok(())
    }
}

impl<'a> DatalogExtractorBackend for &'a mut Backend {
    fn add_elem(&mut self, elem: ElemId, elem_type: ElemType) -> Result<()> {
        (&mut self.vector_backend).add_elem(elem, elem_type)
    }

    fn add_bool(&mut self, elem: ElemId, value: bool) -> Result<()> {
        (&mut self.vector_backend).add_bool(elem, value)
    }

    fn add_i64(&mut self, elem: ElemId, value: i64) -> Result<()> {
        (&mut self.vector_backend).add_i64(elem, value)
    }

    fn add_u64(&mut self, elem: ElemId, value: u64) -> Result<()> {
        (&mut self.vector_backend).add_u64(elem, value)
    }

    fn add_str(&mut self, elem: ElemId, value: &str) -> Result<()> {
        (&mut self.vector_backend).add_str(elem, value)
    }

    fn add_map_entry(&mut self, elem: ElemId, key: ElemId, value: ElemId) -> Result<()> {
        (&mut self.vector_backend).add_map_entry(elem, key, value)
    }

    fn add_struct_type(&mut self, elem: ElemId, struct_name: &str) -> Result<()> {
        (&mut self.vector_backend).add_struct_type(elem, struct_name)
    }

    fn add_struct_entry(&mut self, elem: ElemId, key: &str, value: ElemId) -> Result<()> {
        (&mut self.vector_backend).add_struct_entry(elem, key, value)
    }

    fn add_seq_entry(&mut self, elem: ElemId, pos: usize, value: ElemId) -> Result<()> {
        (&mut self.vector_backend).add_seq_entry(elem, pos, value)
    }

    fn add_variant_type(&mut self, elem: ElemId, type_name: &str, variant_name: &str) -> Result<()> {
        (&mut self.vector_backend).add_variant_type(elem, type_name, variant_name)
    }

    fn add_tuple_entry(&mut self, elem: ElemId, pos: usize, value: ElemId) -> Result<()> {
        (&mut self.vector_backend).add_tuple_entry(elem, pos, value)
    }
}
