use dbus::{BusType, Connection as SyncConnection};
use dbus_tokio::AConnection as AsyncConnection;
use std::rc::Rc;
use tokio::{reactor, runtime::current_thread};

use crate::Error;

#[derive(Debug)]
pub struct Connection {
    pub fallback: Rc<SyncConnection>,
    pub default: AsyncConnection,
}

impl Connection {
    pub fn new(mut runtime: &mut current_thread::Runtime) -> Result<Self, Error> {
        let fallback = Rc::new(SyncConnection::get_private(BusType::System)?);
        let default =
            AsyncConnection::new(fallback.clone(), reactor::Handle::current(), &mut runtime)?;

        Ok(Connection { fallback, default })
    }
}
