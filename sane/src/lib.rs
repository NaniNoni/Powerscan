// sane/src/lib.rs
#[allow(non_camel_case_types)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/sane.rs"));
}

pub use bindings::*;

mod device;
mod handle;
mod option_descriptor;

use crate::{
    device::{DeviceType, DeviceVendor},
    handle::Handle,
};
use std::ffi::{CStr, CString};
use thiserror::Error;

pub use crate::device::Device;

/// "Safe" SANE interface wrapper
pub struct Sane {
    _version_code: i32,
}

#[derive(Debug, Error)]
pub enum SaneError {
    #[error("internal SANE error, status: {status:?}")]
    InternalSANE { status: SANE_Status },

    #[error("invalid UTF8 in device string: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("ffi nul error: {0}")]
    FfiError(#[from] std::ffi::NulError),
}

impl Sane {
    /// This function must be called before any other SANE function can be called.
    /// The behavior of a SANE backend is undefined if this function is not called first or if the status code returned by [`Sane::init`] is different from [`Ok`].
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

    /// This function can be used to query the list of devices that are available.
    /// The returned list is guaranteed to remain unchanged and valid until (a) another call to this function is performed or (b) a call to [`Sane::drop`] is performed.
    /// This function can be called repeatedly to detect when new devices become available.
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

                    let name = CStr::from_ptr(device.name).to_string_lossy().into_owned();
                    let vendor = DeviceVendor::try_from(CStr::from_ptr(device.vendor))?;
                    let model = CStr::from_ptr(device.model).to_string_lossy().into_owned();
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

    /// This function is used to establish a connection to a particular device.
    /// The name of the device to be opened is passed in argument name.
    /// If the call completes successfully, a handle for the device is returned.
    /// As a special case, specifying a zero-length string as the device requests opening the first available device (if there is such a device).
    pub fn open(&self, device_name: &str) -> Result<Handle, SaneError> {
        let name = CString::new(device_name)?;
        unsafe {
            let mut raw: SANE_Handle = std::ptr::null_mut();
            let status = sane_open(name.as_ptr(), &mut raw);
            if status != SANE_Status::SANE_STATUS_GOOD {
                return Err(SaneError::InternalSANE { status });
            }
            Ok(Handle { raw })
        }
    }
}

impl Drop for Sane {
    /// This function must be called to terminate use of a backend.
    /// The function will first close all device handles that still might be open (it is recommended to close device handles explicitly through a call to [`Handle::drop`], but backends are required to release all resources upon a call to this function).
    /// After this function returns, no function other than [`Sane::init`] may be called (regardless of the status value returned by [`Sane::drop`]. Neglecting to call this function may result in some resources not being released properly.
    fn drop(&mut self) {
        unsafe { sane_exit() }
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

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

        // With the test backend, device names typically look like "test:0"
        assert!(
            devices.iter().any(|d| d.name.starts_with("test:")),
            "Expected a device from the test backend, got: {:?}",
            devices
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn open_test_device() -> Result<(), SaneError> {
        let sane = Sane::init()?;
        let devices = sane.get_devices()?;
        let first = devices
            .first()
            .expect("Test backend should expose at least one device");

        // Try to open the first device
        let handle = sane.open(&first.name)?;
        assert!(
            !handle.raw.is_null(),
            "Expected a valid handle for device {}",
            first.name
        );

        Ok(())
    }
}
