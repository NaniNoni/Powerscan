include!(concat!(env!("OUT_DIR"), "/sane.rs"));

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

/// "Safe" SANE interface wrapper. All functions correspond to their respective C `sane_` function.
pub struct Sane {}

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
    pub fn init(version_code: i32) -> Result<Self, SaneError> {
        unsafe {
            let mut version_code = version_code;
            let status = sane_init(&mut version_code, None);
            if status != SANE_Status::SANE_STATUS_GOOD {
                return Err(SaneError::InternalSANE { status });
            }

            Ok(Self {})
        }
    }

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
        let _sane = Sane::init(0)?;
        Ok(())
    }

    #[test]
    #[serial]
    fn sane_get_devices() -> Result<(), SaneError> {
        let sane = Sane::init(0)?;
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
        let sane = Sane::init(0)?;
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
