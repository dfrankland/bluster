use dbus::{BusType, Connection as SyncConnection};
use dbus_tokio::AConnection as AsyncConnection;
use std::rc::Rc;
use tokio::{reactor::Handle, runtime::current_thread::Runtime};

use crate::Error;

#[derive(Debug)]
pub struct Connection {
    pub fallback: Rc<SyncConnection>,
    pub default: AsyncConnection,
}

impl Connection {
    pub fn new(mut runtime: &mut Runtime) -> Result<Self, Error> {
        let connection = Rc::new(SyncConnection::get_private(BusType::System)?);
        let aconnection =
            AsyncConnection::new(connection.clone(), Handle::current(), &mut runtime)?;

        Ok(Connection {
            fallback: connection,
            default: aconnection,
        })
    }
}
