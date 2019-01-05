use futures::{future, prelude::*, sync::mpsc::channel};
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

const ADVERTISING_NAME: &str = "hello";
const ADVERTISING_TIMEOUT: u64 = 60;

#[test]
fn it_advertises_gatt() {
    let (sender, receiver) = channel(1);

    let mut characteristics: HashSet<Characteristic> = HashSet::new();
    characteristics.insert(Characteristic::new(
        Uuid::from_sdp_short_uuid(0x2A3D as u16),
        characteristic::Properties::new(
            Some(characteristic::Secure::Insecure(sender.clone())),
            None,
            Some(sender.clone()),
            None,
        ),
        None,
        HashSet::<Descriptor>::new(),
    ));

    let mut runtime = Runtime::new().unwrap();

    let peripheral_future = Peripheral::new(&mut runtime);
    let peripheral = Arc::new(runtime.block_on(peripheral_future).unwrap());

    peripheral
        .add_service(
            &mut runtime,
            &Service::new(
                Uuid::from_sdp_short_uuid(0x1234 as u16),
                true,
                characteristics,
            ),
        )
        .unwrap();

    let advertisement = future::loop_fn(Arc::clone(&peripheral), |peripheral| {
        peripheral.is_powered().and_then(move |is_powered| {
            if is_powered {
                println!("Peripheral powered on");
                Ok(future::Loop::Break(peripheral))
            } else {
                Ok(future::Loop::Continue(peripheral))
            }
        })
    })
    .and_then(|peripheral| {
        let peripheral2 = Arc::clone(&peripheral);
        peripheral
            .start_advertising(ADVERTISING_NAME, &[])
            .and_then(move |advertising_stream| Ok((advertising_stream, peripheral2)))
    })
    .and_then(|(advertising_stream, peripheral)| {
        let handled_advertising_stream = receiver
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
                    Event::NotifySubscribe(notify_subscribe) => {
                        println!("GATT server got a notify subscription!");
                        thread::spawn(move || {
                            let mut count = 0;
                            loop {
                                count += 1;
                                println!("GATT server notifying \"hi {}\"!", count);
                                notify_subscribe
                                    .clone()
                                    .notification
                                    .try_send(format!("hi {}", count).into())
                                    .unwrap();
                                thread::sleep(Duration::from_secs(2));
                            }
                        });
                    }
                    _ => panic!("Got some other event!"),
                };
            })
            .map_err(bluster::Error::from)
            .select(advertising_stream)
            .skip_while(|_| Ok(true));

        let advertising_timeout = Timeout::new(
            handled_advertising_stream,
            Duration::from_secs(ADVERTISING_TIMEOUT),
        )
        .into_future()
        .then(|_| Ok(()));

        let advertising_check = future::loop_fn(Arc::clone(&peripheral), move |peripheral| {
            peripheral.is_advertising().and_then(move |is_advertising| {
                if is_advertising {
                    println!("Peripheral started advertising \"{}\"", ADVERTISING_NAME);
                    Ok(future::Loop::Break(peripheral))
                } else {
                    Ok(future::Loop::Continue(peripheral))
                }
            })
        })
        .fuse();

        let peripheral2 = Arc::clone(&peripheral);
        advertising_check
            .join(advertising_timeout)
            .and_then(move |_| Ok(peripheral2))
    })
    .and_then(|peripheral| {
        let stop_advertising = peripheral.stop_advertising();

        let advertising_check = future::loop_fn(Arc::clone(&peripheral), |peripheral| {
            peripheral.is_advertising().and_then(move |is_advertising| {
                if !is_advertising {
                    println!("Peripheral stopped advertising");
                    Ok(future::Loop::Break(()))
                } else {
                    Ok(future::Loop::Continue(peripheral))
                }
            })
        })
        .fuse();

        advertising_check.join(stop_advertising)
    });

    runtime.block_on(advertisement).unwrap();
}
