use std::ffi::CStr;

include!(concat!(env!("OUT_DIR"), "/sane.rs"));

/// "Safe" SANE interface wrapper. All functions correspond to their respective C `sane_` function.
pub struct Sane {
    pub version_code: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Device {
    pub name: String,
    pub vendor: String,
    pub model: String,
    pub type_: String,
}

#[derive(Debug)]
pub struct SaneError(pub SANE_Status);

impl Sane {
    pub fn init(version_code: i32) -> Result<Self, SaneError> {
        unsafe {
            let mut version_code = version_code;
            let status = sane_init(&mut version_code, None);
            if status != SANE_Status::SANE_STATUS_GOOD {
                return Err(SaneError(status));
            }

            Ok(Self { version_code })
        }
    }

    pub fn get_devices(&self) -> Result<Vec<Device>, SaneError> {
        unsafe {
            let mut device_list: *mut *const SANE_Device = std::ptr::null_mut();
            let status = sane_get_devices(&mut device_list, 0);
            if status != SANE_Status::SANE_STATUS_GOOD {
                return Err(SaneError(status));
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
                    let vendor = CStr::from_ptr(device.vendor).to_string_lossy().into_owned();
                    let model = CStr::from_ptr(device.model).to_string_lossy().into_owned();
                    let type_ = CStr::from_ptr(device.type_).to_string_lossy().into_owned();

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
}
