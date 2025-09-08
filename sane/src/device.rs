use std::ffi::CStr;

/// <https://sane-project.gitlab.io/standard/api.html#device-descriptor-type>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub name: String,
    pub vendor: DeviceVendor,
    pub model: String,
    pub type_: DeviceType,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceVendor {
    AGFA,
    Abaton,
    Acer,
    Apple,
    Artec,
    Avision,
    CANON,
    Connectix,
    Epson,
    Fujitsu,
    HewlettPackard,
    IBM,
    Kodak,
    Lexmark,
    Logitech,
    Microtek,
    Minolta,
    Mitsubishi,
    Mustek,
    NEC,
    Nikon,
    Plustek,
    Polaroid,
    Relisys,
    Ricoh,
    Sharp,
    Siemens,
    Tamarack,
    UMAX,
    Noname,
}

impl TryFrom<&CStr> for DeviceVendor {
    type Error = std::str::Utf8Error;

    fn try_from(value: &CStr) -> Result<Self, Self::Error> {
        Ok(match value.to_str()? {
            "AGFA" => Self::AGFA,
            "Abaton" => Self::Abaton,
            "Acer" => Self::Acer,
            "Apple" => Self::Apple,
            "Artec" => Self::Avision,
            "CANON" => Self::CANON,
            "Connectix" => Self::Epson,
            "Fujitsu" => Self::Fujitsu,
            "Hewlett-Packard" => Self::HewlettPackard,
            "IBM" => Self::IBM,
            "Kodak" => Self::Kodak,
            "Lexmark" => Self::Lexmark,
            "Logitech" => Self::Logitech,
            "Microtek" => Self::Microtek,
            "Minolta" => Self::Minolta,
            "Mitsubishi" => Self::Mitsubishi,
            "Mustek" => Self::Mustek,
            "NEC" => Self::NEC,
            "Nikon" => Self::Nikon,
            "Plustek" => Self::Plustek,
            "Polaroid" => Self::Polaroid,
            "Relisys" => Self::Relisys,
            "Ricoh" => Self::Ricoh,
            "Sharp" => Self::Sharp,
            "Siemens" => Self::Siemens,
            "Tamarack" => Self::Tamarack,
            "UMAX" => Self::UMAX,
            "Noname" => Self::Noname,
            // TODO: consult this behaviour
            _ => Self::Noname,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceType {
    FilmScanner,
    FlatbedScanner,
    FrameGrabber,
    HandheldScanner,
    MultiFunctionPeripheral,
    SheetfedScanner,
    StillCamera,
    VideoCamera,
    VirtualDevice,
    Other(String),
}

impl TryFrom<&CStr> for DeviceType {
    type Error = std::str::Utf8Error;

    fn try_from(value: &CStr) -> Result<Self, Self::Error> {
        Ok(match value.to_str()? {
            "film scanner" => Self::FilmScanner,
            "flatbed scanner" => Self::FlatbedScanner,
            "frame grabber" => Self::FrameGrabber,
            "handheld scanner" => Self::HandheldScanner,
            "multi-function peripheral" => Self::MultiFunctionPeripheral,
            "sheetfed scanner" => Self::SheetfedScanner,
            "still camera" => Self::StillCamera,
            "video camera" => Self::VideoCamera,
            "virtual device" => Self::VirtualDevice,
            other => Self::Other(other.to_owned()),
        })
    }
}
