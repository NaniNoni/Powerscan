use std::ffi::{CStr, c_void};

use bitflags::bitflags;

use crate::{
    SANE_Action, SANE_Constraint_Type, SANE_Handle, SANE_Status, SaneError,
    option_descriptor::{SaneOptionConstaint, SaneOptionDescriptor},
    parameters::Parameters,
    sane_cancel, sane_close, sane_control_option, sane_get_option_descriptor, sane_get_parameters,
    sane_read, sane_start,
};

/// <https://sane-project.gitlab.io/standard/api.html#scanner-handle-type>
pub struct Handle {
    pub raw: SANE_Handle,
}

// TODO: verify that this is safe
unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}

impl Handle {
    /// <https://sane-project.gitlab.io/standard/api.html#sane-get-option-descriptor>
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
                cap: descriptor.cap,
                constraint: match descriptor.constraint_type {
                    SANE_Constraint_Type::SANE_CONSTRAINT_NONE => None,
                    SANE_Constraint_Type::SANE_CONSTRAINT_RANGE => {
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
                        let word_list = descriptor.constraint.word_list;
                        let length = *word_list as usize;
                        // Add 1 because the slice starts after the length field
                        let slice = std::slice::from_raw_parts(word_list.add(1), length);
                        Some(SaneOptionConstaint::WordList(slice.to_vec()))
                    }
                    SANE_Constraint_Type::SANE_CONSTRAINT_STRING_LIST => {
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

    /// <https://sane-project.gitlab.io/standard/api.html#sane-control-option>
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

    /// <https://sane-project.gitlab.io/standard/api.html#sane-get-parameters>
    pub fn get_parameters(&self) -> Result<Parameters, SaneError> {
        unsafe {
            let mut parameters = Default::default();
            let status = sane_get_parameters(self.raw, &mut parameters);
            if status != SANE_Status::SANE_STATUS_GOOD {
                return Err(SaneError::InternalSANE { status });
            }

            Ok(Parameters::from(parameters))
        }
    }

    /// <https://sane-project.gitlab.io/standard/api.html#sane-start>
    pub fn start(&self) -> Result<(), SaneError> {
        unsafe {
            let status = sane_start(self.raw);
            if status != SANE_Status::SANE_STATUS_GOOD {
                return Err(SaneError::InternalSANE { status });
            }

            Ok(())
        }
    }

    /// <https://sane-project.gitlab.io/standard/api.html#sane-read>
    pub fn read(&self, maxlen: usize) -> Result<Vec<u8>, SaneError> {
        let mut buf = Vec::with_capacity(maxlen);

        unsafe {
            let mut len = 0;
            let status = sane_read(self.raw, buf.as_mut_ptr(), maxlen as i32, &mut len);
            if status != SANE_Status::SANE_STATUS_GOOD {
                return Err(SaneError::InternalSANE { status });
            }

            buf.set_len(len as usize);

            Ok(buf)
        }
    }

    pub fn cancel(&self) {
        unsafe {
            sane_cancel(self.raw);
        }
    }
}

bitflags! {
    #[derive(Debug)]
    /// A [`bitflags`] struct generated as a safe wrapper around `SANE_INFO_*` constants
    pub struct ControlOptionInfo: i32 {
        /// <https://sane-project.gitlab.io/standard/api.html?highlight=sane_info#c.SANE_INFO_INEXACT>
        const INEXACT = crate::SANE_INFO_INEXACT as i32;
        /// <https://sane-project.gitlab.io/standard/api.html?highlight=sane_info#c.SANE_INFO_RELOAD_OPTIONS>
        const RELOAD_OPTIONS = crate::SANE_INFO_RELOAD_OPTIONS as i32;
        /// <https://sane-project.gitlab.io/standard/api.html?highlight=sane_info#c.SANE_INFO_RELOAD_PARAMS>
        const RELOAD_PARAMS = crate::SANE_INFO_RELOAD_PARAMS as i32;
    }
}

impl Drop for Handle {
    /// <https://sane-project.gitlab.io/standard/api.html?highlight=sane_info#sane-close>
    fn drop(&mut self) {
        unsafe { sane_close(self.raw) }
    }
}

#[cfg(test)]
mod tests {
    use std::any::type_name;

    use serial_test::serial;

    use crate::{
        SANE_Frame, SANE_Status, Sane, SaneError, handle::ControlOptionInfo,
        parameters::Parameters, tests::TEST_DEVICE_NAME,
    };

    #[test]
    #[serial]
    fn sane_get_option_descriptor() -> Result<(), SaneError> {
        let sane = Sane::init()?;
        let handle = sane.open(&TEST_DEVICE_NAME)?;

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
        let sane = Sane::init()?;
        let handle = sane.open(&TEST_DEVICE_NAME)?;

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

    #[test]
    #[serial]
    fn sane_get_parameters() -> Result<(), SaneError> {
        let sane = Sane::init()?;
        let handle = sane.open(&TEST_DEVICE_NAME)?;
        let params = handle.get_parameters()?;
        println!("Parameters: {params:?}");

        assert_eq!(
            params,
            Parameters {
                format: SANE_Frame::SANE_FRAME_GRAY,
                last_frame: true,
                bytes_per_line: 157,
                pixels_per_line: 157,
                lines: 196,
                depth: 8
            }
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn sane_start() -> Result<(), SaneError> {
        let sane = Sane::init()?;
        let handle = sane.open(&TEST_DEVICE_NAME)?;
        handle.start()
    }

    #[test]
    #[serial]
    fn sane_read() -> Result<(), SaneError> {
        const CHUNK_SIZE: usize = 512;
        let sane = Sane::init()?;
        let handle = sane.open(&TEST_DEVICE_NAME)?;
        handle.start()?;

        let mut data = Vec::new();
        loop {
            match handle.read(CHUNK_SIZE) {
                Ok(chunk) => {
                    println!("Chunk: {chunk:?}");
                    data.extend_from_slice(&chunk);
                }
                Err(SaneError::InternalSANE { status }) => {
                    if status == SANE_Status::SANE_STATUS_EOF {
                        handle.cancel();
                        break;
                    }
                }
                Err(e) => return Err(e),
            }
        }

        println!("Scanning data: {data:?}");
        assert!(!data.is_empty(), "Expected non-empty scan data");

        Ok(())
    }
}
