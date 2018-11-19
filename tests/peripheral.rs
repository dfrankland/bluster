use bluster::{
    gatt::{
        characteristic,
        characteristic::Characteristic,
        descriptor::Descriptor,
        event::{Event, Response},
        service::Service,
    },
    Peripheral, SdpShortUuid,
};
use futures::{executor::spawn, sync::mpsc::channel};
use std::{collections::HashSet, thread, time};
use uuid::Uuid;

const ITERATIONS: u64 = 60;
const SLEEP_SECS: u64 = 1;

#[test]
fn it_advertises_gatt() {
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
}
