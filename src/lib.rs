use serde::ser;
use std::{
    collections::HashMap,
    result,
    fmt::{self, Display}
};

#[derive(Debug)]
pub enum DatalogExtractionError {
    UnextractableData,
    Custom(String)
}

impl Display for DatalogExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatalogExtractionError::UnextractableData => {
                write!(f, "unextractable data")
            },

            DatalogExtractionError::Custom(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

impl ser::Error for DatalogExtractionError {
    fn custom<T: fmt::Display>(msg:T) -> Self {
        Self::Custom(msg.to_string())
    }
}

impl std::error::Error for DatalogExtractionError { }

type Result<T> = std::result::Result<T, DatalogExtractionError>;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct NodeId(usize);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct SymbolId(usize);

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum NodeType {
    Map,
    Number,
    Seq,
    String,
    Struct,
    StructVariant,
    Tuple,
    TupleStruct,
    TupleVariant,
    Unit,
    UnitStruct,
    UnitVariant,
}

impl NodeType {
    fn name(&self) -> String {
        match self {
            NodeType::Map => "Map",
            NodeType::Number => "Number",
            NodeType::Seq => "Seq",
            NodeType::String => "String",
            NodeType::Struct => "Struct",
            NodeType::StructVariant => "StructVariant",
            NodeType::Tuple => "Tuple",
            NodeType::TupleStruct => "TupleStruct",
            NodeType::TupleVariant => "TupleVariant",
            NodeType::Unit => "Unit",
            NodeType::UnitStruct => "UnitStruct",
            NodeType::UnitVariant => "UnitVariant",
        }.to_string()
    }

    fn all() -> Vec<NodeType> {
        vec![
            NodeType::Map,
            NodeType::Number,
            NodeType::Seq,
            NodeType::String,
            NodeType::Struct,
            NodeType::StructVariant,
            NodeType::Tuple,
            NodeType::TupleStruct,
            NodeType::TupleVariant,
            NodeType::Unit,
            NodeType::UnitStruct,
            NodeType::UnitVariant
        ]
    }
}

pub trait DatalogExtractorBackend {
    type Ok;
    fn add_node(&mut self, node: NodeId, node_type: NodeType) -> Result<Self::Ok>;
    fn add_number(&mut self, node: NodeId, value: isize) -> Result<Self::Ok>;
    fn add_string(&mut self, node: NodeId, value: &str) -> Result<Self::Ok>;
    fn add_map(&mut self, node: NodeId, key: NodeId, value: NodeId) -> Result<Self::Ok>;
    fn add_struct_type(&mut self, node: NodeId, struct_name: &str) -> Result<Self::Ok>;
    fn add_struct(&mut self, node: NodeId, key: &str, value: NodeId) -> Result<Self::Ok>;
    fn add_seq(&mut self, node: NodeId, pos: usize, value: NodeId) -> Result<Self::Ok>;
    fn add_variant_type(&mut self, node: NodeId, type_name: &str, variant_name: &str) -> Result<Self::Ok>;
    fn add_tuple(&mut self, node: NodeId, pos: usize, value: NodeId) -> Result<Self::Ok>;
}

pub struct SouffleSQLiteBackend {
    cur_symbol_id: SymbolId,
    symbol_table: HashMap<String, SymbolId>,

    // (node, node type)
    type_table: Vec<(NodeId, SymbolId)>,

    // (node, value)
    number_table: Vec<(NodeId, isize)>,

    // (node, symbol)
    string_table: Vec<(NodeId, SymbolId)>,

    // (node, key, value)
    map_table: Vec<(NodeId, NodeId, NodeId)>,

    // (node, struct name)
    struct_type_table: Vec<(NodeId, SymbolId)>,

    // (node, field name, value node)
    struct_table: Vec<(NodeId, SymbolId, NodeId)>,

    // (node, index, value)
    seq_table: Vec<(NodeId, usize, NodeId)>,

    // (node, enum name, variant name)
    variant_type_table: Vec<(NodeId, SymbolId, SymbolId)>,

    // (node, index, value)
    tuple_table: Vec<(NodeId, usize, NodeId)>,
}

impl Default for SouffleSQLiteBackend {
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

impl SouffleSQLiteBackend {
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
            println!("{:^33}", "Node Table");
            println!("---------------------------------");
            println!("{:<15} | {:<15}", "Node Id", "Node Type");
            println!("---------------------------------");
            for (node, node_type) in self.type_table.iter() {
                println!("{:<15} | {:<15?}", node.0, node_type.0);
            }
            println!();
        }

        if !self.number_table.is_empty() {
            println!("{:^33}", "Number Table");
            println!("---------------------------------");
            println!("{:<15} | {:<15}", "Node Id", "Number");
            println!("---------------------------------");
            for (node, number) in self.number_table.iter() {
                println!("{:<15} | {:<15?}", node.0, number);
            }
            println!();
        }

        if !self.string_table.is_empty() {
            println!("{:^33}", "String Table");
            println!("---------------------------------");
            println!("{:<15} | {:<15}", "Node Id", "String");
            println!("---------------------------------");
            for (node, str) in self.string_table.iter() {
                println!("{:<15} | {:<15?}", node.0, str.0);
            }
            println!();
        }

        if !self.map_table.is_empty() {
            println!("{:^51}", "Map Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "Node Id", "Key", "Value");
            println!("---------------------------------------------------");
            for (node, key, val) in self.map_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", node.0, key.0, val.0);
            }
            println!();
        }

        if !self.struct_type_table.is_empty() {
            println!("{:^33}", "Struct Type Table");
            println!("---------------------------------");
            println!("{:<15} | {:<15}", "Node Id", "Struct Type");
            println!("---------------------------------");
            for (node, struct_type) in self.string_table.iter() {
                println!("{:<15} | {:<15?}", node.0, struct_type.0);
            }
            println!();
        }

        if !self.struct_table.is_empty() {
            println!("{:^51}", "Struct Field Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "Node Id", "Field", "Value");
            println!("---------------------------------------------------");
            for (node, field, val) in self.struct_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", node.0, field.0, val.0);
            }
            println!();
        }

        if !self.seq_table.is_empty() {
            println!("{:^51}", "Seq Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "Node Id", "Index", "Value");
            println!("---------------------------------------------------");
            for (node, index, val) in self.seq_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", node.0, index, val.0);
            }
            println!();
        }

        if !self.variant_type_table.is_empty() {
            println!("{:^51}", "Variant Type Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "Node Id", "Enum Type", "Variant Name");
            println!("---------------------------------------------------");
            for (node, enum_type, variant_name) in self.variant_type_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", node.0, enum_type.0, variant_name.0);
            }
            println!();
        }

        if !self.tuple_table.is_empty() {
            println!("{:^51}", "Tuple Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "Node Id", "Index", "Value");
            println!("---------------------------------------------------");
            for (node, index, val) in self.tuple_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", node.0, index, val.0);
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

        for (id, node_type) in self.type_table.iter() {
            insert_type_table.execute((id.0, node_type.0))?;
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

impl<'a> DatalogExtractorBackend for &'a mut SouffleSQLiteBackend {
    type Ok = ();

    fn add_node(&mut self, node: NodeId, node_type: NodeType) -> Result<()> {
        let node_type_sym = self.intern_string(&node_type.name());
        self.type_table.push((node, node_type_sym));
        Result::Ok(())
    }

    fn add_number(&mut self, node: NodeId, value: isize) -> Result<()> {
        self.number_table.push((node, value));
        Result::Ok(())
    }

    fn add_string(&mut self, node: NodeId, value: &str) -> Result<()> {
        let value_sym = self.intern_string(value);
        self.string_table.push((node, value_sym));
        Result::Ok(())
    }

    fn add_map(&mut self, node: NodeId, key: NodeId, value: NodeId) -> Result<()> {
        self.map_table.push((node, key, value));
        Result::Ok(())
    }

    fn add_struct_type(&mut self, node: NodeId, struct_name: &str) -> Result<()> {
        let struct_name_sym = self.intern_string(struct_name);
        self.struct_type_table.push((node, struct_name_sym));
        Result::Ok(())
    }

    fn add_struct(&mut self, node: NodeId, key: &str, value: NodeId) -> Result<()> {
        let key_sym = self.intern_string(key);
        self.struct_table.push((node, key_sym, value));
        Result::Ok(())
    }

    fn add_seq(&mut self, node: NodeId, pos: usize, value: NodeId) -> Result<()> {
        self.seq_table.push((node, pos, value));
        Result::Ok(())
    }

    fn add_variant_type(&mut self, node: NodeId, type_name: &str, variant_name: &str) -> Result<()> {
        let type_name_sym = self.intern_string(type_name);
        let variant_name_sym = self.intern_string(variant_name);
        self.variant_type_table.push((node, type_name_sym, variant_name_sym));
        Result::Ok(())
    }

    fn add_tuple(&mut self, node: NodeId, pos: usize, value: NodeId) -> Result<()> {
        self.tuple_table.push((node, pos, value));
        Result::Ok(())
    }
}

pub struct DatalogExtractor<'a> {
    cur_node_id: NodeId,
    node_stack: Vec<NodeId>,
    parent_stack: Vec<(NodeId, usize)>,
    backend: Box<dyn DatalogExtractorBackend<Ok = ()> + 'a>,
}

impl<'a> DatalogExtractor<'a> {
    pub fn new<T: 'a + DatalogExtractorBackend<Ok = ()>>(backend: T) -> Self {
        DatalogExtractor {
            backend: Box::new(backend),
            cur_node_id: NodeId(1),
            node_stack: Vec::new(),
            parent_stack: Vec::new(),
        }
    }

    fn get_fresh_node_id(&mut self, node_type: NodeType) -> Result<NodeId> {
        let id = self.cur_node_id;
        self.backend.add_node(id, node_type)?;
        self.node_stack.push(id);
        self.cur_node_id.0 += 1;
        Result::Ok(id)
    }

    fn serialize_tuple_or_seq_element<T: ?Sized + serde::Serialize>(&mut self, value: &T, node_type: NodeType) -> Result<()> {
        value.serialize(&mut *self)?;
        let child_id = self.node_stack.pop().unwrap();
        let (parent_id , pos) = self.parent_stack.last_mut().unwrap();
        *pos += 1;

        match node_type {
            NodeType::Seq => {
                self.backend.add_seq(*parent_id, *pos, child_id)
            }

            NodeType::Tuple | NodeType::TupleStruct | NodeType::TupleVariant => {
                self.backend.add_tuple(*parent_id, *pos, child_id)
            }

            _ => unreachable!()
        }
    }

    fn end_parent(&mut self) -> Result<()> {
        self.parent_stack.pop();
        Result::Ok(())
    }

    fn serialize_struct_element<T: ?Sized + serde::Serialize>(
        &mut self,
        key: &'static str,
        value: &T
    ) -> Result<()> {
        value.serialize(&mut *self)?;
        let (parent_id, _) = self.parent_stack.last_mut().unwrap();
        let val_id = self.node_stack.pop().unwrap();
        self.backend.add_struct(*parent_id, key, val_id)
    }
}

impl<'a, 'b> ser::Serializer for &'a mut DatalogExtractor<'b> {
    type Ok = ();
    type Error = DatalogExtractionError;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number)?;
        self.backend.add_number(id, if v { 1 } else { 0 })
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number)?;
        self.backend.add_number(id, v as isize)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number)?;
        self.backend.add_number(id, v as isize)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number)?;
        self.backend.add_number(id, v as isize)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number)?;
        self.backend.add_number(id, v as isize)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number)?;
        self.backend.add_number(id, v as isize)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number)?;
        self.backend.add_number(id, v as isize)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number)?;
        self.backend.add_number(id, v as isize)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number)?;
        self.backend.add_number(id, v as isize)
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::String)?;
        self.backend.add_string(id, v)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit_variant("Option", 0, "None")
    }

    fn serialize_some<T: ?Sized + serde::Serialize>(self, value: &T) -> Result<Self::Ok> {
        self.serialize_newtype_variant("Option", 1, "Some", value)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.get_fresh_node_id(NodeType::Unit);
        Result::Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::UnitStruct)?;
        self.backend.add_struct_type(id, name)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        let id = self.get_fresh_node_id(NodeType::UnitVariant)?;
        self.node_stack.push(id);
        self.backend.add_variant_type(id, name, variant)
    }

    fn serialize_newtype_struct<T: ?Sized + serde::Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        value.serialize(&mut *self)?;
        let child_id = self.node_stack.pop().unwrap();
        let id = self.get_fresh_node_id(NodeType::TupleStruct)?;
        self.backend.add_struct_type(id, name)?;
        self.backend.add_tuple(id, 0, child_id)
    }

    fn serialize_newtype_variant<T: ?Sized + serde::Serialize>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        value.serialize(&mut *self)?;
        let child_id = self.node_stack.pop().unwrap();

        let id = self.get_fresh_node_id(NodeType::TupleVariant)?;
        self.backend.add_variant_type(id, name, variant)?;
        self.backend.add_tuple(id, 0, child_id)
    }

    fn serialize_seq(self, _len_opt: Option<usize>) -> Result<Self::SerializeSeq> {
        let id = self.get_fresh_node_id(NodeType::Seq)?;
        self.parent_stack.push((id, 0));
        Result::Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        let id = self.get_fresh_node_id(NodeType::Tuple)?;
        self.parent_stack.push((id, 0));
        Result::Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        let id = self.get_fresh_node_id(NodeType::TupleStruct)?;
        self.parent_stack.push((id, 0));
        self.backend.add_struct_type(id, name)?;
        Result::Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        let id = self.get_fresh_node_id(NodeType::TupleVariant)?;
        self.parent_stack.push((id, 0));
        self.backend.add_variant_type(id, name, variant)?;
        Result::Ok(self)
    }

    fn serialize_map(self, _len_opt: Option<usize>) -> Result<Self::SerializeMap> {
        let id = self.get_fresh_node_id(NodeType::Map)?;
        self.parent_stack.push((id, 0));
        Result::Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        let id = self.get_fresh_node_id(NodeType::Struct)?;
        self.parent_stack.push((id, 0));
        self.backend.add_struct_type(id, name)?;
        Result::Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        let id = self.get_fresh_node_id(NodeType::StructVariant)?;
        self.parent_stack.push((id, 0));
        self.backend.add_variant_type(id, name, variant)?;
        Result::Ok(self)
    }
}

