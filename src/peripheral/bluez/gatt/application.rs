use dbus::{
    arg::{RefArg, Variant},
    Message, Path,
};
use dbus_tokio::tree::AFactory;
use futures::prelude::*;
use std::{collections::HashMap, sync::Arc};

use super::super::{
    common,
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
        tree: &mut common::Tree,
        adapter: Path<'static>,
    ) -> Self {
        let factory = AFactory::new_afn::<common::TData>();

        let object_path = factory
            .object_path(PATH_BASE, common::GattDataType::None)
            .introspectable()
            .object_manager();

        let path = object_path.get_name().clone();

        tree.insert(object_path);

        Application {
            connection,
            object_path: path,
            adapter,
        }
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
            HashMap::<String, Variant<Box<dyn RefArg>>>::new(),
        );

        Box::new(
            self.connection
                .default
                .method_call(message)
                .unwrap()
                .map_err(Error::from),
        )
    }

    pub fn unregister(self: &Self) -> Box<impl Future<Item = (), Error = Error>> {
        let message = Message::new_method_call(
            BLUEZ_SERVICE_NAME,
            &self.adapter,
            GATT_GATT_MANAGER_IFACE,
            "UnregisterApplication",
        )
        .unwrap()
        .append1(&self.object_path);

        let method_call = self
            .connection
            .default
            .method_call(message)
            .unwrap()
            .map(|_| ())
            .map_err(Error::from);

        Box::new(method_call)
    }
}
