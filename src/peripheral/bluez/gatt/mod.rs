mod application;
mod characteristic;
mod descriptor;
mod flags;
mod service;

use dbus::Path;
use dbus_tokio::tree::{AFactory, ATree, ATreeServer};
use futures01::{future, prelude::*, sync::mpsc::unbounded};
use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

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
    tree: Arc<Mutex<Option<common::Tree>>>,
    application: Arc<Mutex<Option<Application>>>,
    service_index: Arc<Mutex<u64>>,
    characteristic_index: Arc<Mutex<u64>>,
    descriptor_index: Arc<Mutex<u64>>,
}

impl Gatt {
    pub fn new(connection: Arc<Connection>, adapter: Path<'static>) -> Self {
        let factory = AFactory::new_afn::<common::TData>();

        Gatt {
            adapter,
            connection,
            tree: Arc::new(Mutex::new(Some(factory.tree(ATree::new())))),
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

    pub fn register(
        self: &Self,
    ) -> Box<impl Future<Item = Box<impl Stream<Item = (), Error = Error>>, Error = Error>> {
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

        if let Err(err) = tree.set_registered(&self.connection.fallback, true) {
            return Box::new(future::Either::A(future::err(Error::from(err))));
        };

        let (sender, receiver) = unbounded();

        let registration = new_application.register().and_then(move |_| {
            sender.unbounded_send(()).unwrap();
            Ok(())
        });

        let server = ATreeServer::new(
            Rc::clone(&self.connection.fallback),
            Arc::new(tree),
            self.connection.default.messages().unwrap(),
        )
        .map(|_| ())
        .map_err(Error::from)
        .select(receiver.map_err(Error::from))
        .into_future()
        .map(|(.., stream)| Box::new(stream))
        .map_err(|(err, ..)| err);

        Box::new(future::Either::B(
            registration.join(server).map(|(.., server)| server),
        ))
    }

    pub fn unregister(self: &Self) -> Box<impl Future<Item = (), Error = Error>> {
        let application = self.application.lock().unwrap().take().unwrap();
        Box::new(application.unregister().map(|_| ()))
    }
}
