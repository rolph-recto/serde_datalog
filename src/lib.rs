//! This crate provides functionality to serialize any data structure whose type
//! implements [serde::Serialize] to be converted into a database of facts that
//! can be used as input to Datalog programs. In Datalog parlance,
//! this crate serializes data structures to EDBs.
//! 
//! There are two main components. [DatalogExtractor] is an implementation of
//! [serde::Serializer] that generates facts from data structures mapped onto
//! [Serde's data model](https://serde.rs/data-model.html). The extractor does
//! not explicitly represent these facts; instead, it calls out into
//! implementations of the [DatalogExtractorBackend] trait, which materializes
//! these facts into some explicit representation. For example,
//! the [backend::souffle_sqlite::Backend] stores these facts within tables
//! of a [SQLite](https://www.sqlite.org/) database.
//! 
//! # Example
//! 
//! Consider the following enum type:
//! 
//! ```ignore
//! enum Foo {
//!     A(Box<Foo>),
//!     B(i64)
//! }
//! ```
//! 
//! Then consider the enum instance `Foo::A(Foo::B(10))`. The extractor
//! generates the following facts to represent this data structure:
//! 
//! - Element 1 is a newtype variant
//! - Element 1 has type `Foo` and variant name `A`
//! - The first field of Element 1 references Element 2
//! - Element 2 is a newtype variant
//! - Element 2 has type `Foo` and variant name `B`
//! - The first field of Element 2 references Element 3
//! - Element 3 is an i64
//! - Element 3 has value 10
//! 
//! The extractor generates facts from a data structure through flattening:
//! it generates unique identifiers for each element within the data structure,
//! and references between elements are
//! ["unswizzled"](https://en.wikipedia.org/wiki/Pointer_swizzling)
//! into identifiers.
//! 
//! For each of these facts, the extractor will make the following calls to an
//! implementation of [DatalogExtractorBackend]:
//! 
//! ```ignore
//! backend.add_elem(ElemId(1), ElemType::TupleVariant)
//! backend.add_variant_type(ElemId(1), "Foo", "A")
//! backend.add_tuple(ElemId(1), 0, ElemId(2))
//! backend.add_elem(ElemId(2), ElemType::TupleVariant)
//! backend.add_variant_type(ElemId(1), "Foo", "B")
//! backend.add_tuple(ElemId(2), 0, ElemId(3))
//! backend.add_elem(ElemId(3), ElemType::I64)
//! backend.add_i64(ElemId(3), 10)
//! ```

use serde::ser;
use std::{
    result,
    fmt::{self, Display}
};

pub mod backend;

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
pub struct ElemId(usize);

