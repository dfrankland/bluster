mod adapter;
mod advertisement;
mod characteristic;
mod constants;
mod descriptor;
mod service;

use adapter::Adapter;
use advertisement::Advertisement;
use crate::gatt;
use dbus::{
    tree::{Factory, MTFn},
    BusType, Connection,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use uuid::Uuid;

use characteristic::Characteristic;
use descriptor::Descriptor;
use service::Service;

#[derive(Debug)]
pub struct Peripheral {
    connection: Connection,
    adapter: Adapter,
    factory: Factory<MTFn>,
    advertisement: Option<Advertisement>,
    is_advertising: Arc<AtomicBool>,
}

impl Peripheral {
    pub fn new() -> Self {
        let connection = Connection::get_private(BusType::System).unwrap();
        let adapter = Adapter::new(&connection).unwrap();
        adapter.powered_on(&connection, true).unwrap();

        Peripheral {
            connection,
            adapter,
            factory: Factory::new_fn::<()>(),
            advertisement: None,
            is_advertising: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_powered_on(self: &Self) -> bool {
        self.adapter.is_powered_on(&self.connection).unwrap()
    }

    pub fn start_advertising(self: &mut Self, name: &str, uuids: &[Uuid]) {
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
        self.advertisement = Some(advertisement);
    }

    pub fn stop_advertising(self: &mut Self) {
        self.advertisement
            .as_ref()
            .unwrap()
            .unregister(&self.connection);
        self.advertisement = None;
    }

    pub fn is_advertising(self: &Self) -> bool {
        let is_advertising = self.is_advertising.clone();
        is_advertising.load(Ordering::Relaxed)
    }

    pub fn add_service(self: &Self, service: &gatt::service::Service) {
        let gatt_service = Service::new(&self.factory, &Arc::new(service.clone()));
        gatt_service.register(&self.connection).unwrap();
        for characteristic in service.characteristics.iter() {
            let gatt_characteristic = Characteristic::new(
                &self.factory,
                &Arc::new(characteristic.clone()),
                &Arc::new(gatt_service.object_path.clone()),
            );
            gatt_characteristic.register(&self.connection).unwrap();
            for descriptor in characteristic.descriptors.iter() {
                let gatt_descriptor = Descriptor::new(
                    &self.factory,
                    &Arc::new(descriptor.clone()),
                    &Arc::new(gatt_characteristic.object_path.clone()),
                );
                gatt_descriptor.register(&self.connection).unwrap();
            }
        }
    }
}

impl Default for Peripheral {
    fn default() -> Self {
        Peripheral::new()
    }
}
