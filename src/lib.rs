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

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct NodeId(usize);

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum NodeType {
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
