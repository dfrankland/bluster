use dbus::{
    arg::{RefArg, Variant},
    stdintf::org_freedesktop_dbus::ObjectManager,
    tree::{Factory, MTFn},
    BusType, Connection, Message, MessageItem, MessageItemArray, Path, Props, Signature,
};
use std::{borrow::Borrow, boxed::Box, collections::HashMap, sync::Arc};
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

        Peripheral {
            connection,
            adapter_object_path,
            factory: Factory::new_fn::<()>(),
            advertisement_object_path: format!("{}{}", PATH_BASE, 0),
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

    pub fn start_advertising(self: &Self, name: &str, uuids: &[Uuid]) {
        let name = Arc::new(name.to_string());
        let uuids = Arc::new(
            uuids
                .to_vec()
                .iter()
                .map(|uuid| uuid.to_string())
                .collect::<Vec<String>>(),
        );

        // Create advertisment object
        let object_path = self
            .factory
            .object_path(self.advertisement_object_path.clone(), ())
            .add(
                self.factory
                    .interface(LE_ADVERTISEMENT_IFACE, ())
                    .add_m(self.factory.method("Release", (), |method_info| {
                        Ok(vec![method_info.msg.method_return()])
                    })),
            )
            .add(
                self.factory.interface(DBUS_PROP_IFACE, ()).add_m(
                    self.factory
                        .method("GetAll", (), move |method_info| {
                            let local_name = name.clone();
                            let local_name: &String = local_name.borrow();

                            let service_uuids = uuids.clone();
                            let service_uuids: &Vec<String> = service_uuids.borrow();

                            let mut props = HashMap::new();

                            props.insert(
                                "Type",
                                Variant(MessageItem::Str("peripheral".to_string())),
                            );
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
                ),
            );

        // Register advertisement object to DBus
        let tree = self.factory.tree(()).add(object_path);
        tree.set_registered(&self.connection, true).unwrap();
        self.connection.add_handler(tree);

        // Create message to register advertisement with Bluez
        let mut message = Message::new_method_call(
            BLUEZ_SERVICE_NAME,
            &self.adapter_object_path,
            LE_ADVERTISING_MANAGER_IFACE,
            "RegisterAdvertisement",
        )
        .unwrap();
        message = message.append2(
            Path::new(self.advertisement_object_path.clone()).unwrap(),
            HashMap::<String, Variant<Box<RefArg>>>::new(),
        );

        // Send message
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
