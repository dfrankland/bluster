mod adapter;
mod advertisement;
mod common;
mod connection;
mod constants;
mod error;
mod gatt;

use futures::{compat::*, future, prelude::*};
use futures01::future::Future as _;
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
    pub fn new(runtime: &Arc<Mutex<Runtime>>) -> Box<impl Future<Output = Result<Self, Error>>> {
        let connection = match Connection::new(Arc::clone(&runtime)) {
            Ok(connection) => Arc::new(connection),
            Err(err) => return Box::new(future::Either::Left(future::err(err))),
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
            })
            .compat();

        Box::new(future::Either::Right(peripheral))
    }

    pub async fn is_powered(self: &Self) -> Result<bool, Error> {
        self.adapter.is_powered().compat().await
    }

    pub async fn start_advertising(
        self: &Self,
        name: &str,
        uuids: &[Uuid],
    ) -> Result<impl Stream<Item = Result<(), Error>>, Error> {
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
        let registration = gatt.join(advertisement).map(|(stream, ..)| stream.compat());

        registration.compat().await
    }

    pub async fn stop_advertising(self: &Self) -> Result<(), Error> {
        let advertisement = self.advertisement.unregister();
        let gatt = self.gatt.unregister();
        advertisement.join(gatt).map(|_| ()).compat().await
    }

    pub async fn is_advertising(self: &Self) -> bool {
        self.advertisement.is_advertising()
    }

    pub fn add_service(self: &Self, service: &Service) -> Result<(), Error> {
        self.gatt.add_service(service)
    }
}
