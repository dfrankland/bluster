mod adapter;
mod advertisement;
mod common;
mod connection;
mod constants;
mod error;
mod gatt;

use dbus::Message;
use futures::prelude::*;
use std::sync::Arc;
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
    pub fn new(runtime: &mut Runtime) -> Result<Self, Error> {
        let connection = Arc::new(Connection::new(runtime)?);

        let adapter = Adapter::new(connection.clone())?;
        adapter.powered_on(true)?;

        let gatt = Gatt::new(connection.clone(), adapter.object_path.clone());
        let advertisement = Advertisement::new(connection.clone(), adapter.object_path.clone());

        Ok(Peripheral {
            adapter,
            gatt,
            advertisement,
        })
    }

    pub fn is_powered_on(self: &Self) -> Result<Box<impl Future<Item = (), Error = Error>>, Error> {
        Ok(self.adapter.is_powered_on())
    }

    pub fn start_advertising(
        self: &Self,
        name: &str,
        uuids: &[Uuid],
    ) -> Result<Box<impl Future<Item = Box<impl Stream<Item = Message, Error = ()>>>>, Error> {
        self.advertisement.add_name(name);
        self.advertisement.add_uuids(
            uuids
                .to_vec()
                .iter()
                .map(|uuid| uuid.to_string())
                .collect::<Vec<String>>(),
        );

        let advertisement = self.advertisement.clone();
        let registration = self
            .gatt
            .register()?
            .map_err(|_| ())
            .and_then(move |stream| {
                advertisement
                    .register()
                    .unwrap()
                    .and_then(move |_| Ok(stream))
                    .map_err(|_| ())
            });

        Ok(Box::new(registration))
    }

    pub fn stop_advertising(self: &Self) -> Result<Box<impl Future>, Error> {
        let advertisement = self.advertisement.unregister()?;
        let gatt = self.gatt.unregister()?;
        Ok(Box::new(advertisement.and_then(move |_| Ok(gatt))))
    }

    pub fn is_advertising(self: &Self) -> Result<bool, Error> {
        self.advertisement.is_advertising()
    }

    pub fn add_service(self: &Self, service: &Service) -> Result<(), Error> {
        self.gatt.add_service(service)
    }
}
