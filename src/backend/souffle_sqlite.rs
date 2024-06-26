//! A backend that stores facts in a [SQLite](https://sqlite.org) database,
//! in the format expected by [Souffle](https://souffle-lang.github.io/).

use delegate::delegate;
use std::{fmt::Display, hash::Hash};

use crate::{
    backend::vector::{self, BackendData},
    DatalogExtractorBackend, ElemId, ElemType, Result,
};

pub trait AbstractBackend: DatalogExtractorBackend {
    /// Print generated table facts to stdout.
    fn dump(self);

    /// Store facts in a SQLite file with name `filename`.
    fn dump_to_db(self, filename: &str) -> rusqlite::Result<()>;
}

struct BackendUtil;

impl BackendUtil {
    fn dump_to_db<K: Display + Eq + Hash>(
        data: &BackendData<K>,
        filename: &str,
    ) -> rusqlite::Result<rusqlite::Connection> {
        let conn = rusqlite::Connection::open(filename)?;
        {
            conn.execute_batch(
                "BEGIN;

                CREATE TABLE __SymbolTable (
                    id INTEGER NOT NULL,
                    symbol TEXT NOT NULL,
                    PRIMARY KEY (id)
                );

                CREATE TABLE _rootElem (
                    file INTEGER NOT NULL,
                    elem INTEGER NOT NULL,
                    PRIMARY KEY (file)
                );

                CREATE VIEW rootElem AS
                SELECT __SymbolTable.symbol AS file, _rootElem.elem as elem
                FROM _rootElem INNER JOIN __SymbolTable
                ON _rootElem.file = __SymbolTable.id;

                CREATE TABLE _type (
                    id INTEGER NOT NULL,
                    type INTEGER NOT NULL,
                    PRIMARY KEY (id)
                );

                CREATE VIEW type AS
                SELECT _type.id AS id, __SymbolTable.symbol AS type
                FROM _type INNER JOIN __SymbolTable
                ON _type.type = __SymbolTable.id;

                CREATE TABLE _bool (
                    id INTEGER NOT NULL,
                    value INTEGER NOT NULL,
                    PRIMARY KEY (id),
                    FOREIGN KEY(id) REFERENCES _type(id)
                );

                CREATE VIEW bool AS
                SELECT id, value FROM _bool;

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

                COMMIT;",
            )?;

            let mut insert_symbol_table =
                conn.prepare("INSERT INTO __SymbolTable (id, symbol) VALUES (?1, ?2);")?;

            for (sym, id) in data.symbol_table.iter() {
                insert_symbol_table.execute((id.0, sym))?;
            }

            let mut insert_root_elem_table =
                conn.prepare("INSERT INTO _rootElem (file, elem) VALUES (?1, ?2);")?;

            for (file, elem) in data.root_elem_table.iter() {
                insert_root_elem_table.execute((file.0, elem.0))?;
            }

            let mut insert_type_table =
                conn.prepare("INSERT INTO _type (id, type) VALUES (?1, ?2);")?;

            for (id, sym) in data.type_table.iter() {
                insert_type_table.execute((id.0, sym.0))?;
            }

            let mut insert_bool_table =
                conn.prepare("INSERT INTO _bool (id, value) VALUES (?1, ?2);")?;

            for (id, value) in data.bool_table.iter() {
                insert_bool_table.execute((id.0, if *value { 1 } else { 0 }))?;
            }

            let mut insert_number_table =
                conn.prepare("INSERT INTO _number (id, value) VALUES (?1, ?2);")?;

            for (id, value) in data.number_table.iter() {
                insert_number_table.execute((id.0, *value))?;
            }

            let mut insert_string_table =
                conn.prepare("INSERT INTO _string (id, value) VALUES (?1, ?2);")?;

            for (id, value) in data.string_table.iter() {
                insert_string_table.execute((id.0, value.0))?;
            }

            let mut insert_struct_table =
                conn.prepare("INSERT INTO _struct (id, field, value) VALUES (?1, ?2, ?3);")?;

            for ((id, field), value) in data.struct_table.iter() {
                insert_struct_table.execute((id.0, field.0, value.0))?;
            }

            let mut insert_seq_table =
                conn.prepare("INSERT INTO _seq (id, pos, value) VALUES (?1, ?2, ?3);")?;

            for ((id, pos), value) in data.seq_table.iter() {
                insert_seq_table.execute((id.0, pos, value.0))?;
            }

            let mut insert_tuple_table =
                conn.prepare("INSERT INTO _tuple (id, pos, value) VALUES (?1, ?2, ?3);")?;

            for ((id, pos), value) in data.tuple_table.iter() {
                insert_tuple_table.execute((id.0, pos, value.0))?;
            }

            let mut insert_struct_type_table =
                conn.prepare("INSERT INTO _structType (id, type) VALUES (?1, ?2);")?;

            for (id, type_name) in data.struct_type_table.iter() {
                insert_struct_type_table.execute((id.0, type_name.0))?;
            }

            let mut insert_variant_type_table =
                conn.prepare("INSERT INTO _variantType (id, type, variant) VALUES (?1, ?2, ?3);")?;

            for (id, (type_name, variant_name)) in data.variant_type_table.iter() {
                insert_variant_type_table.execute((id.0, type_name.0, variant_name.0))?;
            }
        }

        rusqlite::Result::Ok(conn)
    }
}

