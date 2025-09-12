// sane/src/lib.rs
#[allow(non_camel_case_types, unused)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/sane.rs"));
}

// TODO: remove this by implementing a proper exported Rust type
pub use bindings::SANE_Status;
use bindings::*;

mod device;
mod handle;
mod option_descriptor;
mod parameters;

use std::ffi::{CStr, CString};
use thiserror::Error;

pub use crate::device::Device;
pub use crate::device::DeviceType;
pub use crate::device::DeviceVendor;
pub use crate::handle::Handle;

/// "Safe" SANE interface wrapper
/// This type is [`Clone`], as it barely stores any state, and mostly exists as a container for the
/// C SANE functions.
#[derive(Debug, Clone, Default)]
pub struct Sane {
    _version_code: i32,
}

/// Error type returned by all [`Sane`] functions that can fail
#[derive(Debug, Error)]
pub enum SaneError {
    /// Any [`SANE_Status`] which doesn't equal [`SANE_Status::SANE_STATUS_GOOD`]
    #[error("internal SANE error, status: {status:?}")]
    InternalSANE { status: SANE_Status },

    /// UTF8 Error most likely from converting C string to Rust ones
    #[error("invalid UTF8: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("ffi nul error: {0}")]
    FfiError(#[from] std::ffi::NulError),
}

impl Sane {
    /// <https://sane-project.gitlab.io/standard/api.html#sane-init>
    // TODO: research authorization
    pub fn init() -> Result<Self, SaneError> {
        unsafe {
            let mut version_code = 0;
            let status = sane_init(&mut version_code, None);
            if status != SANE_Status::SANE_STATUS_GOOD {
                return Err(SaneError::InternalSANE { status });
            }

            Ok(Self {
                _version_code: version_code,
            })
        }
    }

    /// <https://sane-project.gitlab.io/standard/api.html#sane-get-devices>
    // TODO: research local only vs remote
    pub fn get_devices(&self) -> Result<Vec<Device>, SaneError> {
        unsafe {
            let mut device_list: *mut *const SANE_Device = std::ptr::null_mut();
            let status = sane_get_devices(&mut device_list, 0);
            if status != SANE_Status::SANE_STATUS_GOOD {
                return Err(SaneError::InternalSANE { status });
            }

            let mut devices = Vec::new();

            let mut i = 0;
            if !device_list.is_null() {
                loop {
                    let device_ptr = *device_list.add(i);
                    // The devices array is null terminated
                    if device_ptr.is_null() {
                        break;
                    }

                    let device = &*device_ptr;

                    let name = CStr::from_ptr(device.name).to_str()?.to_owned();
                    let vendor = DeviceVendor::try_from(CStr::from_ptr(device.vendor))?;
                    let model = CStr::from_ptr(device.model).to_str()?.to_owned();
                    let type_ = DeviceType::try_from(CStr::from_ptr(device.type_))?;

                    devices.push(Device {
                        name,
                        vendor,
                        model,
                        type_,
                    });
                    i += 1;
                }
            }

            Ok(devices)
        }
    }

    /// <https://sane-project.gitlab.io/standard/api.html#sane-open>
    pub fn open(&self, device_name: &str) -> Result<Handle, SaneError> {
        let name = CString::new(device_name)?;
        unsafe {
            let mut raw = Default::default();
            let status = sane_open(name.as_ptr(), &mut raw);
            if status != SANE_Status::SANE_STATUS_GOOD {
                return Err(SaneError::InternalSANE { status });
            }
            Ok(Handle { raw })
        }
    }
}

impl Drop for Sane {
    /// <https://sane-project.gitlab.io/standard/api.html#sane-exit>
    fn drop(&mut self) {
        unsafe { sane_exit() }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use serial_test::serial;

    use super::*;

    pub static TEST_DEVICE_NAME: LazyLock<String> = LazyLock::new(|| {
        std::env::var("POWERSCAN_SANE_TEST_DEVICE").unwrap_or_else(|_| "test:0".to_owned())
    });

    #[test]
    #[serial]
    fn sane_init() -> Result<(), SaneError> {
        let _sane = Sane::init()?;
        Ok(())
    }

    #[test]
    #[serial]
    fn sane_get_devices() -> Result<(), SaneError> {
        let sane = Sane::init()?;
        let devices = sane.get_devices()?;

        assert!(
            !devices.is_empty(),
            "Expected at least one device from the test backend"
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn open_test_device() -> Result<(), SaneError> {
        let sane = Sane::init()?;
        let handle = sane.open(&TEST_DEVICE_NAME)?;
        assert!(
            !handle.raw.is_null(),
            "Expected a valid handle for device {}",
            *TEST_DEVICE_NAME
        );

        Ok(())
    }
}
