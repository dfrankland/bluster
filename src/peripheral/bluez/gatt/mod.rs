mod application;
mod characteristic;
mod descriptor;
mod service;

use dbus::Path;
use dbus_tokio::tree::{AFactory, ATree};
use std::sync::Arc;

use self::{
    application::Application, characteristic::Characteristic, descriptor::Descriptor,
    service::Service,
};
use super::{common, Connection};
use crate::{gatt, Error};

#[derive(Debug)]
pub struct Gatt {
    connection: Arc<Connection>,
    adapter: Path<'static>,
    tree: Option<common::Tree>,
    application: Option<Application>,
}

impl Gatt {
    pub fn new(connection: Arc<Connection>, adapter: Path<'static>) -> Self {
        let factory = AFactory::new_afn::<()>();

        Gatt {
            adapter,
            connection,
            tree: Some(factory.tree(ATree::new())),
            application: None,
        }
    }

    pub fn add_service(self: &mut Self, service: &gatt::service::Service) -> Result<(), Error> {
        let gatt_service = Service::new(self.tree.as_mut().unwrap(), &Arc::new(service.clone()))?;

        for characteristic in service.characteristics.iter() {
            let gatt_characteristic = Characteristic::new(
                self.tree.as_mut().unwrap(),
                &Arc::new(characteristic.clone()),
                &Arc::new(gatt_service.object_path.clone()),
            )?;

            for descriptor in characteristic.descriptors.iter() {
                Descriptor::new(
                    self.tree.as_mut().unwrap(),
                    &Arc::new(descriptor.clone()),
                    &Arc::new(gatt_characteristic.object_path.clone()),
                )?;
            }
        }

        Ok(())
    }

    pub fn register(self: &mut Self) -> Result<(), Error> {
        let tree = self.tree.as_mut().unwrap();
        let application = Application::new(tree, self.adapter.clone())?;

        tree.set_registered(&self.connection.fallback, true)?;
        self.connection
            .fallback
            .add_handler(self.tree.take().unwrap());

        application.register(&self.connection.fallback);

        self.application.replace(application);

        Ok(())
    }

    pub fn unregister(self: &mut Self) -> Result<(), Error> {
        if let Some(application) = self.application.take() {
            application.unregister(&self.connection.fallback);
        }

        Ok(())
    }
}
