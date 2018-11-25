use crate::{Error, ErrorType};
use dbus::Error as DbusError;
use std::io::Error as IoError;

impl From<DbusError> for Error {
    fn from(dbus_error: DbusError) -> Error {
        Error::new(
            dbus_error.name().unwrap_or(""),
            dbus_error.message().unwrap_or(""),
            ErrorType::Bluez,
        )
    }
}

impl From<IoError> for Error {
    fn from(io_error: IoError) -> Error {
        Error::new(
            format!("std::io::Error: {:?}", io_error.kind()),
            format!("{:?}", io_error),
            ErrorType::Bluez,
        )
    }
}
