use dbus::{
    arg::{RefArg, Variant},
    Message, MessageItem, Path,
};
use futures::compat::*;
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
    async fn find_adapter(connection: &Arc<Connection>) -> Result<Path<'static>, Error> {
        let message = Message::new_method_call(
            BLUEZ_SERVICE_NAME,
            "/",
            DBUS_OBJECTMANAGER_IFACE,
            "GetManagedObjects",
        )
        .unwrap();

        Ok(connection
                .default
                .method_call(message)
                .unwrap()
                .compat()
                .await?
                .read1::<HashMap<
                    Path<'static>,
                    HashMap<String, HashMap<String, Variant<Box<dyn RefArg>>>>,
                >>()?
                .into_iter()
                .find(|(_path, props)| props.contains_key(LE_ADVERTISING_MANAGER_IFACE))
                .map(|(path, _props)| path)
                .expect("LEAdvertisingManager1 interface not found"))
    }

    #[allow(clippy::new_ret_no_self)]
    pub async fn new(connection: Arc<Connection>) -> Result<Self, Error> {
        Adapter::find_adapter(&connection)
            .await
            .map(|object_path| Adapter {
                object_path,
                connection,
            })
    }

    pub async fn powered(self: &Self, on: bool) -> Result<(), Error> {
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

        self.connection
            .default
            .method_call(message)
            .unwrap()
            .compat()
            .await
            .map(|_| ())
            .map_err(Error::from)
    }

    pub async fn is_powered(self: &Self) -> Result<bool, Error> {
        let message = Message::new_method_call(
            BLUEZ_SERVICE_NAME,
            &self.object_path,
            DBUS_PROPERTIES_IFACE,
            "Get",
        )
        .unwrap()
        .append2(ADAPTER_IFACE, "Powered");

        self.connection
            .default
            .method_call(message)
            .unwrap()
            .compat()
            .await
            .map_err(Error::from)
            .and_then(|message| match message.read1::<Variant<bool>>() {
                Ok(variant) => Ok(variant.0),
                Err(err) => Err(Error::from(err)),
            })
    }
}
