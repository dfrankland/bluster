mod adapter;
mod advertisement;
mod common;
mod connection;
mod constants;
mod error;
mod gatt;

use futures::{future, prelude::*};
use std::{
    string::ToString,
    sync::{Arc, Mutex},
};
use tokio::runtime::current_thread::Runtime;
use uuid::Uuid;

use self::{adapter::Adapter, advertisement::Advertisement, connection::Connection, gatt::Gatt};
use crate::{gatt::service::Service, Error};

pub struct Peripheral {
    adapter: Adapter,
    gatt: Gatt,
    advertisement: Advertisement,
}

impl Peripheral {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(runtime: &Arc<Mutex<Runtime>>) -> Box<impl Future<Item = Self, Error = Error>> {
        let connection = match Connection::new(Arc::clone(&runtime)) {
            Ok(connection) => Arc::new(connection),
            Err(err) => return Box::new(future::Either::A(future::err(err))),
        };

        let peripheral = Adapter::new(connection.clone())
            .and_then(|adapter| {
                let adapter1 = adapter.clone();
                adapter.powered(true).and_then(move |_| Ok(adapter1))
            })
            .and_then(move |adapter| {
                let gatt = Gatt::new(connection.clone(), adapter.object_path.clone());
                let advertisement = Advertisement::new(connection, adapter.object_path.clone());

                Ok(Peripheral {
                    adapter,
                    gatt,
                    advertisement,
                })
            });

        Box::new(future::Either::B(peripheral))
    }

    pub fn is_powered(self: &Self) -> Box<impl Future<Item = bool, Error = Error>> {
        self.adapter.is_powered()
    }

    pub fn start_advertising(
        self: &Self,
        name: &str,
        uuids: &[Uuid],
    ) -> Box<impl Future<Item = Box<impl Stream<Item = (), Error = Error>>, Error = Error>> {
        self.advertisement.add_name(name);
        self.advertisement.add_uuids(
            uuids
                .to_vec()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        );

        let advertisement = self.advertisement.register();
        let gatt = self.gatt.register();
        let registration = gatt.join(advertisement).map(|(stream, ..)| stream);

        Box::new(registration)
    }

    pub fn stop_advertising(self: &Self) -> Box<impl Future<Item = (), Error = Error>> {
        let advertisement = self.advertisement.unregister();
        let gatt = self.gatt.unregister();
        Box::new(advertisement.join(gatt).map(|_| ()))
    }

    pub fn is_advertising(self: &Self) -> Box<impl Future<Item = bool, Error = Error>> {
        Box::new(future::ok(self.advertisement.is_advertising()))
    }

    pub fn add_service(self: &Self, service: &Service) -> Result<(), Error> {
        self.gatt.add_service(service)
    }
}