/// DatalogExtractorBackend impl that stores facts in a [SQLite](https://sqlite.org)
/// database.
/// The database conforms to the input format for [Souffle](https://souffle-lang.github.io/),
/// a high-performance Datalog implementation.
///
/// The backend stores facts in the following Souffle schema:
///
/// ```text
/// .type ElemId <: number
/// .type ElemType <: symbol
/// .type Field <: symbol
/// .type TypeName <: symbol
/// .type VariantName <: symbol
///
/// .decl type(id: ElemId, type: ElemType)
/// .decl number(id: ElemId, value: number)
/// .decl string(id: ElemId, value: symbol)
/// .decl map(id: ElemId, key: ElemId, value: ElemId)
/// .decl struct(id: ElemId, field: Field, value: ElemId)
/// .decl seq(id: ElemId, pos: number, value: ElemId)
/// .decl tuple(id: ElemId, pos: number, value: ElemId)
/// .decl structType(id: ElemId, type: TypeName)
/// .decl variantType(id: ElemId, type: TypeName, variant: VariantName)
/// ```
///
/// Note that this backend does **not** support extraction of
/// floating point values, and will return a
/// [UnextractableData][crate::DatalogExtractionError::UnextractableData] error if
/// the input contains such values.
#[derive(Default)]
pub struct Backend {
    vector_backend: vector::Backend,
}

impl AbstractBackend for Backend {
    /// Print generate fact tables to standard output.
    fn dump(self) {
        self.vector_backend.dump()
    }

    /// Store facts in a SQLite file with name `filename`.
    fn dump_to_db(self, filename: &str) -> rusqlite::Result<()> {
        let data = self.vector_backend.get_data();
        let conn = BackendUtil::dump_to_db(&data, filename)?;

        conn.execute_batch(
            "BEGIN;

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

            COMMIT;",
        )?;

        let mut insert_map_table =
            conn.prepare("INSERT INTO _map (id, key, value) VALUES (?1, ?2, ?3);")?;

        for ((id, key), value) in data.map_table.iter() {
            insert_map_table.execute((id.0, key.0, value.0))?;
        }

        rusqlite::Result::Ok(())
    }
}

impl DatalogExtractorBackend for Backend {
    delegate! {
        to (&mut self.vector_backend) {
            fn add_root_elem(&mut self, file: &str, elem: ElemId) -> Result<()>;
            fn add_elem(&mut self, elem: ElemId, elem_type: ElemType) -> Result<()>;
            fn add_bool(&mut self, elem: ElemId, value: bool) -> Result<()>;
            fn add_i64(&mut self, elem: ElemId, value: i64) -> Result<()>;
            fn add_u64(&mut self, elem: ElemId, value: u64) -> Result<()>;
            fn add_str(&mut self, elem: ElemId, value: &str) -> Result<()>;
            fn add_map_entry(&mut self, elem: ElemId, key: ElemId, value: ElemId) -> Result<()>;
            fn add_struct_type(&mut self, elem: ElemId, struct_name: &str) -> Result<()>;
            fn add_struct_entry(&mut self, elem: ElemId, key: &str, value: ElemId) -> Result<()>;
            fn add_seq_entry(&mut self, elem: ElemId, pos: usize, value: ElemId) -> Result<()>;
            fn add_variant_type(&mut self, elem: ElemId, type_name: &str, variant_name: &str) -> Result<()>;
            fn add_tuple_entry(&mut self, elem: ElemId, pos: usize, value: ElemId) -> Result<()>;
        }
    }
}

