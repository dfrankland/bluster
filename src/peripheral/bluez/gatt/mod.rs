mod application;
mod characteristic;
mod descriptor;
mod flags;
mod service;

use dbus::{channel::MatchingReceiver, message::MatchRule, Path};
use std::sync::{Arc, Mutex};

use self::{
    application::Application, characteristic::Characteristic, descriptor::Descriptor,
    service::Service,
};
use super::{common, constants::PATH_BASE, Connection};
use crate::{gatt, Error};

#[derive(Debug)]
pub struct Gatt {
    connection: Arc<Connection>,
    adapter: Path<'static>,
    tree: Arc<Mutex<Option<common::Tree>>>,
    application: Arc<Mutex<Option<Application>>>,
    service_index: Arc<Mutex<u64>>,
    characteristic_index: Arc<Mutex<u64>>,
    descriptor_index: Arc<Mutex<u64>>,
}

impl Gatt {
    pub fn new(connection: Arc<Connection>, adapter: Path<'static>) -> Self {
        let mut tree = common::Tree::new();
        tree.set_async_support(Some((
            connection.default.clone(),
            Box::new(|x| {
                tokio::spawn(x);
            }),
        )));
        Gatt {
            adapter,
            connection,
            tree: Arc::new(Mutex::new(Some(tree))),
            application: Arc::new(Mutex::new(None)),
            service_index: Arc::new(Mutex::new(0)),
            characteristic_index: Arc::new(Mutex::new(0)),
            descriptor_index: Arc::new(Mutex::new(0)),
        }
    }

    pub fn add_service(self: &Self, service: &gatt::service::Service) -> Result<(), Error> {
        let mut tree = self.tree.lock().unwrap();
        let tree = tree.as_mut().unwrap();

        let mut service_index = self.service_index.lock().unwrap();
        let mut characteristic_index = self.characteristic_index.lock().unwrap();
        let mut descriptor_index = self.descriptor_index.lock().unwrap();

        let gatt_service = Service::new(tree, &Arc::new(service.clone()), *service_index)?;
        *service_index += 1;

        for characteristic in service.characteristics.iter() {
            let gatt_characteristic = Characteristic::new(
                &self.connection.clone(),
                tree,
                &Arc::new(characteristic.clone()),
                &Arc::new(gatt_service.object_path.clone()),
                *characteristic_index,
            )?;
            *characteristic_index += 1;

            for descriptor in characteristic.descriptors.iter() {
                Descriptor::new(
                    tree,
                    &Arc::new(descriptor.clone()),
                    &Arc::new(gatt_characteristic.object_path.clone()),
                    *descriptor_index,
                )?;
                *descriptor_index += 1;
            }
        }

        Ok(())
    }

    pub async fn register(self: &Self) -> Result<(), Error> {
        let mut tree = self.tree.lock().unwrap().take().unwrap();

        let new_application = Application::new(
            Arc::clone(&self.connection),
            &mut tree,
            self.adapter.clone(),
        );

        self.application
            .lock()
            .unwrap()
            .replace(new_application.clone());

        let mut match_rule = MatchRule::new_method_call();
        match_rule.path = Some(PATH_BASE.into());
        match_rule.path_is_namespace = true;
        self.connection.default.start_receive(
            match_rule,
            Box::new(move |msg, conn| {
                tree.handle_message(msg, conn).unwrap();
                true
            }),
        );

        new_application.register().await
    }

    pub async fn unregister(self: &Self) -> Result<(), Error> {
        self.application
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .unregister()
            .await
            .map(|_| ())
    }
}
