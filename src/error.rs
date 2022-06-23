use std::io;
use std::num::{ParseIntError, ParseFloatError};
use crate::types::ResetSettings;
use quick_error::quick_error;
use nvapi::{Status, error_message};

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Nvapi(err: Status) {
            from()
            display("NVAPI error: {}", error_message(*err).unwrap_or_else(|_| format!("{:?}", err)))
        }
        Io(err: io::Error) {
            from()
            cause(err)
            display("IO error: {}", err)
        }
        Json(err: serde_json::Error) {
            from()
            cause(err)
            display("JSON error: {}", err)
        }
        ParseInt(err: ParseIntError) {
            from()
            cause(err)
            display("{}", err)
        }
        ParseFloat(err: ParseFloatError) {
            from()
            cause(err)
            display("{}", err)
        }
        Str(err: &'static str) {
            from()
            display("{}", err)
        }
        ResetError { setting: ResetSettings, err: Status } {
            from(s: (ResetSettings, Status)) -> {
                setting: s.0,
                err: s.1
            }
            display("Reset {:?} failed: {}", setting, Error::from(err))
        }
    }
}

impl<'a> From<&'a Status> for Error {
    fn from(s: &'a Status) -> Self {
        s.clone().into()
    }
}
