mod adapter;
mod advertisement;
mod common;
mod connection;
mod constants;
mod error;
mod gatt;

use dbus::ConnectionItems;
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
    pub fn new() -> Result<Self, Error> {
        let mut runtime = Runtime::new().unwrap();
        let connection = Arc::new(Connection::new(&mut runtime)?);

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

    pub fn is_powered_on(self: &Self) -> Result<bool, Error> {
        self.adapter.is_powered_on()
    }

    pub fn start_advertising(
        self: &mut Self,
        name: &str,
        uuids: &[Uuid],
    ) -> Result<ConnectionItems, Error> {
        self.gatt.register()?;
        self.advertisement.add_name(name);
        self.advertisement.add_uuids(
            uuids
                .to_vec()
                .iter()
                .map(|uuid| uuid.to_string())
                .collect::<Vec<String>>(),
        );
        self.advertisement.register()
    }

    pub fn stop_advertising(self: &mut Self) -> Result<(), Error> {
        self.advertisement.unregister()?;
        self.gatt.unregister()?;

        Ok(())
    }

    pub fn is_advertising(self: &Self) -> Result<bool, Error> {
        self.advertisement.is_advertising()
    }

    pub fn add_service(self: &mut Self, service: &Service) -> Result<(), Error> {
        self.gatt.add_service(service)
    }
}
