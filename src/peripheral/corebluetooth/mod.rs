mod characteristic_flags;
mod constants;
mod error;
mod events;
mod ffi;
mod into_bool;
mod into_cbuuid;
mod peripheral_manager;

use futures01::{future, prelude::*};
use std::{
    sync::{Arc, Mutex},
    time,
};
use tokio::{runtime::current_thread::Runtime, timer};
use uuid::Uuid;

use self::peripheral_manager::PeripheralManager;
use crate::{gatt::service::Service, Error};

pub struct Peripheral {
    peripheral_manager: PeripheralManager,
}

impl Peripheral {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_runtime: &Arc<Mutex<Runtime>>) -> Box<impl Future<Item = Self, Error = Error>> {
        Box::new(future::ok(Peripheral {
            peripheral_manager: PeripheralManager::new(),
        }))
    }

    pub fn is_powered(self: &Self) -> Box<impl Future<Item = bool, Error = Error>> {
        Box::new(future::ok(self.peripheral_manager.is_powered()))
    }

    pub fn start_advertising(
        self: &Self,
        name: &str,
        uuids: &[Uuid],
    ) -> Box<impl Future<Item = Box<impl Stream<Item = (), Error = Error>>, Error = Error>> {
        self.peripheral_manager.start_advertising(name, uuids);

        // TODO: Create an actual stream
        let read_stream = timer::Interval::new_interval(time::Duration::from_secs(1))
            .map(|_| ())
            .map_err(|_| Error::from(()));
        Box::new(future::ok(Box::new(read_stream)))
    }

    pub fn stop_advertising(self: &Self) -> Box<impl Future<Item = (), Error = Error>> {
        self.peripheral_manager.stop_advertising();
        Box::new(future::ok(()))
    }

    pub fn is_advertising(self: &Self) -> Box<impl Future<Item = bool, Error = Error>> {
        Box::new(future::ok(self.peripheral_manager.is_advertising()))
    }

    pub fn add_service(self: &Self, service: &Service) -> Result<(), Error> {
        self.peripheral_manager.add_service(service);
        Ok(())
    }
}
