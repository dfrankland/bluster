mod adapter;
mod advertisement;
mod application;
mod characteristic;
mod constants;
mod descriptor;
mod error;
mod service;

use dbus::{
    ConnectionItems,
    tree::{Factory, MTFn, Tree},
    BusType, Connection,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use uuid::Uuid;

use crate::{gatt, Error};

use adapter::Adapter;
use advertisement::Advertisement;
use application::Application;
use characteristic::Characteristic;
use descriptor::Descriptor;
use service::Service;

#[derive(Debug)]
pub struct Peripheral {
    connection: Connection,
    adapter: Adapter,
    factory: Factory<MTFn>,
    tree: Option<Tree<MTFn, ()>>,
    application: Option<Application>,
    advertisement: Option<Advertisement>,
    is_advertising: Arc<AtomicBool>,
}

impl Peripheral {
    pub fn new() -> Result<Self, Error> {
        let connection = Connection::get_private(BusType::System)?;
        let adapter = Adapter::new(&connection)?;
        adapter.powered_on(&connection, true)?;

        let factory = Factory::new_fn::<()>();

        Ok(
            Peripheral {
                connection,
                adapter,
                tree: Some(factory.tree(())),
                factory,
                application: None,
                advertisement: None,
                is_advertising: Arc::new(AtomicBool::new(false)),
            }
        )
    }

    pub fn is_powered_on(self: &Self) -> Result<bool, Error> {
        Ok(self.adapter.is_powered_on(&self.connection)?)
    }

    pub fn start_advertising(self: &mut Self, name: &str, uuids: &[Uuid]) -> Result<ConnectionItems, Error> {
        let application = Application::new(
            &self.factory,
            self.tree.as_mut().unwrap(),
            self.adapter.clone(),
        )?;

        self.tree.as_ref().unwrap().set_registered(&self.connection, true)?;
        self.connection.add_handler(Arc::new(self.tree.take().unwrap()));

        application.register(&self.connection);
        self.application.replace(application);

        let advertisement = Advertisement::new(
            &self.factory,
            self.adapter.clone(),
            self.is_advertising.clone(),
            Arc::new(name.to_string()),
            Arc::new(
                uuids
                    .to_vec()
                    .iter()
                    .map(|uuid| uuid.to_string())
                    .collect::<Vec<String>>(),
            ),
        );
        advertisement.register(&self.connection).unwrap();
        self.advertisement.replace(advertisement);

        Ok(self.connection.iter(1000))
    }

    pub fn stop_advertising(self: &mut Self) -> Result<(), Error> {
        self.advertisement
            .take()
            .unwrap()
            .unregister(&self.connection);

        self.application
            .take()
            .unwrap()
            .unregister(&self.connection);

        Ok(())
    }

    pub fn is_advertising(self: &Self) -> Result<bool, Error> {
        let is_advertising = self.is_advertising.clone();
        Ok(is_advertising.load(Ordering::Relaxed))
    }

    pub fn add_service(self: &mut Self, service: &gatt::service::Service) -> Result<(), Error> {
        let gatt_service = Service::new(
            &self.factory,
            self.tree.as_mut().unwrap(),
            &Arc::new(service.clone()),
        )?;

        for characteristic in service.characteristics.iter() {
            let gatt_characteristic = Characteristic::new(
                &self.factory,
                self.tree.as_mut().unwrap(),
                &Arc::new(characteristic.clone()),
                &Arc::new(gatt_service.object_path.clone()),
            )?;

            for descriptor in characteristic.descriptors.iter() {
                Descriptor::new(
                    &self.factory,
                    self.tree.as_mut().unwrap(),
                    &Arc::new(descriptor.clone()),
                    &Arc::new(gatt_characteristic.object_path.clone()),
                )?;
            }
        }

        Ok(())
    }
}
