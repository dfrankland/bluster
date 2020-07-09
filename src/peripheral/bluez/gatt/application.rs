use dbus::{
    arg::{RefArg, Variant},
    Path,
};
use std::{collections::HashMap, sync::Arc};

use super::super::{
    common,
    constants::{GATT_GATT_MANAGER_IFACE, PATH_BASE},
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
        tree.insert(PATH_BASE, &[tree.object_manager()], ());

        Application {
            connection,
            object_path: PATH_BASE.into(),
            adapter,
        }
    }

    pub async fn register(self: &Self) -> Result<(), Error> {
        let proxy = self.connection.get_bluez_proxy(&self.adapter);
        proxy
            .method_call(
                GATT_GATT_MANAGER_IFACE,
                "RegisterApplication",
                (
                    &self.object_path,
                    HashMap::<String, Variant<Box<dyn RefArg>>>::new(),
                ),
            )
            .await
            .map_err(From::from)
    }

    pub async fn unregister(self: &Self) -> Result<(), Error> {
        let proxy = self.connection.get_bluez_proxy(&self.adapter);
        proxy
            .method_call(
                GATT_GATT_MANAGER_IFACE,
                "UnregisterApplication",
                (&self.object_path,),
            )
            .await
            .map_err(From::from)
    }
}
