use dbus::{stdintf::org_freedesktop_dbus::ObjectManager, Connection, MessageItem, Path, Props};

use super::constants::{ADAPTER_IFACE, BLUEZ_SERVICE_NAME, LE_ADVERTISING_MANAGER_IFACE};

#[derive(Debug, Clone)]
pub struct Adapter {
    pub object_path: Path<'static>,
}

impl Adapter {
    fn find_adapter(connection: &Connection) -> Result<Path<'static>, dbus::Error> {
        let connection_path = connection.with_path(BLUEZ_SERVICE_NAME, "/", 5000);
        let managed_objects = connection_path.get_managed_objects()?;
        for (path, props) in managed_objects.iter() {
            if props.contains_key(LE_ADVERTISING_MANAGER_IFACE) {
                return Ok(path.clone());
            }
        }

        panic!("LEAdvertisingManager1 interface not found");
    }

    pub fn new(connection: &Connection) -> Result<Self, dbus::Error> {
        let object_path = Adapter::find_adapter(connection)?;
        let adapter = Adapter { object_path };
        Ok(adapter)
    }

    pub fn powered_on(self: &Self, connection: &Connection, on: bool) -> Result<(), dbus::Error> {
        Props::new(
            connection,
            BLUEZ_SERVICE_NAME,
            &self.object_path,
            ADAPTER_IFACE,
            1000,
        )
        .set("Powered", MessageItem::Bool(on))
    }

    pub fn is_powered_on(self: &Self, connection: &Connection) -> Result<bool, dbus::Error> {
        let props = Props::new(
            connection,
            BLUEZ_SERVICE_NAME,
            &self.object_path,
            ADAPTER_IFACE,
            1000,
        );
        let powered = props.get("Powered")?;
        Ok(powered.inner::<bool>().unwrap())
    }
}
