use dbus::{
    arg::{RefArg, Variant},
    tree::Access,
    Message, Path,
};
use dbus_tokio::tree::{AFactory, ATree};
use futures::{compat::*, prelude::*};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use super::{
    common,
    connection::Connection,
    constants::{
        BLUEZ_SERVICE_NAME, LE_ADVERTISEMENT_IFACE, LE_ADVERTISING_MANAGER_IFACE, PATH_BASE,
    },
};
use crate::Error;

#[derive(Debug, Clone)]
pub struct Advertisement {
    connection: Arc<Connection>,
    adapter: Path<'static>,
    pub object_path: Path<'static>,
    tree: Arc<Mutex<Option<common::Tree>>>,
    is_advertising: Arc<AtomicBool>,
    name: Arc<Mutex<Option<String>>>,
    uuids: Arc<Mutex<Option<Vec<String>>>>,
}

impl Advertisement {
    pub fn new(connection: Arc<Connection>, adapter: Path<'static>) -> Self {
        let factory = AFactory::new_afn::<common::TData>();

        let is_advertising = Arc::new(AtomicBool::new(false));
        let is_advertising_release = is_advertising.clone();

        let name = Arc::new(Mutex::new(None));
        let name_property = name.clone();

        let uuids = Arc::new(Mutex::new(None));
        let uuids_property = uuids.clone();

        let advertisement = factory
            .interface(LE_ADVERTISEMENT_IFACE, ())
            .add_m(factory.amethod("Release", (), move |method_info| {
                is_advertising_release.store(false, Ordering::Relaxed);
                Ok(vec![method_info.msg.method_return()])
            }))
            .add_p(
                factory
                    .property::<&str, _>("Type", ())
                    .access(Access::Read)
                    .on_get(|i, _| {
                        i.append("peripheral");
                        Ok(())
                    }),
            )
            .add_p(
                factory
                    .property::<&str, _>("LocalName", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        if let Ok(guard) = name_property.lock() {
                            if let Some(local_name) = guard.clone() {
                                i.append(local_name);
                            }
                        }
                        Ok(())
                    }),
            )
            .add_p(
                factory
                    .property::<&[&str], _>("ServiceUUIDs", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        if let Ok(guard) = uuids_property.lock() {
                            if let Some(service_uuids) = guard.clone() {
                                i.append(service_uuids);
                            }
                        }
                        Ok(())
                    }),
            );

        let object_path = factory
            .object_path(
                format!("{}/advertisement{:04}", PATH_BASE, 0),
                common::GattDataType::None,
            )
            .add(advertisement);
        let path = object_path.get_name().clone();
        let tree = factory.tree(ATree::new()).add(object_path);

        Advertisement {
            connection,
            adapter,
            object_path: path,
            tree: Arc::new(Mutex::new(Some(tree))),
            is_advertising,
            name,
            uuids,
        }
    }

    pub fn add_name<T: Into<String>>(self: &Self, name: T) {
        self.name.lock().unwrap().replace(name.into());
    }

    pub fn add_uuids<T: Into<Vec<String>>>(self: &Self, uuids: T) {
        self.uuids.lock().unwrap().replace(uuids.into());
    }

    pub async fn register(self: &Self) -> Result<(), Error> {
        // Register with DBus
        let mut tree = self.tree.lock().unwrap();

        tree.as_mut()
            .unwrap()
            .set_registered(&self.connection.fallback, true)?;

        self.connection
            .fallback
            .add_handler(Arc::new(tree.take().unwrap()));

        // Create message to register advertisement with Bluez
        let message = Message::new_method_call(
            BLUEZ_SERVICE_NAME,
            &self.adapter,
            LE_ADVERTISING_MANAGER_IFACE,
            "RegisterAdvertisement",
        )
        .unwrap()
        .append2(
            &self.object_path,
            HashMap::<String, Variant<Box<dyn RefArg>>>::new(),
        );

        // Send message
        let is_advertising = self.is_advertising.clone();
        self.connection
            .default
            .method_call(message)
            .unwrap()
            .compat()
            .map_ok(move |_| {
                is_advertising.store(true, Ordering::Relaxed);
            })
            .map_err(Error::from)
            .await
    }

    pub async fn unregister(self: &Self) -> Result<(), Error> {
        // Create message to ungregister advertisement with Bluez
        let message = Message::new_method_call(
            BLUEZ_SERVICE_NAME,
            &self.adapter,
            LE_ADVERTISING_MANAGER_IFACE,
            "UnregisterAdvertisement",
        )
        .unwrap()
        .append1(&self.object_path);

        // Send message
        let is_advertising = self.is_advertising.clone();
        let method_call = self
            .connection
            .default
            .method_call(message)
            .unwrap()
            .compat()
            .map_ok(|_| ())
            .map_err(Error::from);

        is_advertising.store(false, Ordering::Relaxed);

        method_call.await
    }

    pub fn is_advertising(self: &Self) -> bool {
        let is_advertising = self.is_advertising.clone();
        is_advertising.load(Ordering::Relaxed)
    }
}
