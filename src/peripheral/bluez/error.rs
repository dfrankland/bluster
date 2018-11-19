use crate::{Error, ErrorType};
use dbus::Error as DbusError;

impl From<DbusError> for Error {
    fn from(dbus_error: DbusError) -> Error {
        Error::new(
            dbus_error.name().unwrap_or(""),
            dbus_error.message().unwrap_or(""),
            ErrorType::Bluez,
        )
    }
}
