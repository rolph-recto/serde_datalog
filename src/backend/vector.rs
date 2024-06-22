//! A backend that stores facts as vectors of tuples.

use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
};

use crate::{DatalogExtractionError, DatalogExtractorBackend, ElemId, ElemType, Result};

/// Identifier for an interned string.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct SymbolId(pub usize);

impl Display for SymbolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

const BOOL_NAME: &str = "Bool";
const NUMBER_NAME: &str = "Number";
const STR_NAME: &str = "Str";
const MAP_NAME: &str = "Map";
const SEQ_NAME: &str = "Seq";
const STRUCT_NAME: &str = "Struct";
const STRUCT_VARIANT_NAME: &str = "StructVariant";
const TUPLE_NAME: &str = "Tuple";
const TUPLE_STRUCT_NAME: &str = "TupleStruct";
const TUPLE_VARIANT_NAME: &str = "TupleVariant";
const UNIT_NAME: &str = "Unit";
const UNIT_STRUCT_NAME: &str = "UnitStruct";
const UNIT_VARIANT_NAME: &str = "UnitVariant";

pub struct BackendData<K: Debug + Eq + Hash> {
    pub symbol_table: HashMap<String, SymbolId>,

    /// Columns: (elem, elem type)
    pub type_table: Vec<(ElemId, SymbolId)>,

    /// Stores values of boolean elements.
    /// Columns: (elem, value)
    pub bool_table: HashMap<ElemId, bool>,

    /// Stores values of number elements.
    /// Columns: (elem, value)
    pub number_table: HashMap<ElemId, isize>,

    /// Stores values of string elements.
    /// Columns: (elem, symbol)
    pub string_table: HashMap<ElemId, SymbolId>,

    /// Stores map entry facts.
    /// Columns: (elem, key, value)
    pub map_table: HashMap<(ElemId, K), ElemId>,

    /// Stores type names of structs.
    /// Columns: (elem, struct name)
    pub struct_type_table: HashMap<ElemId, SymbolId>,

    /// Stores struct field facts.
    /// Columns: (elem, field name, value elem)
    pub struct_table: HashMap<(ElemId, SymbolId), ElemId>,

    /// Stores sequence entry facts.
    /// Columns: (elem, index, value)
    pub seq_table: HashMap<(ElemId, usize), ElemId>,

    /// Stores type and variant names of variant elements.
    /// (elem, enum name, variant name)
    pub variant_type_table: HashMap<ElemId, (SymbolId, SymbolId)>,

    /// Stores tuple entry facts.
    /// Columns: (elem, index, value)
    pub tuple_table: HashMap<(ElemId, usize), ElemId>,
}

impl<K: Debug + Eq + Hash> Default for BackendData<K> {
    fn default() -> Self {
        Self {
            symbol_table: Default::default(),
            type_table: Default::default(),
            bool_table: Default::default(),
            number_table: Default::default(),
            string_table: Default::default(),
            map_table: Default::default(),
            struct_type_table: Default::default(),
            struct_table: Default::default(),
            seq_table: Default::default(),
            variant_type_table: Default::default(),
            tuple_table: Default::default(),
        }
    }
}

