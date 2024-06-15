//! A backend that stores facts as vectors of tuples.

use std::collections::HashMap;

use crate::{ElemId, DatalogExtractorBackend, ElemType, Result, DatalogExtractionError};

/// Identifier for an interned string.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct SymbolId(pub usize);

const BOOL_NAME: &'static str = "Bool";
const NUMBER_NAME: &'static str = "Number";
const STR_NAME: &'static str = "Str";
const MAP_NAME: &'static str = "Map";
const SEQ_NAME: &'static str = "Seq";
const STRUCT_NAME: &'static str = "Struct";
const STRUCT_VARIANT_NAME: &'static str = "StructVariant";
const TUPLE_NAME: &'static str = "Tuple";
const TUPLE_STRUCT_NAME: &'static str = "TupleStruct";
const TUPLE_VARIANT_NAME: &'static str = "TupleVariant";
const UNIT_NAME: &'static str = "Unit";
const UNIT_STRUCT_NAME: &'static str = "UnitStruct";
const UNIT_VARIANT_NAME: &'static str = "UnitVariant";

/// DatalogExtractorBackend impl that stores facts in vectors.
/// Note that this backend interns strings, so tables store a string's
/// [SymbolId] instead of the string itself.
/// 
/// Note that this backend does **not** support extraction of
/// floating point values, and will return a
/// [UnextractableData][DatalogExtractionError::UnextractableData] error if
/// the input contains such values.
pub struct Backend {
    cur_symbol_id: SymbolId,

    /// Intern table for strings.
    pub symbol_table: HashMap<String, SymbolId>,

    /// Columns: (elem, elem type)
    pub type_table: Vec<(ElemId, SymbolId)>,

    /// Stores values of boolean elements.
    /// Columns: (elem, value)
    pub bool_table: Vec<(ElemId, bool)>,

    /// Stores values of number elements.
    /// Columns: (elem, value)
    pub number_table: Vec<(ElemId, isize)>,

    /// Stores values of string elements.
    /// Columns: (elem, symbol)
    pub string_table: Vec<(ElemId, SymbolId)>,

    /// Stores map entry facts.
    /// Columns: (elem, key, value)
    pub map_table: Vec<(ElemId, ElemId, ElemId)>,

    /// Stores type names of structs.
    /// Columns: (elem, struct name)
    pub struct_type_table: Vec<(ElemId, SymbolId)>,

    /// Stores struct field facts.
    /// Columns: (elem, field name, value elem)
    pub struct_table: Vec<(ElemId, SymbolId, ElemId)>,

    /// Stores sequence entry facts.
    /// Columns: (elem, index, value)
    pub seq_table: Vec<(ElemId, usize, ElemId)>,

    /// Stores type and variant names of variant elements.
    /// (elem, enum name, variant name)
    pub variant_type_table: Vec<(ElemId, SymbolId, SymbolId)>,

    /// Stores tuple entry facts.
    /// Columns: (elem, index, value)
    pub tuple_table: Vec<(ElemId, usize, ElemId)>,
}

impl Default for Backend {
    fn default() -> Self {
        let mut backend = 
            Self {
                cur_symbol_id: SymbolId(1),
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
                tuple_table: Default::default()
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
            for (elem, key, val) in self.map_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", elem.0, key.0, val.0);
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
            for (elem, field, val) in self.struct_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", elem.0, field.0, val.0);
            }
            println!();
        }

        if !self.seq_table.is_empty() {
            println!("{:^51}", "Seq Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "Elem Id", "Index", "Value");
            println!("---------------------------------------------------");
            for (elem, index, val) in self.seq_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", elem.0, index, val.0);
            }
            println!();
        }

        if !self.variant_type_table.is_empty() {
            println!("{:^51}", "Variant Type Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "Elem Id", "Enum Type", "Variant Name");
            println!("---------------------------------------------------");
            for (elem, enum_type, variant_name) in self.variant_type_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", elem.0, enum_type.0, variant_name.0);
            }
            println!();
        }

        if !self.tuple_table.is_empty() {
            println!("{:^51}", "Tuple Table");
            println!("---------------------------------------------------");
            println!("{:<15} | {:<15} | {:<15}", "Elem Id", "Index", "Value");
            println!("---------------------------------------------------");
            for (elem, index, val) in self.tuple_table.iter() {
                println!("{:<15} | {:<15?} | {:<15?}", elem.0, index, val.0);
            }
            println!();
        }
    }
}

impl<'a> DatalogExtractorBackend for &'a mut Backend {
    fn add_elem(&mut self, elem: ElemId, elem_type: ElemType) -> Result<()> {
        let type_name: &str;

        match elem_type {
            ElemType::Bool => {
                type_name = "Bool";
            }

            ElemType::I8 | ElemType::I16 | ElemType::I32 | ElemType::I64 |
            ElemType::U8 | ElemType::U16 | ElemType::U32 | ElemType::U64 => {
                type_name = "Number";
            },

            ElemType::Char | ElemType::Str => {
                type_name = "Str";
            }

            ElemType::F32 | ElemType::F64 | ElemType::Bytes => {
                return Result::Err(DatalogExtractionError::UnextractableData);
            }

            ElemType::Map => {
                type_name = "Map";
            }

            ElemType::Seq => {
                type_name = "Seq";
            }

            ElemType::Struct => {
                type_name = "Struct";
            }

            ElemType::StructVariant => {
                type_name = "StructVariant";
            }

            ElemType::Tuple => {
                type_name = "Tuple";
            }

            ElemType::TupleStruct | ElemType::NewtypeStruct => {
                type_name = "TupleStruct";
            }

            ElemType::TupleVariant | ElemType::NewtypeVariant => {
                type_name = "TupleVariant";
            }

            ElemType::Unit => {
                type_name = "Unit";
            }

            ElemType::UnitStruct => {
                type_name = "UnitStruct";
            }

            ElemType::UnitVariant => {
                type_name = "UnitVariant";
            }
        };

        let elem_type_sym = self.intern_string(type_name);
        self.type_table.push((elem, elem_type_sym));
        Result::Ok(())
    }

    fn add_bool(&mut self, elem: ElemId, value: bool) -> Result<()> {
        self.bool_table.push((elem, value));
        Result::Ok(())
    }

    fn add_i64(&mut self, elem: ElemId, value: i64) -> Result<()> {
        self.number_table.push((elem, value as isize));
        Result::Ok(())
    }

    fn add_u64(&mut self, elem: ElemId, value: u64) -> Result<()> {
        self.number_table.push((elem, value as isize));
        Result::Ok(())
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
