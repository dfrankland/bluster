use dbus::{
    arg::{RefArg, Variant},
    stdintf::org_freedesktop_dbus::ObjectManager,
    tree::{Factory, MTFn},
    BusType, Connection, Message, MessageItem, Path, Props,
};
use std::{boxed::Box, collections::HashMap};
use uuid::Uuid;

const DBUS_PROP_IFACE: &str = "org.freedesktop.DBus.Properties";
const BLUEZ_SERVICE_NAME: &str = "org.bluez";
const ADAPTER_IFACE: &str = "org.bluez.Adapter1";
const LE_ADVERTISING_MANAGER_IFACE: &str = "org.bluez.LEAdvertisingManager1";
const LE_ADVERTISEMENT_IFACE: &str = "org.bluez.LEAdvertisement1";

const PATH_BASE: &str = "/org/bluez/example/advertisement";

#[derive(Debug)]
pub struct Peripheral {
    connection: Connection,
    adapter_object_path: String,
    factory: Factory<MTFn>,
    advertisement_object_path: String,
}

impl Peripheral {
    fn find_adapter(connection: &Connection) -> String {
        let connection_path = connection.with_path(BLUEZ_SERVICE_NAME, "/", 5000);
        let managed_objects = connection_path.get_managed_objects().unwrap();
        for (path, props) in managed_objects.iter() {
            if props.contains_key(LE_ADVERTISING_MANAGER_IFACE) {
                return path.to_string();
            }
        }

        panic!("LEAdvertisingManager1 interface not found");
    }

    pub fn new() -> Self {
        let connection = Connection::get_private(BusType::System).unwrap();
        let adapter_object_path = Peripheral::find_adapter(&connection);
        let adapter_props = Props::new(
            &connection,
            BLUEZ_SERVICE_NAME,
            &adapter_object_path,
            ADAPTER_IFACE,
            1000,
        );

        adapter_props
            .set("Powered", MessageItem::Bool(true))
            .unwrap();

        let advertisement_object_path = format!("{}{}", PATH_BASE, 0);

        let factory = Factory::new_fn::<()>();

        let object_path = factory
            .object_path(advertisement_object_path.clone(), ())
            .add(
                factory
                    .interface(LE_ADVERTISEMENT_IFACE, ())
                    .add_m(factory.method("Release", (), |method_info| {
                        Ok(vec![method_info.msg.method_return()])
                    })),
            )
            .add(
                factory.interface(DBUS_PROP_IFACE, ()).add_m(
                    factory
                        .method("GetAll", (), |method_info| {
                            let mut props: HashMap<&str, Variant<&str>> = HashMap::new();
                            props.insert("Type", Variant("peripheral"));
                            props.insert("LocalName", Variant("hello"));
                            Ok(vec![method_info.msg.method_return().append1(props)])
                        })
                        .in_arg("s")
                        .out_arg("a{sv}"),
                ),
            );
        let tree = factory.tree(()).add(object_path);

        tree.set_registered(&connection, true).unwrap();
        connection.add_handler(tree);

        Peripheral {
            connection,
            adapter_object_path,
            factory,
            advertisement_object_path,
        }
    }

    pub fn is_powered_on(self: &Self) -> bool {
        let adapter_props = Props::new(
            &self.connection,
            BLUEZ_SERVICE_NAME,
            &self.adapter_object_path,
            ADAPTER_IFACE,
            1000,
        );
        adapter_props
            .get("Powered")
            .unwrap()
            .inner::<bool>()
            .unwrap()
    }

    pub fn start_advertising(self: &Self, _name: &str, _uuids: &[Uuid]) {
        let mut message = Message::new_method_call(
            BLUEZ_SERVICE_NAME,
            &self.adapter_object_path,
            LE_ADVERTISING_MANAGER_IFACE,
            "RegisterAdvertisement",
        )
        .unwrap();

        let path = Path::new(self.advertisement_object_path.clone()).unwrap();
        let options: HashMap<String, Variant<Box<RefArg>>> = HashMap::new();
        message = message.append2(path, options);

        let done: std::rc::Rc<std::cell::Cell<bool>> = Default::default();
        let done2 = done.clone();
        self.connection.add_handler(
            self.connection
                .send_with_reply(message, move |_| {
                    done2.set(true);
                })
                .unwrap(),
        );
        while !done.get() {
            self.connection.incoming(100).next();
        }
    }

    pub fn is_advertising(self: &Self) -> bool {
        true
    }
}

impl Default for Peripheral {
    fn default() -> Self {
        Peripheral::new()
    }
}