impl<K: Debug + Eq + Hash> BackendData<K> {
    /// Print generated fact tables to standard output.
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
            println!("{:^33}", "Type Table");
            println!("---------------------------------");
            println!("{:<15} | {:<15}", "Elem Id", "Elem Type");
            println!("---------------------------------");
            for (elem, elem_type) in self.type_table.iter() {
                println!("{:<15} | {:<15?}", elem.0, elem_type.0);
            }
            println!();
        }

        if !self.bool_table.is_empty() {
            println!("{:^33}", "Bool Table");
            println!("---------------------------------");
            println!("{:<15} | {:<15}", "Elem Id", "Value");
            println!("---------------------------------");
            for (elem, value) in self.bool_table.iter() {
                println!("{:<15} | {:<15?}", elem.0, value);
            }
            println!();
        }

        if !self.number_table.is_empty() {
            println!("{:^33}", "Number Table");
            println!("---------------------------------");
            println!("{:<15} | {:<15}", "Elem Id", "Value");
            println!("---------------------------------");
            for (elem, value) in self.number_table.iter() {
                println!("{:<15} | {:<15?}", elem.0, value);
            }
            println!();
        }

        if !self.string_table.is_empty() {
            println!("{:^33}", "String Table");
            println!("---------------------------------");
            println!("{:<15} | {:<15}", "Elem Id", "Value");
            println!("---------------------------------");
            for (elem, value) in self.string_table.iter() {
                println!("{:<15} | {:<15?}", elem.0, value.0);
            }
            println!();
        }

        if !self.map_table.is_empty() {
            println!("{:^51}", "Map Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "Elem Id", "Key", "Value");
            println!("---------------------------------------------------");
            for ((elem, key), val) in self.map_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", elem.0, key, val.0);
            }
            println!();
        }

        if !self.struct_type_table.is_empty() {
            println!("{:^33}", "Struct Type Table");
            println!("---------------------------------");
            println!("{:<15} | {:<15}", "Elem Id", "Struct Type");
            println!("---------------------------------");
            for (elem, struct_type) in self.string_table.iter() {
                println!("{:<15} | {:<15?}", elem.0, struct_type.0);
            }
            println!();
        }

        if !self.struct_table.is_empty() {
            println!("{:^51}", "Struct Field Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "Elem Id", "Field", "Value");
            println!("---------------------------------------------------");
            for ((elem, field), val) in self.struct_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", elem.0, field.0, val.0);
            }
            println!();
        }

        if !self.seq_table.is_empty() {
            println!("{:^51}", "Seq Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "Elem Id", "Index", "Value");
            println!("---------------------------------------------------");
            for ((elem, index), val) in self.seq_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", elem.0, index, val.0);
            }
            println!();
        }

        if !self.variant_type_table.is_empty() {
            println!("{:^51}", "Variant Type Table");
            println!("---------------------------------------------------");
            println!(
                "{:<15} | {:<15} | {:<15}",
                "Elem Id", "Enum Type", "Variant Name"
            );
            println!("---------------------------------------------------");
            for (elem, (enum_type, variant_name)) in self.variant_type_table.iter() {
                println!(
                    "{:<15} | {:<15?} | {:<15?}",
                    elem.0, enum_type.0, variant_name.0
                );
            }
            println!();
        }

        if !self.tuple_table.is_empty() {
            println!("{:^51}", "Tuple Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "Elem Id", "Index", "Value");
            println!("---------------------------------------------------");
            for ((elem, index), val) in self.tuple_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", elem.0, index, val.0);
            }
            println!();
        }
    }
}

/// DatalogExtractorBackend impl that stores facts in vectors.
/// Note that this backend interns strings, so tables store a string's
/// [SymbolId] instead of the string itself.
///
/// Note that this backend does **not** support extraction of
/// floating point values, and will return a
/// [UnextractableData][DatalogExtractionError::UnextractableData] error if
/// the input contains such values.
pub struct AbstractBackend<K: Debug + Eq + Hash> {
    pub(crate) cur_symbol_id: SymbolId,
    pub(crate) data: BackendData<K>,
}

impl<K: Debug + Eq + Hash> Default for AbstractBackend<K> {
    fn default() -> Self {
        let mut backend = Self {
            cur_symbol_id: SymbolId(1),
            data: Default::default(),
        };

        backend.intern_string(BOOL_NAME);
        backend.intern_string(NUMBER_NAME);
        backend.intern_string(STR_NAME);
        backend.intern_string(MAP_NAME);
        backend.intern_string(SEQ_NAME);
        backend.intern_string(STRUCT_NAME);
        backend.intern_string(STRUCT_VARIANT_NAME);
        backend.intern_string(TUPLE_NAME);
        backend.intern_string(TUPLE_STRUCT_NAME);
        backend.intern_string(TUPLE_VARIANT_NAME);
        backend.intern_string(UNIT_NAME);
        backend.intern_string(UNIT_STRUCT_NAME);
        backend.intern_string(UNIT_VARIANT_NAME);

        backend
    }
}

impl<K: Debug + Eq + Hash> AbstractBackend<K> {
    fn intern_string(&mut self, s: &str) -> SymbolId {
        match self.data.symbol_table.get(s) {
            Some(id) => *id,
            None => {
                let SymbolId(id) = self.cur_symbol_id;
                self.data
                    .symbol_table
                    .insert(s.to_string(), self.cur_symbol_id);
                self.cur_symbol_id.0 += 1;
                SymbolId(id)
            }
        }
    }

    pub fn get_data(self) -> BackendData<K> {
        self.data
    }

