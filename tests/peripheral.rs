use futures::{prelude::*, sync::mpsc::channel};
use std::{collections::HashSet, sync::Arc, thread, time::Duration};
use tokio::{runtime::current_thread::Runtime, timer::Timeout};
use uuid::Uuid;

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

#[test]
fn it_advertises_gatt() {
    let (sender, receiver) = channel(1);

    thread::spawn(move || {
        let handler = receiver
            .map(|event| {
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
            })
            .collect();
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(handler).unwrap();
        runtime.run().unwrap();
    });

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

    let mut runtime = Runtime::new().unwrap();
    let peripheral = Arc::new(Peripheral::new(&mut runtime).unwrap());

    peripheral
        .add_service(&Service::new(
            Uuid::from_sdp_short_uuid(0x1234 as u16),
            true,
            characteristics,
        ))
        .unwrap();

    let advertisement = peripheral
        .is_powered_on()
        .unwrap()
        .map_err(|_| ())
        .and_then(|_| {
            println!("Peripheral powered on");
            peripheral
                .start_advertising("hello", &[])
                .unwrap()
                .map_err(|_| ())
        })
        .and_then(|stream| {
            println!("Peripheral started advertising \"hello\"");
            Timeout::new(stream, Duration::from_secs(60))
                .into_future()
                .then(|_| Ok(()))
        })
        .and_then(|_| peripheral.stop_advertising().map_err(|_| ()))
        .and_then(|_| {
            println!("Peripheral stopped advertising");
            Ok(())
        });

    runtime.block_on(advertisement).unwrap();
    runtime.run().unwrap();
}
