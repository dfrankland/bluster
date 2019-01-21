#[cfg(any(target_os = "macos", target_os = "ios"))]
mod corebluetooth;
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use self::corebluetooth::Peripheral;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod bluez;
#[cfg(any(target_os = "linux", target_os = "android"))]
pub use self::bluez::Peripheral;

#[cfg(any(target_os = "windows", target_os = "freebsd"))]
mod usb;
#[cfg(any(target_os = "windows", target_os = "freebsd"))]
pub use self::usb::Peripheral;

// TODO: Add struct / traits to implement for each OS
//
// pub enum BindingsEvent {
//     StateChange,
//     Platform,
//     AddressChange,
//     AdvertisingStart,
//     AdvertisingStop,
//     ServicesSet,
//     Accept,
//     MtuChange,
//     Disconnect,
//     RssiUpdate,
// }
//
// #[derive(Debug, Clone)]
// pub enum State {
//     Unknown,
//     Resetting,
//     Unsupported,
//     Unauthorized,
//     PoweredOff,
//     PoweredOn,
// }
//
// #[derive(Debug, Clone)]
// pub struct Ble {
//     initialized: bool,
//     platform: String, // TODO: Make this an enum?
//     state: State,
//     address: String, // TODO: Make this a struct or something?
//     rssi: u8,
//     mtu: u8,
// }