    fn add_elem(&mut self, elem: ElemId, elem_type: ElemType) -> Result<()> {
        let type_name: &str = match elem_type {
            ElemType::Bool => BOOL_NAME,

            ElemType::I8
            | ElemType::I16
            | ElemType::I32
            | ElemType::I64
            | ElemType::U8
            | ElemType::U16
            | ElemType::U32
            | ElemType::U64 => NUMBER_NAME,

            ElemType::Char | ElemType::Str => STR_NAME,

            ElemType::F32 | ElemType::F64 | ElemType::Bytes => {
                return Result::Err(DatalogExtractionError::UnextractableData);
            }

            ElemType::Map => MAP_NAME,
            ElemType::Seq => SEQ_NAME,
            ElemType::Struct => STRUCT_NAME,
            ElemType::StructVariant => STRUCT_VARIANT_NAME,
            ElemType::Tuple => TUPLE_NAME,
            ElemType::TupleStruct | ElemType::NewtypeStruct => TUPLE_STRUCT_NAME,

            ElemType::TupleVariant | ElemType::NewtypeVariant => TUPLE_VARIANT_NAME,

            ElemType::Unit => UNIT_NAME,
            ElemType::UnitStruct => UNIT_STRUCT_NAME,
            ElemType::UnitVariant => UNIT_VARIANT_NAME,
        };

        let elem_type_sym = self.intern_string(type_name);
        self.data.type_table.push((elem, elem_type_sym));
        Result::Ok(())
    }

    fn add_bool(&mut self, elem: ElemId, value: bool) -> Result<()> {
        assert!(self.data.bool_table.insert(elem, value).is_none());
        Result::Ok(())
    }

    fn add_i64(&mut self, elem: ElemId, value: i64) -> Result<()> {
        assert!(self
            .data
            .number_table
            .insert(elem, value as isize)
            .is_none());
        Result::Ok(())
    }

    fn add_u64(&mut self, elem: ElemId, value: u64) -> Result<()> {
        assert!(self
            .data
            .number_table
            .insert(elem, value as isize)
            .is_none());
        Result::Ok(())
    }

    fn add_f64(&mut self, _elem: ElemId, _value: f64) -> Result<()> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    fn add_bytes(&mut self, _elem: ElemId, _value: &[u8]) -> Result<()> {
        Result::Err(DatalogExtractionError::UnextractableData)
    }

    fn add_str(&mut self, elem: ElemId, value: &str) -> Result<()> {
        let value_sym = self.intern_string(value);
        assert!(self.data.string_table.insert(elem, value_sym).is_none());
        Result::Ok(())
    }

    fn add_struct_type(&mut self, elem: ElemId, struct_name: &str) -> Result<()> {
        let struct_name_sym = self.intern_string(struct_name);
        assert!(self
            .data
            .struct_type_table
            .insert(elem, struct_name_sym)
            .is_none());
        Result::Ok(())
    }

    fn add_struct_entry(&mut self, elem: ElemId, key: &str, value: ElemId) -> Result<()> {
        let key_sym = self.intern_string(key);
        assert!(self
            .data
            .struct_table
            .insert((elem, key_sym), value)
            .is_none());
        Result::Ok(())
    }

    fn add_seq_entry(&mut self, elem: ElemId, pos: usize, value: ElemId) -> Result<()> {
        assert!(self.data.seq_table.insert((elem, pos), value).is_none());
        Result::Ok(())
    }

    fn add_variant_type(
        &mut self,
        elem: ElemId,
        type_name: &str,
        variant_name: &str,
    ) -> Result<()> {
        let type_name_sym = self.intern_string(type_name);
        let variant_name_sym = self.intern_string(variant_name);
        assert!(self
            .data
            .variant_type_table
            .insert(elem, (type_name_sym, variant_name_sym))
            .is_none());
        Result::Ok(())
    }

    fn add_tuple_entry(&mut self, elem: ElemId, pos: usize, value: ElemId) -> Result<()> {
        assert!(self.data.tuple_table.insert((elem, pos), value).is_none());
        Result::Ok(())
    }
}

#[derive(Default)]
pub struct Backend {
    parent: AbstractBackend<ElemId>,
}

impl Backend {
    pub fn get_data(self) -> BackendData<ElemId> {
        self.parent.get_data()
    }
}

impl DatalogExtractorBackend for &mut Backend {
    fn add_elem(&mut self, elem: ElemId, elem_type: ElemType) -> Result<()> {
        self.parent.add_elem(elem, elem_type)
    }

    fn add_bool(&mut self, elem: ElemId, value: bool) -> Result<()> {
        self.parent.add_bool(elem, value)
    }

    fn add_i64(&mut self, elem: ElemId, value: i64) -> Result<()> {
        self.parent.add_i64(elem, value)
    }

