use crate::{SANE_Unit, SANE_Value_Type};

/// <https://sane-project.gitlab.io/standard/api.html#option-descriptor-type>
#[derive(Debug)]
pub struct SaneOptionDescriptor {
    /// <https://sane-project.gitlab.io/standard/api.html#option-name>
    pub name: String,
    /// <https://sane-project.gitlab.io/standard/api.html#option-title>
    pub title: String,
    /// <https://sane-project.gitlab.io/standard/api.html#option-description>
    pub desc: String,
    /// <https://sane-project.gitlab.io/standard/api.html#option-description>
    pub type_: SANE_Value_Type,
    /// <https://sane-project.gitlab.io/standard/api.html#option-value-unit>
    pub unit: SANE_Unit,
    /// <https://sane-project.gitlab.io/standard/api.html#option-capabilities>
    pub cap: i32,
    /// There is no need to store the constraint type or size as Rust enums are type-safe, unlike C unions.
    /// <https://sane-project.gitlab.io/standard/api.html#option-value-constraints>
    pub constraint: Option<SaneOptionConstaint>,
}

#[derive(Debug)]
pub enum SaneOptionConstaint {
    /// <https://sane-project.gitlab.io/standard/api.html#c.SANE_CONSTRAINT_RANGE>
    Range { min: i32, max: i32, quant: i32 },
    /// <https://sane-project.gitlab.io/standard/api.html#c.SANE_CONSTRAINT_WORD_LIST>
    WordList(Vec<i32>),
    /// <https://sane-project.gitlab.io/standard/api.html#c.SANE_CONSTRAINT_STRING_LIST>
    StringList(Vec<String>),
}
