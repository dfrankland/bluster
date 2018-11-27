mod application;
mod characteristic;
mod descriptor;
mod service;

use dbus::{Message, Path};
use dbus_tokio::tree::{AFactory, ATree, ATreeServer};
use futures::{prelude::*, Async};
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
}

impl Gatt {
    pub fn new(connection: Arc<Connection>, adapter: Path<'static>) -> Self {
        let factory = AFactory::new_afn::<()>();

        Gatt {
            adapter,
            connection,
            tree: Arc::new(Mutex::new(Some(factory.tree(ATree::new())))),
            application: Arc::new(Mutex::new(None)),
        }
    }

    pub fn add_service(self: &Self, service: &gatt::service::Service) -> Result<(), Error> {
        let mut tree = self.tree.lock().unwrap();
        let tree = tree.as_mut().unwrap();

        let gatt_service = Service::new(tree, &Arc::new(service.clone()))?;

        for characteristic in service.characteristics.iter() {
            let gatt_characteristic = Characteristic::new(
                tree,
                &Arc::new(characteristic.clone()),
                &Arc::new(gatt_service.object_path.clone()),
            )?;

            for descriptor in characteristic.descriptors.iter() {
                Descriptor::new(
                    tree,
                    &Arc::new(descriptor.clone()),
                    &Arc::new(gatt_characteristic.object_path.clone()),
                )?;
            }
        }

        Ok(())
    }

    pub fn register(
        self: &Self,
    ) -> Result<
        Box<
            impl Future<
                Item = Box<impl Stream<Item = Message, Error = ()>>,
                Error = ((), impl Stream<Item = Message, Error = ()>),
            >,
        >,
        Error,
    > {
        let mut tree = self.tree.lock().unwrap();
        let mut tree = tree.take().unwrap();

        let new_application = Application::new(
            Arc::clone(&self.connection),
            &mut tree,
            self.adapter.clone(),
        )?;

        tree.set_registered(&self.connection.fallback, true)?;

        let mut registration = new_application.register();
        self.application.lock().unwrap().replace(new_application);

        let server = ATreeServer::new(
            Rc::clone(&self.connection.fallback),
            Arc::new(tree),
            self.connection.default.messages().unwrap(),
        )
        .skip_while(move |_| match registration.poll() {
            Ok(ready) => match ready {
                Async::Ready(_) => {
                    println!("Application registered");
                    Ok(true)
                }
                _ => Ok(false),
            },
            _ => Ok(false),
        })
        .into_future()
        .and_then(move |(.., stream)| Ok(Box::new(stream)));

        Ok(Box::new(server))
    }

    pub fn unregister(
        self: &Self,
    ) -> Result<Box<impl Future<Item = Message, Error = dbus::Error>>, Error> {
        let application = self.application.lock().unwrap().take().unwrap();
        Ok(application.unregister())
    }
}