impl<'a, 'b> ser::SerializeSeq for &'a mut DatalogExtractor<'b> {
    type Ok = ();
    type Error = DatalogExtractionError;

    fn serialize_element<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        self.serialize_tuple_or_seq_element(value, NodeType::Seq)
    }

    fn end(self) -> Result<()> {
        self.end_parent()
    }
}

impl<'a, 'b> ser::SerializeTuple for &'a mut DatalogExtractor<'b> {
    type Ok = ();
    type Error = DatalogExtractionError;

    fn serialize_element<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        self.serialize_tuple_or_seq_element(value, NodeType::Tuple)
    }

    fn end(self) -> Result<()> {
        self.end_parent()
    }
}

impl<'a, 'b> ser::SerializeTupleVariant for &'a mut DatalogExtractor<'b> {
    type Ok = ();
    type Error = DatalogExtractionError;

    fn serialize_field<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        self.serialize_tuple_or_seq_element(value, NodeType::TupleVariant)
    }

    fn end(self) -> Result<Self::Ok> {
        self.end_parent()
    }
}

impl<'a, 'b> ser::SerializeTupleStruct for &'a mut DatalogExtractor<'b> {
    type Ok = ();
    type Error = DatalogExtractionError;

    fn serialize_field<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        self.serialize_tuple_or_seq_element(value, NodeType::TupleStruct)
    }

    fn end(self) -> Result<Self::Ok> {
        self.end_parent()
    }
}

