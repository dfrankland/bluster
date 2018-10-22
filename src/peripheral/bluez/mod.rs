use std::{collections::HashMap, boxed::Box};
use dbus::{Connection, BusType, Message, stdintf::org_freedesktop_dbus::ObjectManager, Props, MessageItem, tree::{Factory, Tree, MTFn, MethodErr}, Path, arg::{Variant, RefArg}};
use uuid::Uuid;

// const DBUS_OM_IFACE: &str = "org.freedesktop.DBus.ObjectManager"
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
    // tree: Tree<MTFn, ()>,
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
        let adapter_props = Props::new(&connection, BLUEZ_SERVICE_NAME, &adapter_object_path, ADAPTER_IFACE, 1000);

        adapter_props.set("Powered", MessageItem::Bool(true)).unwrap();

        let advertisement_object_path = format!("{}{}", PATH_BASE, 0);

        let factory = Factory::new_fn::<()>();
        let tree = factory
            .tree(())
            .add(
                factory
                    .object_path(advertisement_object_path.clone(), ())
                    .add(
                        factory
                            .interface(LE_ADVERTISEMENT_IFACE, ())
                            .add_m(
                                factory
                                    .method("Release", (), |method_info| {
                                        println!("Release method called!");
                                        Ok(vec![method_info.msg.method_return()])
                                    })
                            )
                    )
                    .add(
                        factory
                            .interface(DBUS_PROP_IFACE, ())
                            .add_m(
                                factory
                                    .method("GetAll", (), |method_info| {
                                        println!("GetAll method called!");
                                        let interface_name: &str = &method_info.iface.get_name();
                                        if interface_name != LE_ADVERTISEMENT_IFACE {
                                            return Err(MethodErr::invalid_arg(method_info));
                                        }
                                        println!("Returning props to GetAll!");
                                        let mut props = HashMap::new();
                                        props.insert("Type", "peripheral");
                                        props.insert("LocalName", "hello");
                                        Ok(vec![method_info.msg.method_return().append1(props)])
                                    })
                                    .in_arg("s")
                                    .out_arg("a{sv}")
                            )
                    )
                    .introspectable()
                    .object_manager()
            );

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
        let adapter_props = Props::new(&self.connection, BLUEZ_SERVICE_NAME, &self.adapter_object_path, ADAPTER_IFACE, 1000);
        adapter_props.get("Powered").unwrap().inner::<bool>().unwrap()
    }

    pub fn start_advertising(self: &Self, _name: &str, _uuids: &[Uuid]) {
        println!("Using HCI path {}", &self.adapter_object_path);
        let mut message = Message::new_method_call(
            BLUEZ_SERVICE_NAME,
            &self.adapter_object_path,
            LE_ADVERTISING_MANAGER_IFACE,
            "RegisterAdvertisement",
        ).unwrap();

        println!("Using advertisment path {}", &self.advertisement_object_path);
        let path = Path::new(self.advertisement_object_path.clone()).unwrap();
        let options: HashMap<String, Variant<Box<RefArg>>> = HashMap::new();
        message = message.append2(path, options);

        let reply = self.connection.send_with_reply_and_block(message, 10000);

        println!("{:?}", reply);
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