    fn add_u64(&mut self, elem: ElemId, value: u64) -> Result<()> {
        self.parent.add_u64(elem, value)
    }

    fn add_f64(&mut self, elem: ElemId, value: f64) -> Result<()> {
        self.parent.add_f64(elem, value)
    }

    fn add_str(&mut self, elem: ElemId, value: &str) -> Result<()> {
        self.parent.add_str(elem, value)
    }

    fn add_bytes(&mut self, elem: ElemId, value: &[u8]) -> Result<()> {
        self.parent.add_bytes(elem, value)
    }

    fn add_map_entry(&mut self, elem: ElemId, key: ElemId, value: ElemId) -> Result<()> {
        assert!(self
            .parent
            .data
            .map_table
            .insert((elem, key), value)
            .is_none());
        Result::Ok(())
    }

    fn add_struct_type(&mut self, elem: ElemId, struct_name: &str) -> Result<()> {
        self.parent.add_struct_type(elem, struct_name)
    }

    fn add_struct_entry(&mut self, elem: ElemId, key: &str, value: ElemId) -> Result<()> {
        self.parent.add_struct_entry(elem, key, value)
    }

    fn add_seq_entry(&mut self, elem: ElemId, pos: usize, value: ElemId) -> Result<()> {
        self.parent.add_seq_entry(elem, pos, value)
    }

    fn add_variant_type(
        &mut self,
        elem: ElemId,
        type_name: &str,
        variant_name: &str,
    ) -> Result<()> {
        self.parent.add_variant_type(elem, type_name, variant_name)
    }

    fn add_tuple_entry(&mut self, elem: ElemId, pos: usize, value: ElemId) -> Result<()> {
        self.parent.add_tuple_entry(elem, pos, value)
    }
}

#[derive(Default)]
pub struct StringKeyBackend {
    parent: AbstractBackend<SymbolId>,
}

impl StringKeyBackend {
    pub fn get_data(self) -> BackendData<SymbolId> {
        self.parent.get_data()
    }
}

impl DatalogExtractorBackend for &mut StringKeyBackend {
    fn add_elem(&mut self, elem: ElemId, elem_type: ElemType) -> Result<()> {
        self.parent.add_elem(elem, elem_type)
    }

    fn add_bool(&mut self, elem: ElemId, value: bool) -> Result<()> {
        self.parent.add_bool(elem, value)
    }

    fn add_i64(&mut self, elem: ElemId, value: i64) -> Result<()> {
        self.parent.add_i64(elem, value)
    }

    fn add_u64(&mut self, elem: ElemId, value: u64) -> Result<()> {
        self.parent.add_u64(elem, value)
    }

    fn add_f64(&mut self, elem: ElemId, value: f64) -> Result<()> {
        self.parent.add_f64(elem, value)
    }

    fn add_str(&mut self, elem: ElemId, value: &str) -> Result<()> {
        self.parent.add_str(elem, value)
    }

    fn add_bytes(&mut self, elem: ElemId, value: &[u8]) -> Result<()> {
        self.parent.add_bytes(elem, value)
    }

    fn add_map_entry(&mut self, elem: ElemId, key: ElemId, value: ElemId) -> Result<()> {
        if let Some(sym) = self.parent.data.string_table.remove(&key) {
            assert!(self
                .parent
                .data
                .map_table
                .insert((elem, sym), value)
                .is_none());
            Result::Ok(())
        } else {
            Result::Err(DatalogExtractionError::UnextractableData)
        }
    }

    fn add_struct_type(&mut self, elem: ElemId, struct_name: &str) -> Result<()> {
        self.parent.add_struct_type(elem, struct_name)
    }

    fn add_struct_entry(&mut self, elem: ElemId, key: &str, value: ElemId) -> Result<()> {
        self.parent.add_struct_entry(elem, key, value)
    }

    fn add_seq_entry(&mut self, elem: ElemId, pos: usize, value: ElemId) -> Result<()> {
        self.parent.add_seq_entry(elem, pos, value)
    }

    fn add_variant_type(
        &mut self,
        elem: ElemId,
        type_name: &str,
        variant_name: &str,
    ) -> Result<()> {
        self.parent.add_variant_type(elem, type_name, variant_name)
    }

    fn add_tuple_entry(&mut self, elem: ElemId, pos: usize, value: ElemId) -> Result<()> {
        self.parent.add_tuple_entry(elem, pos, value)
    }
}
