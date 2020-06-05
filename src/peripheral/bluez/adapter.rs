use dbus::{
    arg::{RefArg, Variant},
    Message, MessageItem, Path,
};
use futures01::prelude::*;
use std::{collections::HashMap, sync::Arc};

use super::{
    connection::Connection,
    constants::{
        ADAPTER_IFACE, BLUEZ_SERVICE_NAME, DBUS_OBJECTMANAGER_IFACE, DBUS_PROPERTIES_IFACE,
        LE_ADVERTISING_MANAGER_IFACE,
    },
};
use crate::Error;

#[derive(Debug, Clone)]
pub struct Adapter {
    pub object_path: Path<'static>,
    connection: Arc<Connection>,
}

impl Adapter {
    fn find_adapter(
        connection: &Arc<Connection>,
    ) -> Box<impl Future<Item = Path<'static>, Error = Error>> {
        let message = Message::new_method_call(
            BLUEZ_SERVICE_NAME,
            "/",
            DBUS_OBJECTMANAGER_IFACE,
            "GetManagedObjects",
        )
        .unwrap();

        let method_call = connection
            .default
            .method_call(message)
            .unwrap()
            .map_err(Error::from)
            .and_then(|reply| {
                reply
                    .read1::<HashMap<
                        Path<'static>,
                        HashMap<String, HashMap<String, Variant<Box<dyn RefArg>>>>,
                    >>()
                    .map_err(Error::from)
            })
            .and_then(|managed_objects| {
                for (path, props) in managed_objects.iter() {
                    if props.contains_key(LE_ADVERTISING_MANAGER_IFACE) {
                        return Ok(path.clone());
                    }
                }

                panic!("LEAdvertisingManager1 interface not found");
            });

        Box::new(method_call)
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new(connection: Arc<Connection>) -> Box<impl Future<Item = Self, Error = Error>> {
        let adapter = Adapter::find_adapter(&connection).and_then(|object_path| {
            Ok(Adapter {
                object_path,
                connection,
            })
        });

        Box::new(adapter)
    }

    pub fn powered(self: &Self, on: bool) -> Box<impl Future<Item = (), Error = Error>> {
        let message = Message::new_method_call(
            BLUEZ_SERVICE_NAME,
            &self.object_path,
            DBUS_PROPERTIES_IFACE,
            "Set",
        )
        .unwrap()
        .append3(
            ADAPTER_IFACE,
            "Powered",
            MessageItem::Variant(Box::new(MessageItem::Bool(on))),
        );

        let method_call = self
            .connection
            .default
            .method_call(message)
            .unwrap()
            .map(|_| ())
            .map_err(Error::from);

        Box::new(method_call)
    }

    pub fn is_powered(self: &Self) -> Box<impl Future<Item = bool, Error = Error>> {
        let message = Message::new_method_call(
            BLUEZ_SERVICE_NAME,
            &self.object_path,
            DBUS_PROPERTIES_IFACE,
            "Get",
        )
        .unwrap()
        .append2(ADAPTER_IFACE, "Powered");

        let method_call = self
            .connection
            .default
            .method_call(message)
            .unwrap()
            .map_err(Error::from)
            .and_then(|message| match message.read1::<Variant<bool>>() {
                Ok(variant) => Ok(variant.0),
                Err(err) => Err(Error::from(err)),
            });

        Box::new(method_call)
    }
}