/// Enumeration of possible element types within a data structure.
/// These correspond directly to the types in
/// [Serde's data model](https://serde.rs/data-model.html).
/// The main exception is that this enum does not contain a variant for
/// option types; instead of treating it as a special case,
/// [DatalogExtractor] instead treats option values as regular enum values.
/// That is, `None` values are treated as unit variants with type name `Option`
/// and variant name `None`, while `Some` values are treated as newtype variants
/// with type name `Option` and variant name `Some`.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum ElemType {
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
    /// Materialize fact that element with ID `elem` has element type `elem_type`.
    fn add_elem(&mut self, elem: ElemId, elem_type: ElemType) -> Result<()>;

    /// Materialize fact that element with ID `elem` is a boolean with value `value`.
    fn add_bool(&mut self, _elem: ElemId, _value: bool) -> Result<()> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    /// Materialize fact that element with ID `elem` is an i8 with value `value`.
    /// 
    /// The default implementation forwards to [add_i64][Self::add_i64].
    fn add_i8(&mut self, elem: ElemId, value: i8) -> Result<()> {
        self.add_i64(elem, value as i64)
    }

    /// Materialize fact that element with ID `elem` is an i16 with value `value`.
    /// 
    /// The default implementation forwards to [add_i64][Self::add_i64].
    fn add_i16(&mut self, elem: ElemId, value: i16) -> Result<()> {
        self.add_i64(elem, value as i64)
    }

    /// Materialize fact that element with ID `elem` is an i32 with value `value`.
    fn add_i32(&mut self, elem: ElemId, value: i32) -> Result<()> {
        self.add_i64(elem, value as i64)
    }

    /// Materialize fact that element with ID `elem` is an i64 with value `value`.
    /// 
    /// The default implementation returns an
    /// [UnextractableData][DatalogExtractionError::UnextractableData] error.
    fn add_i64(&mut self, _elem: ElemId, _value: i64) -> Result<()> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    /// Materialize fact that element with ID `elem` is an u8 with value `value`.
    /// 
    /// The default implementation forwards to [add_u64][Self::add_u64].
    fn add_u8(&mut self, elem: ElemId, value: u8) -> Result<()> {
        self.add_u64(elem, value as u64)
    }

    /// Materialize fact that element with ID `elem` is an u16 with value `value`.
    /// 
    /// The default implementation forwards to [add_u64][Self::add_u64].
    fn add_u16(&mut self, elem: ElemId, value: u16) -> Result<()> {
        self.add_u64(elem, value as u64)
    }

    /// Materialize fact that element with ID `elem` is an u32 with value `value`.
    fn add_u32(&mut self, elem: ElemId, value: u32) -> Result<()> {
        self.add_u64(elem, value as u64)
    }

    /// Materialize fact that element with ID `elem` is an u64 with value `value`.
    /// 
    /// The default implementation returns an
    /// [UnextractableData][DatalogExtractionError::UnextractableData] error.
    fn add_u64(&mut self, _elem: ElemId, _value: u64) -> Result<()> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    /// Materialize fact that element with ID `elem` is a f32 with value `value`.
    fn add_f32(&mut self, elem: ElemId, value: f32) -> Result<()> {
        self.add_f64(elem, value as f64)
    }

    /// Materialize fact that element with ID `elem` is a f64 with value `value`.
    /// 
    /// The default implementation returns an
    /// [UnextractableData][DatalogExtractionError::UnextractableData] error.
    fn add_f64(&mut self, _elem: ElemId, _value: f64) -> Result<()> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    /// Materialize fact that element with ID `elem` is a char with value `value`.
    /// 
    /// The default implementation forwards to [add_str][Self::add_str].
    fn add_char(&mut self, elem: ElemId, value: char) -> Result<()> {
        self.add_str(elem, &value.to_string())
    }

    /// Materialize fact that element with ID `elem` is a str with value `value`.
    fn add_str(&mut self, _elem: ElemId, _value: &str) -> Result<()> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    /// Materialize fact that element with ID `elem` is a byte array with value `value`.
    /// 
    /// The default implementation returns an
    /// [UnextractableData][DatalogExtractionError::UnextractableData] error.
    fn add_bytes(&mut self, _elem: ElemId, _value: &[u8]) -> Result<()> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    /// Materialize fact that element with ID `elem` is a map with
    /// key `key` mapped to value `value`.
    /// 
    /// The default implementation returns an
    /// [UnextractableData][DatalogExtractionError::UnextractableData] error.
    fn add_map_entry(&mut self, _elem: ElemId, _key: ElemId, _value: ElemId) -> Result<()> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    /// Materialize fact that element with ID `elem` is a struct
    /// with type name `struct_name`. The element can have element type of either
    /// [ElemType::NewtypeStruct], [ElemType::Struct], [ElemType::TupleStruct],
    /// or [ElemType::UnitStruct].
    /// 
    /// The default implementation returns an
    /// [UnextractableData][DatalogExtractionError::UnextractableData] error.
    fn add_struct_type(&mut self, _elem: ElemId, _struct_name: &str) -> Result<()> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    /// Materialize fact that element with ID `elem` is a struct, 
    /// with value `value` at key `key`.
    /// The element can have element type [ElemType::Struct] or
    /// [ElemType::StructVariant].
    /// 
    /// The default implementation returns an
    /// [UnextractableData][DatalogExtractionError::UnextractableData] error.
    fn add_struct_entry(&mut self, _elem: ElemId, _key: &str, _value: ElemId) -> Result<()> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    /// Materialize fact that element with ID `elem` is a sequence
    /// with value `value` at position `pos`.
    /// 
    /// The default implementation returns an
    /// [UnextractableData][DatalogExtractionError::UnextractableData] error.
    fn add_seq_entry(&mut self, _elem: ElemId, _pos: usize, _value: ElemId) -> Result<()> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    /// Materialize fact that element with ID `elem` is an enum variant
    /// with type name `type_name` and variant name `variant_name`.
    /// The element can have element type 
    /// [ElemType::NewtypeVariant], [ElemType::StructVariant],
    /// [ElemType::TupleVariant], or [ElemType::UnitVariant].
    /// 
    /// The default implementation returns an
    /// [UnextractableData][DatalogExtractionError::UnextractableData] error.
    fn add_variant_type(&mut self, _elem: ElemId, _type_name: &str, _variant_name: &str) -> Result<()> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    /// Materialize fact that element with ID `elem` is a tuple with value
    /// `value` at position `pos`.
    /// The element can have element type [ElemType::NewtypeStruct],
    /// [ElemType::NewtypeVariant], [ElemType::Tuple], [ElemType::TupleStruct],
    /// or [ElemType::TupleVariant].
    /// 
    /// The default implementation returns an
    /// [UnextractableData][DatalogExtractionError::UnextractableData] error.
    fn add_tuple_entry(&mut self, _elem: ElemId, _pos: usize, _value: ElemId) -> Result<()> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }
}

