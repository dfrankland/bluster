use std::fmt;
use std::sync::Arc;

use dbus::{nonblock::SyncConnection, Path};

use super::constants::{BLUEZ_DBUS_TIMEOUT, BLUEZ_SERVICE_NAME};
use crate::Error;

pub struct Connection {
    pub default: Arc<SyncConnection>,
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Connection")
    }
}

impl<'a> Connection {
    pub fn new() -> Result<Self, Error> {
        let (resource, default) = dbus_tokio::connection::new_system_sync()?;
        tokio::spawn(async {
            let err = resource.await;
            panic!("Lost connection to D-Bus: {}", err);
        });

        Ok(Connection { default })
    }

    pub fn get_bluez_proxy(&'a self, path: &'a Path) -> dbus::nonblock::Proxy<&'a SyncConnection> {
        dbus::nonblock::Proxy::new(BLUEZ_SERVICE_NAME, path, BLUEZ_DBUS_TIMEOUT, &self.default)
    }
}
