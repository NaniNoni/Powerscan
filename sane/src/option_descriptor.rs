use crate::{SANE_Unit, SANE_Value_Type};

#[derive(Debug)]
pub struct SaneOptionDescriptor {
    pub name: String,
    pub title: String,
    pub desc: String,
    pub type_: SANE_Value_Type,
    pub unit: SANE_Unit,
    pub size: i32,
    pub cap: i32,
    // There is no need to store the constraint type,
    // as Rust enums are type-safe, unlike C unions.
    pub constraint: Option<SaneOptionConstaint>,
}

#[derive(Debug)]
pub enum SaneOptionConstaint {
    StringList(Vec<String>),
    WordList(Vec<i32>),
    Range { min: i32, max: i32, quant: i32 },
}
