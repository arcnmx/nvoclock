use std::io;
use std::num::{ParseIntError, ParseFloatError};
use crate::types::ResetSettings;
use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Nvapi(err: nvapi::Error) {
            from()
            source(err)
            display("NVAPI error: {}", err)
        }
        VfpUnsupported {
            display("VFP unsupported")
        }
        DeviceNotFound {
            display("no matching device found")
        }
        Io(err: io::Error) {
            from()
            source(err)
            display("IO error: {}", err)
        }
        Json(err: serde_json::Error) {
            from()
            source(err)
            display("JSON error: {}", err)
        }
        ParseInt(err: ParseIntError) {
            from()
            source(err)
            display("{}", err)
        }
        ParseFloat(err: ParseFloatError) {
            from()
            source(err)
            display("{}", err)
        }
        Str(err: &'static str) {
            from()
            display("{}", err)
        }
        ResetError { setting: ResetSettings, err: nvapi::Error } {
            from(s: (ResetSettings, nvapi::Error)) -> {
                setting: s.0,
                err: s.1
            }
            display("Reset {:?} failed: {}", setting, err)
        }
    }
}

impl From<nvapi::NvapiError> for Error {
    fn from(e: nvapi::NvapiError) -> Self {
        Self::from(nvapi::Error::from(e))
    }
}
