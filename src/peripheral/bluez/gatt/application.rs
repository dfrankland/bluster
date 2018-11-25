use dbus::{
    arg::{RefArg, Variant},
    tree::{MTFn, Tree},
    Connection, Message, Path,
};
use dbus_tokio::tree::{AFactory, ATree};
use std::collections::HashMap;

use super::super::constants::{BLUEZ_SERVICE_NAME, GATT_GATT_MANAGER_IFACE, PATH_BASE};

#[derive(Debug, Clone)]
pub struct Application {
    pub object_path: Path<'static>,
    adapter: Path<'static>,
}

impl Application {
    pub fn new(
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
            object_path: path,
            adapter,
        })
    }

    pub fn register(self: &Self, connection: &Connection) {
        // Create message to register GATT with Bluez
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

        // Send message
        let done: std::rc::Rc<std::cell::Cell<bool>> = Default::default();
        let done2 = done.clone();
        connection.add_handler(
            connection
                .send_with_reply(message, move |_| {
                    done2.set(true);
                })
                .unwrap(),
        );
        while !done.get() {
            connection.incoming(100).next();
        }
    }

    pub fn unregister(self: &Self, connection: &Connection) {
        // Create message to ungregister GATT with Bluez
        let message = Message::new_method_call(
            BLUEZ_SERVICE_NAME,
            &self.adapter,
            GATT_GATT_MANAGER_IFACE,
            "UnregisterApplication",
        )
        .unwrap()
        .append1(&self.object_path);

        // Send message
        let done: std::rc::Rc<std::cell::Cell<bool>> = Default::default();
        let done2 = done.clone();
        connection.add_handler(
            connection
                .send_with_reply(message, move |_| {
                    done2.set(true);
                })
                .unwrap(),
        );
        while !done.get() {
            connection.incoming(100).next();
        }
    }
}
