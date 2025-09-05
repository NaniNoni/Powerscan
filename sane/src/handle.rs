use std::ffi::{CStr, c_void};

use bitflags::bitflags;

use crate::{
    SANE_Action, SANE_Constraint_Type, SANE_Handle, SANE_Status, SaneError,
    option_descriptor::{SaneOptionConstaint, SaneOptionDescriptor},
    sane_close, sane_control_option, sane_get_option_descriptor,
};

// https://sane-project.gitlab.io/standard/api.html#scanner-handle-type
pub struct Handle {
    pub raw: SANE_Handle,
}

impl Handle {
    pub fn get_option_descriptor(&self, n: i32) -> Result<Option<SaneOptionDescriptor>, SaneError> {
        unsafe {
            let descriptor_ptr = sane_get_option_descriptor(self.raw, n);
            if descriptor_ptr.is_null() {
                return Ok(None);
            }

            let descriptor = *descriptor_ptr;

            Ok(Some(SaneOptionDescriptor {
                name: CStr::from_ptr(descriptor.name).to_str()?.to_owned(),
                title: CStr::from_ptr(descriptor.title).to_str()?.to_owned(),
                desc: CStr::from_ptr(descriptor.desc).to_str()?.to_owned(),
                type_: descriptor.type_,
                unit: descriptor.unit,
                size: descriptor.size,
                cap: descriptor.cap,
                constraint: match descriptor.constraint_type {
                    SANE_Constraint_Type::SANE_CONSTRAINT_NONE => None,
                    SANE_Constraint_Type::SANE_CONSTRAINT_RANGE => {
                        println!("RANGE");
                        let range = *descriptor.constraint.range;
                        Some(SaneOptionConstaint::Range {
                            min: range.min,
                            max: range.max,
                            quant: range.quant,
                        })
                    }
                    // The first element in that list is an integer (SANE_Int) that specifies the length of the list (not counting the length itself).
                    // The remaining elements in the list are interpreted according to the type of the option value (SANE_TYPE_INT or SANE_TYPE_FIXED).
                    SANE_Constraint_Type::SANE_CONSTRAINT_WORD_LIST => {
                        println!("WORD_LIST");
                        let word_list = descriptor.constraint.word_list;
                        let length = *word_list as usize;
                        // Add 1 because the slice starts after the length field
                        let slice = std::slice::from_raw_parts(word_list.add(1), length);
                        Some(SaneOptionConstaint::WordList(slice.to_vec()))
                    }
                    SANE_Constraint_Type::SANE_CONSTRAINT_STRING_LIST => {
                        println!("STRING_LIST");
                        let string_list = descriptor.constraint.string_list;
                        let mut strings = Vec::new();
                        let mut i = 0;
                        loop {
                            let ptr = *string_list.add(i);
                            // End of string array
                            if ptr.is_null() {
                                break;
                            }

                            let s = CStr::from_ptr(ptr).to_str()?.to_owned();
                            strings.push(s);
                            i += 1;
                        }

                        Some(SaneOptionConstaint::StringList(strings))
                    }
                },
            }))
        }
    }

    pub fn control_option<T>(
        &self,
        option: i32,
        action: SANE_Action,
        value: &mut T,
    ) -> Result<ControlOptionInfo, SaneError>
    where
        T: Clone,
    {
        unsafe {
            let mut info = 0;
            let status = sane_control_option(
                self.raw,
                option,
                action,
                value as *mut T as *mut c_void,
                &mut info,
            );

            if status != SANE_Status::SANE_STATUS_GOOD {
                return Err(SaneError::InternalSANE { status });
            }

            Ok(ControlOptionInfo::from_bits_truncate(info))
        }
    }
}

bitflags! {
    #[derive(Debug)]
    /// A [`bitflags`] struct generated as a safe wrapper around `SANE_INFO_*` constants
    pub struct ControlOptionInfo: i32 {
        const INEXACT = crate::SANE_INFO_INEXACT as i32;
        const RELOAD_OPTIONS = crate::SANE_INFO_RELOAD_OPTIONS as i32;
        const RELOAD_PARAMS = crate::SANE_INFO_RELOAD_PARAMS as i32;
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { sane_close(self.raw) }
    }
}

#[cfg(test)]
mod tests {
    use std::any::type_name;

    use serial_test::serial;

    use crate::{Sane, SaneError, handle::ControlOptionInfo};

    #[test]
    #[serial]
    fn sane_get_option_descriptor() -> Result<(), SaneError> {
        let sane = Sane::init(0)?;
        let handle = sane.open("test:0")?;

        let mut i = 0;
        loop {
            let option_descriptor = handle.get_option_descriptor(i)?;

            if let Some(opt) = option_descriptor {
                println!("Option: {:?}", opt);
            } else {
                // No more available option descriptors
                break;
            }

            i += 1;
        }

        // At least the first option must be valid
        assert_ne!(i, 0);

        Ok(())
    }

    #[test]
    #[serial]
    fn sane_control_option() -> Result<(), SaneError> {
        let sane = Sane::init(0)?;
        let handle = sane.open("test:0")?;

        let mut value = 0;
        let info =
            handle.control_option(0, crate::SANE_Action::SANE_ACTION_GET_VALUE, &mut value)?;

        assert!(
            value > 0,
            "Expected positive number of options, got {value}",
        );
        assert!(
            info.is_empty(),
            "Expected valid {}, got {info:?}",
            type_name::<ControlOptionInfo>()
        );

        Ok(())
    }
}
