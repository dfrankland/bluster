use crate::{Error, ErrorType};

impl From<()> for Error {
    fn from(_: ()) -> Error {
        Error::new("no name", "no description", ErrorType::CoreBluetooth)
    }
}
