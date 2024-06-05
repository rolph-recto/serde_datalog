use serde::ser;
use std::{
    collections::HashMap,
    result,
    fmt::{self, Display}
};

#[derive(Debug)]
pub enum DatalogExtractionError {
    UnknownLength,
    UnextractableData,
    Custom(String)
}

impl Display for DatalogExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatalogExtractionError::UnknownLength => {
                write!(f, "unknown length")
            },

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

pub struct DatalogExtractor {
    cur_symbol_id: SymbolId,
    cur_node_id: NodeId,
    type_table: Vec<(NodeId, NodeType)>,
    node_stack: Vec<NodeId>,
    parent_stack: Vec<(NodeId, usize)>,
    symbol_table: HashMap<String, SymbolId>,

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

impl DatalogExtractor {
    fn intern_string(&mut self, s: &str) -> SymbolId {
        match self.symbol_table.get(s) {
            Some(id) => *id,
            None => {
                let SymbolId(id) = self.cur_symbol_id;
                self.symbol_table.insert(s.to_string(), SymbolId(id));
                self.cur_symbol_id.0 += 1;
                SymbolId(id)
            }
        }
    }

    fn get_fresh_node_id(&mut self, node_type: NodeType) -> NodeId {
        let id = self.cur_node_id;
        self.type_table.push((id, node_type));
        self.node_stack.push(id);
        self.cur_node_id.0 += 1;
        id
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
                println!("{:<15} | {:<15?}", node.0, node_type);
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
            insert_type_table.execute((id.0, self.symbol_table[&node_type.name()].0))?;
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

    fn serialize_tuple_or_seq_element<T: ?Sized + serde::Serialize>(&mut self, value: &T, node_type: NodeType) -> Result<()> {
        value.serialize(&mut *self)?;
        let table: &mut Vec<(NodeId, usize, NodeId)> =
            match node_type {
                NodeType::Seq => &mut self.seq_table,
                NodeType::Tuple | NodeType::TupleStruct | NodeType::TupleVariant => &mut self.tuple_table,
                _ => unreachable!()
            };

        let child_id = self.node_stack.pop().unwrap();
        let (parent_id , pos) = self.parent_stack.last_mut().unwrap();
        table.push((*parent_id, *pos, child_id));
        *pos += 1;
        Result::Ok(())
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
        let key_sym = self.intern_string(key);
        let (parent_id, _) = self.parent_stack.last_mut().unwrap();
        let val_id = self.node_stack.pop().unwrap();
        self.struct_table.push((*parent_id, key_sym, val_id));
        Result::Ok(())
    }
}

impl Default for DatalogExtractor {
    fn default() -> Self {
        let mut extractor =
            DatalogExtractor {
                cur_symbol_id: SymbolId(1),
                cur_node_id: NodeId(1),
                type_table: Vec::new(),
                node_stack: Vec::new(),
                parent_stack: Vec::new(),
                symbol_table: HashMap::new(),
                number_table: Vec::new(),
                string_table: Vec::new(),
                map_table: Vec::new(),
                struct_type_table: Vec::new(),
                variant_type_table: Vec::new(),
                struct_table: Vec::new(),
                seq_table: Vec::new(),
                tuple_table: Vec::new(),
            };

        for node_type in NodeType::all().iter() {
            extractor.intern_string(&node_type.name());
        }

        extractor
    }
}

impl<'a> ser::Serializer for &'a mut DatalogExtractor {
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
        let id = self.get_fresh_node_id(NodeType::Number);
        self.number_table.push((id, if v { 1 } else { 0 }));
        Result::Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number);
        self.number_table.push((id, v as isize));
        Result::Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number);
        self.number_table.push((id, v as isize));
        Result::Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number);
        self.number_table.push((id, v as isize));
        Result::Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number);
        self.number_table.push((id, v as isize));
        Result::Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number);
        self.number_table.push((id, v as isize));
        Result::Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number);
        self.number_table.push((id, v as isize));
        Result::Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number);
        self.number_table.push((id, v as isize));
        Result::Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Number);
        self.number_table.push((id, v as isize));
        Result::Ok(())
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
        let sym = self.intern_string(v);
        let id = self.get_fresh_node_id(NodeType::String);
        self.string_table.push((id, sym));
        Result::Ok(())
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
        let type_sym = self.intern_string(name);
        let id = self.get_fresh_node_id(NodeType::UnitStruct);
        self.struct_type_table.push((id, type_sym));
        Result::Ok(())
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        let id = self.get_fresh_node_id(NodeType::UnitVariant);
        self.node_stack.push(id);
        let type_sym = self.intern_string(name);
        let variant_sym = self.intern_string(variant);
        self.variant_type_table.push((id, type_sym, variant_sym));
        Result::Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized + serde::Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        value.serialize(&mut *self)?;
        let child_id = self.node_stack.pop().unwrap();

        let id = self.get_fresh_node_id(NodeType::TupleStruct);
        let type_sym = self.intern_string(name);
        self.struct_type_table.push((id, type_sym));
        self.tuple_table.push((id, 0, child_id));
        Result::Ok(())
    }

    fn serialize_newtype_variant<T: ?Sized + serde::Serialize>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        value.serialize(&mut *self)?;
        let child_id = self.node_stack.pop().unwrap();

        let id = self.get_fresh_node_id(NodeType::TupleVariant);
        let type_sym = self.intern_string(name);
        let variant_sym = self.intern_string(variant);
        self.variant_type_table.push((id, type_sym, variant_sym));
        self.tuple_table.push((id, 0, child_id));
        Result::Ok(())
    }

    fn serialize_seq(self, _len_opt: Option<usize>) -> Result<Self::SerializeSeq> {
        let id = self.get_fresh_node_id(NodeType::Seq);
        self.parent_stack.push((id, 0));
        Result::Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        let id = self.get_fresh_node_id(NodeType::Tuple);
        self.parent_stack.push((id, 0));
        Result::Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        let id = self.get_fresh_node_id(NodeType::TupleStruct);
        self.parent_stack.push((id, 0));
        let type_sym = self.intern_string(name);
        self.struct_type_table.push((id, type_sym));
        Result::Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        let id = self.get_fresh_node_id(NodeType::TupleVariant);
        self.parent_stack.push((id, 0));
        let type_sym = self.intern_string(name);
        let variant_sym = self.intern_string(variant);
        self.variant_type_table.push((id, type_sym, variant_sym));
        Result::Ok(self)
    }

    fn serialize_map(self, _len_opt: Option<usize>) -> Result<Self::SerializeMap> {
        let id = self.get_fresh_node_id(NodeType::Map);
        self.parent_stack.push((id, 0));
        Result::Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        let id = self.get_fresh_node_id(NodeType::Struct);
        self.parent_stack.push((id, 0));
        let type_sym = self.intern_string(name);
        self.struct_type_table.push((id, type_sym));
        Result::Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        let id = self.get_fresh_node_id(NodeType::StructVariant);
        self.parent_stack.push((id, 0));
        let type_sym = self.intern_string(name);
        let variant_sym = self.intern_string(variant);
        self.variant_type_table.push((id, type_sym, variant_sym));
        Result::Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut DatalogExtractor {
    type Ok = ();
    type Error = DatalogExtractionError;

    fn serialize_element<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        self.serialize_tuple_or_seq_element(value, NodeType::Seq)
    }

    fn end(self) -> Result<()> {
        self.end_parent()
    }
}

