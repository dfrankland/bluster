use dbus::{
    arg::{RefArg, Variant},
    channel::Sender,
    nonblock::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged,
    tree::MethodErr,
    Message, Path,
};
use futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use std::{collections::HashMap, sync::Arc};

use super::{
    super::{
        common,
        common::GattDataType,
        constants::{BLUEZ_ERROR_FAILED, BLUEZ_ERROR_NOTSUPPORTED, GATT_CHARACTERISTIC_IFACE},
        Connection,
    },
    flags::Flags,
};
use crate::{gatt, Error};

type OptionsMap = HashMap<String, Variant<Box<dyn RefArg>>>;

#[derive(Debug, Clone)]
pub struct Characteristic {
    pub object_path: Path<'static>,
}

impl Characteristic {
    pub fn new(
        connection: &Arc<Connection>,
        tree: &mut common::Tree,
        characteristic: &Arc<gatt::characteristic::Characteristic>,
        service: &Path<'static>,
        index: u64,
    ) -> Result<Self, Error> {
        let object_path: Path = format!("{}/characteristic{:04}", service, index).into();
        let object_path_data = common::GattDataType::Characteristic(Arc::clone(characteristic));
        // Setup a channel for notifications
        let (message_sender, message_receiver) = mpsc::channel(1);
        {
            let object_path = object_path.clone();
            let connection = Arc::clone(connection);
            tokio::spawn(
                message_receiver
                    .map(move |notification: Vec<u8>| {
                        // For notifications, BlueZ wants a PropertiesChanged
                        // signal on the optional `Value` property. It doesn't
                        // require that the property actually exists.
                        let mut props = HashMap::new();
                        props.insert("Value".to_owned(), Variant(Box::new(notification) as _));
                        let signal = PropertiesPropertiesChanged {
                            interface_name: GATT_CHARACTERISTIC_IFACE.to_string(),
                            changed_properties: props,
                            invalidated_properties: Vec::new(),
                        };
                        let mut signal_message = Message::signal(
                            &object_path,
                            &"org.freedesktop.DBus.Properties".into(),
                            &"PropertiesChanged".into(),
                        );
                        signal_message.append_all(signal);
                        connection.default.send(signal_message).ok();
                    })
                    .collect::<()>(),
            );
        }

        let iface_token = tree.register::<GattDataType, _, _>(GATT_CHARACTERISTIC_IFACE, |b| {
            let message_sender = message_sender.clone();
            b.method_with_cr_async(
                "ReadValue",
                ("options",),
                ("value",),
                |mut ctx, cr, (options,): (OptionsMap,)| {
                    let offset = options.get("offset").and_then(RefArg::as_u64).unwrap_or(0) as u16;
                    let characteristic = cr
                        .data_mut::<GattDataType>(ctx.path())
                        .unwrap()
                        .get_characteristic();
                    async move {
                        let event_sender = characteristic
                            .properties
                            .read
                            .clone()
                            .ok_or_else(|| MethodErr::from((BLUEZ_ERROR_NOTSUPPORTED, "")))?;
                        let (sender, receiver) = oneshot::channel();
                        event_sender
                            .sender()
                            .send(gatt::event::Event::ReadRequest(gatt::event::ReadRequest {
                                offset,
                                response: sender,
                            }))
                            .await
                            .map_err(|_| MethodErr::from((BLUEZ_ERROR_FAILED, "")))?;
                        receiver
                            .await
                            .map_err(|_| MethodErr::from((BLUEZ_ERROR_FAILED, "")))
                            .and_then(|resp| match resp {
                                gatt::event::Response::Success(value) => Ok((value,)),
                                _ => Err(MethodErr::from((BLUEZ_ERROR_FAILED, ""))),
                            })
                    }
                    .map(move |result| ctx.reply(result))
                },
            );
            b.method_with_cr_async(
                "WriteValue",
                ("data", "options"),
                ("value",),
                |mut ctx, cr, (data, options): (Vec<u8>, OptionsMap)| {
                    let offset = options.get("offset").and_then(RefArg::as_u64).unwrap_or(0) as u16;
                    let characteristic = cr
                        .data_mut::<GattDataType>(ctx.path())
                        .unwrap()
                        .get_characteristic();
                    async move {
                        let event_sender = characteristic
                            .properties
                            .write
                            .clone()
                            .ok_or_else(|| MethodErr::from((BLUEZ_ERROR_NOTSUPPORTED, "")))?;
                        let (sender, receiver) = oneshot::channel();
                        event_sender
                            .sender()
                            .send(gatt::event::Event::WriteRequest(
                                gatt::event::WriteRequest {
                                    data,
                                    offset,
                                    without_response: false,
                                    response: sender,
                                },
                            ))
                            .await
                            .map_err(|_| MethodErr::from((BLUEZ_ERROR_FAILED, "")))?;
                        receiver
                            .await
                            .map_err(|_| MethodErr::from((BLUEZ_ERROR_FAILED, "")))
                            .and_then(|resp| match resp {
                                gatt::event::Response::Success(value) => Ok((value,)),
                                _ => Err(MethodErr::from((BLUEZ_ERROR_FAILED, ""))),
                            })
                    }
                    .map(move |result| ctx.reply(result))
                },
            );
            b.method_with_cr_async("StartNotify", (), (), move |mut ctx, cr, ()| {
                let characteristic = cr
                    .data_mut::<GattDataType>(ctx.path())
                    .unwrap()
                    .get_characteristic();
                let message_sender = message_sender.clone();
                async move {
                    let (sender, mut receiver) = mpsc::channel(1);
                    let notify_subscribe = gatt::event::NotifySubscribe {
                        notification: sender,
                    };
                    tokio::spawn(async move {
                        while let Some(notification) = receiver.next().await {
                            let mut message_sender = message_sender.clone();
                            let _ = message_sender.send(notification).await;
                        }
                    });
                    let mut event_sender = characteristic
                        .properties
                        .notify
                        .clone()
                        .or_else(|| characteristic.properties.indicate.clone())
                        .ok_or_else(|| MethodErr::from((BLUEZ_ERROR_NOTSUPPORTED, "")))?;
                    event_sender
                        .send(gatt::event::Event::NotifySubscribe(notify_subscribe))
                        .await
                        .map_err(|_| MethodErr::from((BLUEZ_ERROR_FAILED, "")))
                        .map(|_| ())
                }
                .map(move |result| ctx.reply(result))
            });
            b.method_with_cr_async("StopNotify", (), (), |mut ctx, cr, ()| {
                let characteristic = cr
                    .data_mut::<GattDataType>(ctx.path())
                    .unwrap()
                    .get_characteristic();
                async move {
                    let mut event_sender = characteristic
                        .properties
                        .notify
                        .clone()
                        .or_else(|| characteristic.properties.indicate.clone())
                        .ok_or_else(|| MethodErr::from((BLUEZ_ERROR_NOTSUPPORTED, "")))?;
                    event_sender
                        .send(gatt::event::Event::NotifyUnsubscribe)
                        .await
                        .map_err(|_| MethodErr::from((BLUEZ_ERROR_FAILED, "")))
                        .map(|_| ())
                }
                .map(move |result| ctx.reply(result))
            });
            b.property("UUID")
                .get(|_ctx, data| Ok(data.get_characteristic().uuid.to_string()));
            let service = service.clone();
            b.property("Service")
                .get(move |_ctx, _data| Ok(service.clone()));
            b.property("Flags")
                .get(move |_ctx, data| Ok(data.get_characteristic().properties.flags()));
        });

        tree.insert(object_path.clone(), &[iface_token], object_path_data);

        Ok(Characteristic { object_path })
    }
}
