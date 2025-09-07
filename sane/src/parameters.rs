use crate::{SANE_Frame, SANE_Parameters};

/// A wrapper around [`SANE_Parameters`]
#[derive(Debug, PartialEq, Eq)]
pub struct Parameters {
    pub format: SANE_Frame,
    pub last_frame: bool,
    pub bytes_per_line: i32,
    pub pixels_per_line: i32,
    pub lines: i32,
    pub depth: i32,
}

impl From<SANE_Parameters> for Parameters {
    fn from(value: SANE_Parameters) -> Self {
        Self {
            format: value.format,
            last_frame: value.last_frame == 1,
            bytes_per_line: value.bytes_per_line,
            pixels_per_line: value.pixels_per_line,
            lines: value.lines,
            depth: value.depth,
        }
    }
}