impl<'a> ser::SerializeTuple for &'a mut DatalogExtractor {
    type Ok = ();
    type Error = DatalogExtractionError;

    fn serialize_element<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        self.serialize_tuple_or_seq_element(value, NodeType::Tuple)
    }

    fn end(self) -> Result<()> {
        self.end_parent()
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut DatalogExtractor {
    type Ok = ();
    type Error = DatalogExtractionError;

    fn serialize_field<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        self.serialize_tuple_or_seq_element(value, NodeType::TupleVariant)
    }

    fn end(self) -> Result<Self::Ok> {
        self.end_parent()
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut DatalogExtractor {
    type Ok = ();
    type Error = DatalogExtractionError;

    fn serialize_field<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        self.serialize_tuple_or_seq_element(value, NodeType::TupleStruct)
    }

    fn end(self) -> Result<Self::Ok> {
        self.end_parent()
    }
}

impl<'a> ser::SerializeMap for &'a mut DatalogExtractor {
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
        self.map_table.push((*parent_id, key_id, val_id));
        Result::Ok(())
    }

    fn end(self) -> result::Result<Self::Ok, Self::Error> {
        self.end_parent()
    }
}

impl<'a> ser::SerializeStruct for &'a mut DatalogExtractor {
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

impl<'a> ser::SerializeStructVariant for &'a mut DatalogExtractor {
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