/// Implementation of [serde::Serializer] that extracts facts from a data structure.
/// Note that the extractor does *not* contain an explicit representation of
/// the facts that it generates from a data structure. Instead, it calls out
/// to a [DatalogExtractorBackend] to materialize facts.
pub struct DatalogExtractor<'a> {
    cur_elem_id: ElemId,
    elem_stack: Vec<ElemId>,
    parent_stack: Vec<(ElemId, usize)>,
    backend: Box<dyn DatalogExtractorBackend + 'a>,
}

impl<'a> DatalogExtractor<'a> {
    pub fn new<T: 'a + DatalogExtractorBackend>(backend: T) -> Self {
        DatalogExtractor {
            backend: Box::new(backend),
            cur_elem_id: ElemId(1),
            elem_stack: Vec::new(),
            parent_stack: Vec::new(),
        }
    }

    fn get_fresh_elem_id(&mut self, elem_type: ElemType) -> Result<ElemId> {
        let id = self.cur_elem_id;
        self.backend.add_elem(id, elem_type)?;
        self.elem_stack.push(id);
        self.cur_elem_id.0 += 1;
        Result::Ok(id)
    }

    fn serialize_tuple_or_seq_element<T: ?Sized + serde::Serialize>(&mut self, value: &T, elem_type: ElemType) -> Result<()> {
        value.serialize(&mut *self)?;
        let child_id = self.elem_stack.pop().unwrap();
        let (parent_id , pos) = self.parent_stack.last_mut().unwrap();

        match elem_type {
            ElemType::Seq => {
                self.backend.add_seq_entry(*parent_id, *pos, child_id)
            }

            ElemType::Tuple | ElemType::TupleStruct | ElemType::TupleVariant => {
                self.backend.add_tuple_entry(*parent_id, *pos, child_id)
            }

            _ => unreachable!()
        }?;

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
        let (parent_id, _) = self.parent_stack.last_mut().unwrap();
        let val_id = self.elem_stack.pop().unwrap();
        self.backend.add_struct_entry(*parent_id, key, val_id)
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

    /// Generate facts about a boolean value.
    /// Given a fresh element ID `id`, this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::Bool)
    /// add_bool(id, value)
    /// ```
    fn serialize_bool(self, value: bool) -> Result<Self::Ok> {
        let id = self.get_fresh_elem_id(ElemType::Bool)?;
        self.backend.add_bool(id, value)
    }

    /// Generate facts about an i8 value.
    /// Given a fresh element ID `id`, this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::I8)
    /// add_i8(id, value)
    /// ```
    fn serialize_i8(self, value: i8) -> Result<Self::Ok> {
        let id = self.get_fresh_elem_id(ElemType::I8)?;
        self.backend.add_i8(id, value)
    }

    /// Generate facts about an i16 value.
    /// Given a fresh element ID `id`, this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::I16)
    /// add_i16(id, value)
    /// ```
    fn serialize_i16(self, value: i16) -> Result<Self::Ok> {
        let id = self.get_fresh_elem_id(ElemType::I16)?;
        self.backend.add_i16(id, value)
    }

    /// Generate facts about an i32 value.
    /// Given a fresh element ID `id`, this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::I32)
    /// add_i32(id, value)
    /// ```
    fn serialize_i32(self, value: i32) -> Result<Self::Ok> {
        let id = self.get_fresh_elem_id(ElemType::I32)?;
        self.backend.add_i32(id, value)
    }

    /// Generate facts about an i64 value.
    /// Given a fresh element ID `id`, this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::I64)
    /// add_i64(id, value)
    /// ```
    fn serialize_i64(self, value: i64) -> Result<Self::Ok> {
        let id = self.get_fresh_elem_id(ElemType::I64)?;
        self.backend.add_i64(id, value)
    }

    /// Generate facts about an u8 value.
    /// Given a fresh element ID `id`, this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::U8)
    /// add_u8(id, value)
    /// ```
    fn serialize_u8(self, value: u8) -> Result<Self::Ok> {
        let id = self.get_fresh_elem_id(ElemType::U8)?;
        self.backend.add_u8(id, value)
    }

    /// Generate facts about an u16 value.
    /// Given a fresh element ID `id`, this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::I8)
    /// add_u16(id, value)
    /// ```
    fn serialize_u16(self, value: u16) -> Result<Self::Ok> {
        let id = self.get_fresh_elem_id(ElemType::U16)?;
        self.backend.add_u16(id, value)
    }

    /// Generate facts about an u32 value.
    /// Given a fresh element ID `id`, this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::U32)
    /// add_u32(id, value)
    /// ```
    fn serialize_u32(self, value: u32) -> Result<Self::Ok> {
        let id = self.get_fresh_elem_id(ElemType::U32)?;
        self.backend.add_u32(id, value)
    }

    /// Generate facts about an u64 value.
    /// Given a fresh element ID `id`, this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::U64)
    /// add_u64(id, value)
    /// ```
    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        let id = self.get_fresh_elem_id(ElemType::U64)?;
        self.backend.add_u64(id, value)
    }

    /// Generate facts about an f32 value.
    /// Given a fresh element ID `id`, this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::F32)
    /// add_f32(id, value)
    /// ```
    fn serialize_f32(self, value: f32) -> Result<Self::Ok> {
        let id = self.get_fresh_elem_id(ElemType::F32)?;
        self.backend.add_f32(id, value)
    }

    /// Generate facts about an f64 value.
    /// Given a fresh element ID `id`, this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::F64)
    /// add_f64(id, value)
    /// ```
    fn serialize_f64(self, value: f64) -> Result<Self::Ok> {
        let id = self.get_fresh_elem_id(ElemType::F64)?;
        self.backend.add_f64(id, value)
    }

    /// Generate facts about a char value.
    /// Given a fresh element ID `id`, this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::Char)
    /// add_char(id, value)
    /// ```
    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        let id = self.get_fresh_elem_id(ElemType::Char)?;
        self.backend.add_char(id, value)
    }

    /// Generate facts about a str value.
    /// Given a fresh element ID `id`, this will make the following call to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::Str)
    /// add_str(id, value)
    /// ```
    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        let id = self.get_fresh_elem_id(ElemType::Str)?;
        self.backend.add_str(id, value)
    }

    /// Generate facts about a byte array value.
    /// Given a fresh element ID `id`, this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::Bytes)
    /// add_bytes(id, value)
    /// ```
    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        let id = self.get_fresh_elem_id(ElemType::Bytes)?;
        self.backend.add_bytes(id, value)
    }

    /// Generate facts about a None value.
    /// Given a fresh element ID `id`, this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::UnitVariant)
    /// add_variant_type(id, "Option", "None")
    /// ```
    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit_variant("Option", 0, "None")
    }

    /// Generate facts about a Some value.
    /// Given a fresh element ID `id`, and that `value` has element ID `value_id`,
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::NewtypeVariant)
    /// add_variant_type(id, "Option", "Some")
    /// add_tuple_entry(id, 0, value_id)
    /// ```
    fn serialize_some<T: ?Sized + serde::Serialize>(self, value: &T) -> Result<Self::Ok> {
        self.serialize_newtype_variant("Option", 1, "Some", value)
    }

    /// Generate facts about a unit value.
    /// Given a fresh element ID `id` this will make the following call to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::Unit)
    /// ```
    fn serialize_unit(self) -> Result<Self::Ok> {
        self.get_fresh_elem_id(ElemType::Unit)?;
        Result::Ok(())
    }

    /// Generate facts about a unit struct value.
    /// Given a fresh element ID `id` this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::UnitStruct)
    /// add_struct_type(id, name)
    /// ```
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        let id = self.get_fresh_elem_id(ElemType::UnitStruct)?;
        self.backend.add_struct_type(id, name)
    }

    /// Generate facts about a unit variant value.
    /// Given a fresh element ID `id` this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::UnitVariant)
    /// add_variant_type(id, name, variant)
    /// ```
    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        let id = self.get_fresh_elem_id(ElemType::UnitVariant)?;
        self.elem_stack.push(id);
        self.backend.add_variant_type(id, name, variant)
    }

    /// Generate facts about a newtype struct value.
    /// Given a fresh element ID `id` and that `value` has element ID `value_id`,
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::NewtypeStruct)
    /// add_struct_type(id, name)
    /// add_tuple_entry(id, 0, value_id)
    /// ```
    fn serialize_newtype_struct<T: ?Sized + serde::Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        value.serialize(&mut *self)?;
        let child_id = self.elem_stack.pop().unwrap();
        let id = self.get_fresh_elem_id(ElemType::NewtypeStruct)?;
        self.backend.add_struct_type(id, name)?;
        self.backend.add_tuple_entry(id, 0, child_id)
    }

    /// Generate facts about a newtype variant value.
    /// Given a fresh element ID `id` and that `value` has element ID `value_id`,
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::NewtypeVariant)
    /// add_variant_type(id, name, variant)
    /// add_tuple_entry(id, 0, value_id)
    /// ```
    fn serialize_newtype_variant<T: ?Sized + serde::Serialize>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        value.serialize(&mut *self)?;
        let child_id = self.elem_stack.pop().unwrap();

        let id = self.get_fresh_elem_id(ElemType::NewtypeVariant)?;
        self.backend.add_variant_type(id, name, variant)?;
        self.backend.add_tuple_entry(id, 0, child_id)
    }

    /// Generate facts about a sequence value.
    /// Given a fresh element ID `id`
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::Seq)
    /// ```
    fn serialize_seq(self, _len_opt: Option<usize>) -> Result<Self::SerializeSeq> {
        let id = self.get_fresh_elem_id(ElemType::Seq)?;
        self.parent_stack.push((id, 0));
        Result::Ok(self)
    }

    /// Generate facts about a tuple value.
    /// Given a fresh element ID `id`
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::Tuple)
    /// ```
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        let id = self.get_fresh_elem_id(ElemType::Tuple)?;
        self.parent_stack.push((id, 0));
        Result::Ok(self)
    }

    /// Generate facts about a tuple struct value.
    /// Given a fresh element ID `id`
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::TupleStruct)
    /// add_struct_type(id, name)
    /// ```
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        let id = self.get_fresh_elem_id(ElemType::TupleStruct)?;
        self.parent_stack.push((id, 0));
        self.backend.add_struct_type(id, name)?;
        Result::Ok(self)
    }

    /// Generate facts about a tuple variant value.
    /// Given a fresh element ID `id`
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::TupleVariant)
    /// add_variant_type(id, name, variant)
    /// ```
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        let id = self.get_fresh_elem_id(ElemType::TupleVariant)?;
        self.parent_stack.push((id, 0));
        self.backend.add_variant_type(id, name, variant)?;
        Result::Ok(self)
    }

    /// Generate facts about a map value.
    /// Given a fresh element ID `id`
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::Map)
    /// ```
    fn serialize_map(self, _len_opt: Option<usize>) -> Result<Self::SerializeMap> {
        let id = self.get_fresh_elem_id(ElemType::Map)?;
        self.parent_stack.push((id, 0));
        Result::Ok(self)
    }

    /// Generate facts about a struct value.
    /// Given a fresh element ID `id`
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::Struct)
    /// add_struct_type(id, name)
    /// ```
    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct> {
        let id = self.get_fresh_elem_id(ElemType::Struct)?;
        self.parent_stack.push((id, 0));
        self.backend.add_struct_type(id, name)?;
        Result::Ok(self)
    }

    /// Generate facts about a struct value.
    /// Given a fresh element ID `id`
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_elem(id, ElemType::StructVariant)
    /// add_variant_type(id, name, variant)
    /// ```
    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        let id = self.get_fresh_elem_id(ElemType::StructVariant)?;
        self.parent_stack.push((id, 0));
        self.backend.add_variant_type(id, name, variant)?;
        Result::Ok(self)
    }
}