impl<'a, 'b> ser::SerializeMap for &'a mut DatalogExtractor<'b> {
    type Ok = ();
    type Error = DatalogExtractionError;

    fn serialize_key<T: ?Sized + serde::Serialize>(&mut self, key: &T) -> Result<Self::Ok> {
        key.serialize(&mut **self)?;
        Result::Ok(())
    }

    fn serialize_value<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        value.serialize(&mut **self)?;
        let (parent_id, _) = self.parent_stack.last().unwrap();
        let val_id = self.node_stack.pop().unwrap();
        let key_id = self.node_stack.pop().unwrap();
        self.backend.add_map(*parent_id, key_id, val_id)
    }

    fn end(self) -> result::Result<Self::Ok, Self::Error> {
        self.end_parent()
    }
}

impl<'a, 'b> ser::SerializeStruct for &'a mut DatalogExtractor<'b> {
    type Ok = ();
    type Error = DatalogExtractionError;

    fn serialize_field<T: ?Sized + serde::Serialize>(
        &mut self,
        key: &'static str,
        value: &T
    ) -> Result<Self::Ok> {
        self.serialize_struct_element(key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.end_parent()
    }
}

impl<'a, 'b> ser::SerializeStructVariant for &'a mut DatalogExtractor<'b> {
    type Ok = ();
    type Error = DatalogExtractionError;

    fn serialize_field<T: ?Sized + serde::Serialize>(
        &mut self,
        key: &'static str,
        value: &T
    ) -> Result<Self::Ok> {
        self.serialize_struct_element(key, value)
    }

    fn end(self) -> result::Result<Self::Ok, Self::Error> {
        self.end_parent()
    }
}
