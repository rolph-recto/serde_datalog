use serde::ser;
use std::{
    result,
    fmt::{self, Display}
};

pub mod backends;

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

pub type Result<T> = std::result::Result<T, DatalogExtractionError>;

/// A unique identifier for data.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct NodeId(usize);

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum NodeType {
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Char,
    Str,
    Bytes,
    Map,
    NewtypeStruct,
    NewtypeVariant,
    Seq,
    Struct,
    StructVariant,
    Tuple,
    TupleStruct,
    TupleVariant,
    Unit,
    UnitStruct,
    UnitVariant,
}

/// An implementation of `DatalogExtractorBackend` materializes facts generated
/// by [DatalogExtractor]. These facts can be represented in whatever format
/// the backend chooses, e.g. a SQLite database, a set of vectors, etc.
pub trait DatalogExtractorBackend {
    type Ok;
    fn add_node(&mut self, node: NodeId, node_type: NodeType) -> Result<Self::Ok>;
    fn add_bool(&mut self, node: NodeId, value: bool) -> Result<Self::Ok>;
    fn add_i8(&mut self, node: NodeId, value: i8) -> Result<Self::Ok>;
    fn add_i16(&mut self, node: NodeId, value: i16) -> Result<Self::Ok>;
    fn add_i32(&mut self, node: NodeId, value: i32) -> Result<Self::Ok>;
    fn add_i64(&mut self, node: NodeId, value: i64) -> Result<Self::Ok>;
    fn add_u8(&mut self, node: NodeId, value: u8) -> Result<Self::Ok>;
    fn add_u16(&mut self, node: NodeId, value: u16) -> Result<Self::Ok>;
    fn add_u32(&mut self, node: NodeId, value: u32) -> Result<Self::Ok>;
    fn add_u64(&mut self, node: NodeId, value: u64) -> Result<Self::Ok>;
    fn add_f32(&mut self, node: NodeId, value: f32) -> Result<Self::Ok>;
    fn add_f64(&mut self, node: NodeId, value: f64) -> Result<Self::Ok>;
    fn add_char(&mut self, node: NodeId, value: char) -> Result<Self::Ok>;
    fn add_str(&mut self, node: NodeId, value: &str) -> Result<Self::Ok>;
    fn add_bytes(&mut self, node: NodeId, value: &[u8]) -> Result<Self::Ok>;
    fn add_map(&mut self, node: NodeId, key: NodeId, value: NodeId) -> Result<Self::Ok>;
    fn add_struct_type(&mut self, node: NodeId, struct_name: &str) -> Result<Self::Ok>;
    fn add_struct(&mut self, node: NodeId, key: &str, value: NodeId) -> Result<Self::Ok>;
    fn add_seq(&mut self, node: NodeId, pos: usize, value: NodeId) -> Result<Self::Ok>;
    fn add_variant_type(&mut self, node: NodeId, type_name: &str, variant_name: &str) -> Result<Self::Ok>;
    fn add_tuple(&mut self, node: NodeId, pos: usize, value: NodeId) -> Result<Self::Ok>;
}

/// Implementation of [serde::Serializer] that extracts facts from a data structure.
/// The extractor generates facts from a data structure through flattening:
/// it generates unique identifiers, and references within a data structure
/// are ["unswizzled"](https://en.wikipedia.org/wiki/Pointer_swizzling)
/// into identifiers.
/// 
/// Note that the extractor does *not* contain an explicit representation of
/// the facts that it generates from a data structure. Instead, it calls out
/// to a [DatalogExtractorBackend] to materialize facts.
/// 
/// # Example
/// 
/// Consider the following enum type:
/// 
/// ```
/// enum Foo {
///     A(Box<Foo>),
///     B(i64)
/// }
/// ```
/// 
/// Then consider the enum instance `Foo::A(Foo::B(10))`. The extractor can
/// generate the following facts to represent this data structure:
/// 
/// - Element 1 is a newtype variant
/// - Element 1 has type `Foo` and variant name `A`
/// - The first field of Element 1 references Element 2
/// - Element 2 is a newtype variant
/// - Element 2 has type `Foo` and variant name `B`
/// - The first field of Element 2 references Element 3
/// - Element 3 is an i64
/// - Element 3 has value 10
/// 
/// Each of these facts are materialized by calling into the appropriate
/// method of an implementation of [DatalogExtractorBackend]:
/// 
/// - `add_node(NodeId(1), NodeType::TupleVariant)`
/// - `add_variant_type(NodeId(1), "Foo", "A")`
/// - `add_tuple(NodeId(1), 0, NodeId(2))`
/// - `add_node(NodeId(2), NodeType::TupleVariant)`
/// - `add_variant_type(NodeId(1), "Foo", "B")`
/// - `add_tuple(NodeId(2), 0, NodeId(3))`
/// - `add_node(NodeId(3), NodeType::I64)`
/// - `add_i64(NodeId(3), 10)`
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

    fn serialize_bool(self, value: bool) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Bool)?;
        self.backend.add_bool(id, value)
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::I8)?;
        self.backend.add_i8(id, value)
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::I16)?;
        self.backend.add_i16(id, value)
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::I32)?;
        self.backend.add_i32(id, value)
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::I64)?;
        self.backend.add_i64(id, value)
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::U8)?;
        self.backend.add_u8(id, value)
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::U16)?;
        self.backend.add_u16(id, value)
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::U32)?;
        self.backend.add_u32(id, value)
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::U64)?;
        self.backend.add_u64(id, value)
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::F32)?;
        self.backend.add_f32(id, value)
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::F64)?;
        self.backend.add_f64(id, value)
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Char)?;
        self.backend.add_char(id, value)
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Str)?;
        self.backend.add_str(id, value)
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        let id = self.get_fresh_node_id(NodeType::Bytes)?;
        self.backend.add_bytes(id, value)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit_variant("Option", 0, "None")
    }

    fn serialize_some<T: ?Sized + serde::Serialize>(self, value: &T) -> Result<Self::Ok> {
        self.serialize_newtype_variant("Option", 1, "Some", value)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.get_fresh_node_id(NodeType::Unit)?;
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
        let id = self.get_fresh_node_id(NodeType::NewtypeStruct)?;
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

        let id = self.get_fresh_node_id(NodeType::NewtypeVariant)?;
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
