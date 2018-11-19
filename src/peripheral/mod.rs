#[cfg(any(target_os = "macos", target_os = "ios"))]
mod corebluetooth;
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use corebluetooth::Peripheral;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod bluez;
#[cfg(any(target_os = "linux", target_os = "android"))]
pub use bluez::Peripheral;

#[cfg(any(target_os = "windows", target_os = "freebsd"))]
mod usb;
#[cfg(any(target_os = "windows", target_os = "freebsd"))]
pub use usb::Peripheral;

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

#[test]
fn it_connects_changes_state() -> Result<(), crate::Error> {
    use crate::{
        gatt::{
            characteristic,
            characteristic::Characteristic,
            descriptor::Descriptor,
            event::{Event, Response},
            service::Service,
        },
        SdpShortUuid,
    };
    use futures::{sync::mpsc::channel, executor::spawn};
    use std::{collections::HashSet, thread, time};
    use uuid::Uuid;

    const ITERATIONS: u64 = 60;
    const SLEEP_SECS: u64 = 1;

    let (sender, r) = channel(1);
    let mut receiver = spawn(r);

    thread::spawn(move || {
        let mut characteristics: HashSet<Characteristic> = HashSet::new();
        characteristics.insert(Characteristic::new(
            Uuid::from_sdp_short_uuid(0x2A3D as u16),
            characteristic::Properties::new(
                Some(characteristic::Secure::Insecure(sender)),
                None,
                None,
                None,
            ),
            None,
            HashSet::<Descriptor>::new(),
        ));

        let mut peripheral = Peripheral::new().unwrap();
        peripheral
            .add_service(&Service::new(
                Uuid::from_sdp_short_uuid(0x1234 as u16),
                true,
                characteristics,
            ))
            .unwrap();

        while !peripheral.is_powered_on().unwrap() {}

        println!("Peripheral powered on");

        let mut check = peripheral.start_advertising("hello", &[]).unwrap();

        println!("Peripheral started advertising \"hello\"");

        for _ in 0..ITERATIONS {
            check.next();
            thread::sleep(time::Duration::from_secs(SLEEP_SECS));
        }

        peripheral.stop_advertising().unwrap();

        while !peripheral.is_advertising().unwrap() {}

        println!("Peripheral stopped");
    });

    for _ in 0..ITERATIONS {
        if let Some(result) = receiver.wait_stream() {
            if let Ok(event) = result {
                match event {
                    Event::ReadRequest(read_request) => {
                        println!(
                            "GATT server got a read request with offset {}!",
                            read_request.offset
                        );
                        read_request
                            .response
                            .send(Response::Success("hi".into()))
                            .unwrap();
                        println!("GATT server responded with \"hi\"");
                    }
                    _ => panic!("Got some other event!"),
                };
            }
        }
        thread::sleep(time::Duration::from_secs(SLEEP_SECS));
    }

    Ok(())
}
