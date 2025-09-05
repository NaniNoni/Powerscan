use crate::{SANE_Unit, SANE_Value_Type};

/// Option descriptors are at the same time the most intricate and powerful type in the SANE standard.
/// Options are used to control virtually all aspects of device operation.
/// Much of the power of the SANE API stems from the fact that most device controls are completely described by their respective option descriptor.
/// Thus, a frontend can control a scanner abstractly, without requiring knowledge as to what the purpose of any given option is.
/// Conversely, a scanner can describe its controls without requiring knowledge of how the frontend operates.
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
    /// This constraint is applicable to integer and fixed-point valued options only.
    /// It constrains the option value to a possibly quantized range of numbers.
    Range { min: i32, max: i32, quant: i32 },
    /// This constraint is applicable to string-valued options only.
    /// It constrains the option value to a list of strings.
    StringList(Vec<String>),
    /// This constraint is applicable to integer and fixed-point valued options only.
    /// It constrains the option value to a list of numeric values.
    /// The first element in that list is an integer ([`i32`]) that specifies the length of the list (not counting the length itself).
    /// The remaining elements in the list are interpreted according to the type of the option value (SANE_TYPE_INT or SANE_TYPE_FIXED).
    WordList(Vec<i32>),
}