impl<'a, 'b> ser::SerializeSeq for &'a mut DatalogExtractor<'b> {
    type Ok = ();
    type Error = DatalogExtractionError;

    /// Generate facts about a sequence element.
    /// Given that the parent sequence has element ID `parent_id`,
    /// that `value` has element ID `value_id`,
    /// and that the current sequence position is `pos`,
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_seq_entry(parent_id, pos, value_id)
    /// ```
    fn serialize_element<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        self.serialize_tuple_or_seq_element(value, ElemType::Seq)
    }

    fn end(self) -> Result<()> {
        self.end_parent()
    }
}

impl<'a, 'b> ser::SerializeTuple for &'a mut DatalogExtractor<'b> {
    type Ok = ();
    type Error = DatalogExtractionError;

    /// Generate facts about a tuple entry.
    /// Given that the parent tuple has element ID `parent_id`,
    /// that `value` has element ID `value_id`,
    /// and that the current tuple position is `pos`,
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_tuple_entry(parent_id, pos, value_id)
    /// ```
    fn serialize_element<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        self.serialize_tuple_or_seq_element(value, ElemType::Tuple)
    }

    fn end(self) -> Result<()> {
        self.end_parent()
    }
}

impl<'a, 'b> ser::SerializeTupleVariant for &'a mut DatalogExtractor<'b> {
    type Ok = ();
    type Error = DatalogExtractionError;

