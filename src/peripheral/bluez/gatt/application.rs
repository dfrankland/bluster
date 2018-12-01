use dbus::{
    arg::{RefArg, Variant},
    tree::{MTFn, Tree},
    Message, Path,
};
use dbus_tokio::tree::{AFactory, ATree};
use futures::prelude::*;
use std::{collections::HashMap, sync::Arc};

use super::super::{
    constants::{BLUEZ_SERVICE_NAME, GATT_GATT_MANAGER_IFACE, PATH_BASE},
    Connection, Error,
};

#[derive(Debug, Clone)]
pub struct Application {
    connection: Arc<Connection>,
    pub object_path: Path<'static>,
    adapter: Path<'static>,
}

impl Application {
    pub fn new(
        connection: Arc<Connection>,
        tree: &mut Tree<MTFn<ATree<()>>, ATree<()>>,
        adapter: Path<'static>,
    ) -> Result<Self, dbus::Error> {
        let factory = AFactory::new_afn::<()>();

        let object_path = factory
            .object_path(PATH_BASE, ())
            .introspectable()
            .object_manager();

        let path = object_path.get_name().clone();

        tree.insert(object_path);

        Ok(Application {
            connection,
            object_path: path,
            adapter,
        })
    }

    pub fn register(self: &Self) -> Box<impl Future<Item = Message, Error = Error>> {
        let message = Message::new_method_call(
            BLUEZ_SERVICE_NAME,
            &self.adapter,
            GATT_GATT_MANAGER_IFACE,
            "RegisterApplication",
        )
        .unwrap()
        .append2(
            &self.object_path,
            HashMap::<String, Variant<Box<RefArg>>>::new(),
        );

        Box::new(
            self.connection
                .default
                .method_call(message)
                .unwrap()
                .map_err(Error::from),
        )
    }

    pub fn unregister(self: &Self) -> Box<impl Future<Item = Message, Error = Error>> {
        let message = Message::new_method_call(
            BLUEZ_SERVICE_NAME,
            &self.adapter,
            GATT_GATT_MANAGER_IFACE,
            "UnregisterApplication",
        )
        .unwrap()
        .append1(&self.object_path);
        Box::new(
            self.connection
                .default
                .method_call(message)
                .unwrap()
                .map_err(Error::from),
        )
    }
}
