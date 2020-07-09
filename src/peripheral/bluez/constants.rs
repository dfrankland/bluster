use std::time::Duration;

pub const DBUS_PROPERTIES_IFACE: &str = "org.freedesktop.DBus.Properties";
pub const DBUS_OBJECTMANAGER_IFACE: &str = "org.freedesktop.DBus.ObjectManager";

pub const BLUEZ_SERVICE_NAME: &str = "org.bluez";

pub const ADAPTER_IFACE: &str = "org.bluez.Adapter1";

pub const LE_ADVERTISING_MANAGER_IFACE: &str = "org.bluez.LEAdvertisingManager1";
pub const LE_ADVERTISEMENT_IFACE: &str = "org.bluez.LEAdvertisement1";

pub const GATT_SERVICE_IFACE: &str = "org.bluez.GattService1";
pub const GATT_CHARACTERISTIC_IFACE: &str = "org.bluez.GattCharacteristic1";
pub const GATT_DESCRIPTOR_IFACE: &str = "org.bluez.GattDescriptor1";
pub const GATT_GATT_MANAGER_IFACE: &str = "org.bluez.GattManager1";

pub const BLUEZ_ERROR_FAILED: &str = "org.bluez.Error.Failed";
// pub const BLUEZ_ERROR_INPROGRESS: &str = "org.bluez.Error.InProgress";
// pub const BLUEZ_ERROR_NOTPERMITTED: &str = "org.bluez.Error.NotPermitted";
// pub const BLUEZ_ERROR_NOTAUTHORIZED: &str = "org.bluez.Error.NotAuthorized";
// pub const BLUEZ_ERROR_INVALIDOFFSET: &str = "org.bluez.Error.InvalidOffset";
pub const BLUEZ_ERROR_NOTSUPPORTED: &str = "org.bluez.Error.NotSupported";

pub const PATH_BASE: &str = "/org/bluez/example";

pub const BLUEZ_DBUS_TIMEOUT: Duration = Duration::from_secs(30);
