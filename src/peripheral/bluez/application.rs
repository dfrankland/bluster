use dbus::{
    arg::{RefArg, Variant},
    tree::{Factory, MTFn, Tree},
    Connection, Message, Path,
};
use std::collections::HashMap;

use super::{
    adapter::Adapter,
    constants::{BLUEZ_SERVICE_NAME, GATT_GATT_MANAGER_IFACE, PATH_BASE},
};

#[derive(Debug, Clone)]
pub struct Application {
    pub object_path: Path<'static>,
    adapter: Adapter,
}

impl Application {
    pub fn new(
        factory: &Factory<MTFn>,
        tree: &mut Tree<MTFn, ()>,
        adapter: Adapter,
    ) -> Result<Self, dbus::Error> {
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
            &self.adapter.object_path,
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
            &self.adapter.object_path,
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
