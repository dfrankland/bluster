use dbus::{BusType, Connection as SyncConnection};
use dbus_tokio::AConnection as AsyncConnection;
use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};
use tokio::{reactor, runtime::current_thread};

use crate::Error;

#[derive(Debug)]
pub struct Connection {
    pub fallback: Rc<SyncConnection>,
    pub default: AsyncConnection,
    pub runtime: Arc<Mutex<current_thread::Runtime>>,
}

impl Connection {
    pub fn new(runtime: Arc<Mutex<current_thread::Runtime>>) -> Result<Self, Error> {
        let fallback = Rc::new(SyncConnection::get_private(BusType::System)?);
        let default = AsyncConnection::new(
            fallback.clone(),
            reactor::Handle::default(),
            &mut runtime.lock().unwrap(),
        )?;

        Ok(Connection {
            fallback,
            default,
            runtime,
        })
    }
}
