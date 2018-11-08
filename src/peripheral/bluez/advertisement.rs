use dbus::{
    arg::{RefArg, Variant},
    tree::{Factory, MTFn, Tree},
    Connection, Message, MessageItem, MessageItemArray, Path, Signature,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use super::{
    adapter::Adapter,
    constants::{
        BLUEZ_SERVICE_NAME, DBUS_PROP_IFACE, LE_ADVERTISEMENT_IFACE, LE_ADVERTISING_MANAGER_IFACE,
        PATH_BASE,
    },
};

#[derive(Debug, Clone)]
pub struct Advertisement {
    pub object_path: Path<'static>,
    tree: Arc<Tree<MTFn, ()>>,
    is_advertising: Arc<AtomicBool>,
    adapter: Adapter,
}

impl Advertisement {
    pub fn new(
        factory: &Factory<MTFn>,
        adapter: Adapter,
        is_advertising: Arc<AtomicBool>,
        name: Arc<String>,
        uuids: Arc<Vec<String>>,
    ) -> Self {
        let mut object_path =
            factory.object_path(format!("{}/advertisement{:04}", PATH_BASE, 0), ());

        let is_advertising_release = is_advertising.clone();
        let release = factory
            .interface(LE_ADVERTISEMENT_IFACE, ())
            .add_m(factory.method("Release", (), move |method_info| {
                is_advertising_release.store(false, Ordering::Relaxed);
                Ok(vec![method_info.msg.method_return()])
            }));
        object_path = object_path.add(release);

        let get_all = factory.interface(DBUS_PROP_IFACE, ()).add_m(
            factory
                .method("GetAll", (), move |method_info| {
                    let mut props = HashMap::new();

                    let (local_name, service_uuids) =
                        (name.clone().to_owned(), uuids.clone().to_owned());

                    props.insert("Type", Variant(MessageItem::Str("peripheral".to_string())));
                    props.insert(
                        "LocalName",
                        Variant(MessageItem::Str(local_name.to_string())),
                    );
                    if !service_uuids.is_empty() {
                        props.insert(
                            "ServiceUUIDs",
                            Variant(MessageItem::Array(
                                MessageItemArray::new(
                                    service_uuids
                                        .iter()
                                        .map(|x| MessageItem::Str(x.to_string()))
                                        .collect::<Vec<MessageItem>>(),
                                    Signature::make::<String>(),
                                )
                                .unwrap(),
                            )),
                        );
                    }

                    Ok(vec![method_info.msg.method_return().append1(props)])
                })
                .in_arg(Signature::make::<String>())
                .out_arg(Signature::make::<HashMap<String, Variant<MessageItem>>>()),
        );
        object_path = object_path.add(get_all);

        let path = object_path.get_name().clone();

        let tree = Arc::new(factory.tree(()).add(object_path));

        Advertisement {
            object_path: path,
            tree,
            is_advertising,
            adapter,
        }
    }

    pub fn register(self: &Self, connection: &Connection) -> Result<(), dbus::Error> {
        self.register_with_dbus(connection)?;
        self.register_with_bluez(connection);
        Ok(())
    }

    fn register_with_dbus(self: &Self, connection: &Connection) -> Result<(), dbus::Error> {
        self.tree.set_registered(connection, true)?;
        connection.add_handler(self.tree.clone());
        Ok(())
    }

    fn register_with_bluez(self: &Self, connection: &Connection) {
        // Create message to register advertisement with Bluez
        let message = Message::new_method_call(
            BLUEZ_SERVICE_NAME,
            &self.adapter.object_path,
            LE_ADVERTISING_MANAGER_IFACE,
            "RegisterAdvertisement",
        )
        .unwrap()
        .append2(
            &self.object_path,
            HashMap::<String, Variant<Box<RefArg>>>::new(),
        );

        // Send message
        let is_advertising = self.is_advertising.clone();
        let done: std::rc::Rc<std::cell::Cell<bool>> = Default::default();
        let done2 = done.clone();
        connection.add_handler(
            connection
                .send_with_reply(message, move |_| {
                    is_advertising.store(true, Ordering::Relaxed);
                    done2.set(true);
                })
                .unwrap(),
        );
        while !done.get() {
            connection.incoming(100).next();
        }
    }

    pub fn unregister(self: &Self, connection: &Connection) {
        // Create message to ungregister advertisement with Bluez
        let message = Message::new_method_call(
            BLUEZ_SERVICE_NAME,
            &self.adapter.object_path,
            LE_ADVERTISING_MANAGER_IFACE,
            "UnregisterAdvertisement",
        )
        .unwrap()
        .append1(&self.object_path);

        // Send message
        let is_advertising = self.is_advertising.clone();
        let done: std::rc::Rc<std::cell::Cell<bool>> = Default::default();
        let done2 = done.clone();
        connection.add_handler(
            connection
                .send_with_reply(message, move |_| {
                    is_advertising.store(false, Ordering::Relaxed);
                    done2.set(true);
                })
                .unwrap(),
        );
        while !done.get() {
            connection.incoming(100).next();
        }
    }
}