    /// Generate facts about a tuple variant entry.
    /// Given that the parent variant has element ID `parent_id`,
    /// that `value` has element ID `value_id`,
    /// and that the current tuple position is `pos`,
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_tuple_entry(parent_id, pos, value_id)
    /// ```
    fn serialize_field<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        self.serialize_tuple_or_seq_element(value, ElemType::TupleVariant)
    }

    fn end(self) -> Result<Self::Ok> {
        self.end_parent()
    }
}

impl<'a, 'b> ser::SerializeTupleStruct for &'a mut DatalogExtractor<'b> {
    type Ok = ();
    type Error = DatalogExtractionError;

    /// Generate facts about a tuple struct entry.
    /// Given that the parent struct has element ID `parent_id`,
    /// that `value` has element ID `value_id`,
    /// and that the current tuple position is `pos`,
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_tuple_entry(parent_id, pos, value_id)
    /// ```
    fn serialize_field<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        self.serialize_tuple_or_seq_element(value, ElemType::TupleStruct)
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

    /// Generate facts about a map entry.
    /// Given that the parent struct has element ID `parent_id`,
    /// that `value` has element ID `value_id`,
    /// and that the corresponding key has element ID `key_id`,
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_map_entry(parent_id, key_id, value_id)
    /// ```
    fn serialize_value<T: ?Sized + serde::Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        value.serialize(&mut **self)?;
        let (parent_id, _) = self.parent_stack.last().unwrap();
        let val_id = self.elem_stack.pop().unwrap();
        let key_id = self.elem_stack.pop().unwrap();
        self.backend.add_map_entry(*parent_id, key_id, val_id)
    }

    fn end(self) -> result::Result<Self::Ok, Self::Error> {
        self.end_parent()
    }
}

impl<'a, 'b> ser::SerializeStruct for &'a mut DatalogExtractor<'b> {
    type Ok = ();
    type Error = DatalogExtractionError;

    /// Generate facts about a struct entry.
    /// Given that the parent struct has element ID `parent_id` and
    /// that `value` has element ID `value_id`,
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_struct_entry(parent_id, key, value_id)
    /// ```
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

    /// Generate facts about a struct variant entry.
    /// Given that the parent struct has element ID `parent_id` and
    /// that `value` has element ID `value_id`,
    /// this will make the following calls to
    /// an implementation of [DatalogExtractorBackend]:
    /// 
    /// ```ignore
    /// add_struct_entry(parent_id, key, value_id)
    /// ```
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