/// DatalogExtractorBackend impl similar to [Backend], except this impl assumes
/// map keys are always strings.
///
/// The backend stores facts in the following Souffle schema:
///
/// ```text
/// .decl type(id: ElemId, type: ElemType)
/// .decl number(id: ElemId, value: number)
/// .decl string(id: ElemId, value: symbol)
/// .decl map(id: ElemId, key: symbol, value: ElemId)
/// .decl struct(id: ElemId, field: Field, value: ElemId)
/// .decl seq(id: ElemId, pos: number, value: ElemId)
/// .decl tuple(id: ElemId, pos: number, value: ElemId)
/// .decl structType(id: ElemId, type: TypeName)
/// .decl variantType(id: ElemId, type: TypeName, variant: VariantName)
/// ```
#[derive(Default)]
pub struct StringKeyBackend {
    vector_backend: vector::StringKeyBackend,
}

impl AbstractBackend for StringKeyBackend {
    /// Print generate fact tables to standard output.
    fn dump(self) {
        self.vector_backend.dump()
    }

    /// Store facts in a SQLite file with name `filename`.
    fn dump_to_db(self, filename: &str) -> rusqlite::Result<()> {
        let data = self.vector_backend.get_data();
        let conn = BackendUtil::dump_to_db(&data, filename)?;

        conn.execute_batch(
            "BEGIN;

            CREATE TABLE _map (
                id INTEGER NOT NULL,
                key INTEGER NOT NULL,
                value INTEGER NOT NULL,
                PRIMARY KEY (id, key),
                FOREIGN KEY(id) REFERENCES _type(id),
                FOREIGN KEY(key) REFERENCES __SymbolTable(id),
                FOREIGN KEY(value) REFERENCES _type(id)
            );

            CREATE VIEW map AS
            SELECT _map.id AS id, __SymbolTable.symbol AS key, _map.value AS value
            FROM _map INNER JOIN __SymbolTable
            ON _map.key = __SymbolTable.id;

            COMMIT;",
        )?;

        let mut insert_map_table =
            conn.prepare("INSERT INTO _map (id, key, value) VALUES (?1, ?2, ?3);")?;

        for ((id, key), value) in data.map_table.iter() {
            insert_map_table.execute((id.0, key.0, value.0))?;
        }

        rusqlite::Result::Ok(())
    }
}

impl DatalogExtractorBackend for StringKeyBackend {
    delegate! {
        to (&mut self.vector_backend) {
            fn add_root_elem(&mut self, file: &str, elem: ElemId) -> Result<()>;
            fn add_elem(&mut self, elem: ElemId, elem_type: ElemType) -> Result<()>;
            fn add_bool(&mut self, elem: ElemId, value: bool) -> Result<()>;
            fn add_i64(&mut self, elem: ElemId, value: i64) -> Result<()>;
            fn add_u64(&mut self, elem: ElemId, value: u64) -> Result<()>;
            fn add_str(&mut self, elem: ElemId, value: &str) -> Result<()>;
            fn add_map_entry(&mut self, elem: ElemId, key: ElemId, value: ElemId) -> Result<()>;
            fn add_struct_type(&mut self, elem: ElemId, struct_name: &str) -> Result<()>;
            fn add_struct_entry(&mut self, elem: ElemId, key: &str, value: ElemId) -> Result<()>;
            fn add_seq_entry(&mut self, elem: ElemId, pos: usize, value: ElemId) -> Result<()>;
            fn add_variant_type(&mut self, elem: ElemId, type_name: &str, variant_name: &str) -> Result<()>;
            fn add_tuple_entry(&mut self, elem: ElemId, pos: usize, value: ElemId) -> Result<()>;
        }
    }
}
