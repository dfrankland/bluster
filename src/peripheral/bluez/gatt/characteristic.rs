use dbus::{
    arg::{RefArg, Variant},
    tree::{Access, MethodErr},
    Message, MessageItem, Path,
};
use dbus_tokio::tree::AFactory;
use futures::{prelude::*, sync::{oneshot, mpsc}};
use std::{collections::HashMap, sync::Arc};

use super::{
    flags::Flags,
    super::{
        Connection,
        common,
        constants::{BLUEZ_SERVICE_NAME, DBUS_PROPERTIES_IFACE, BLUEZ_ERROR_FAILED, BLUEZ_ERROR_NOTSUPPORTED, GATT_CHARACTERISTIC_IFACE},
    },
};
use crate::{gatt, Error};

#[derive(Debug, Clone)]
pub struct Characteristic {
    pub object_path: Path<'static>,
}

impl Characteristic {
    pub fn new(
        connection: Arc<Connection>,
        tree: &mut common::Tree,
        characteristic: &Arc<gatt::characteristic::Characteristic>,
        service: &Path<'static>,
    ) -> Result<Self, Error> {
        let factory = AFactory::new_afn::<()>();

        let mut object_path = factory
            .object_path(format!("{}/characteristic{:04}", service, 0), ());
        let object_path_name = object_path.get_name().clone();

        let read_value = characteristic.properties.read.clone();
        let write_value = characteristic.properties.write.clone();

        let notify_value_start = characteristic.properties.notify.clone();
        let notify_value_stop = notify_value_start.clone();

        // TODO: Indicate too
        // let indicate_value_start = characteristic.properties.indicate.clone();
        // let indicate_value_stop = indicate_value_start.clone();

        let flags_value = characteristic.properties.flags().clone();
        let uuid_value = characteristic.uuid.to_string();
        let service_value = service.clone();
        let initial_value = characteristic.value.clone().unwrap_or_else(Vec::new);

        let mut gatt_characteristic = factory
            .interface(GATT_CHARACTERISTIC_IFACE, ())
            .add_m(factory.amethod("ReadValue", (), move |method_info| {
                let offset = method_info
                    .msg
                    .get1::<HashMap<String, Variant<MessageItem>>>()
                    .unwrap_or_else(HashMap::new)
                    .get("offset")
                    .and_then(|offset| offset.as_u64())
                    .unwrap_or(0) as u16;
                let mret = method_info.msg.method_return();

                read_value
                    .clone()
                    .ok_or_else(|| MethodErr::from((BLUEZ_ERROR_NOTSUPPORTED, "")))
                    .into_future()
                    .and_then(move |event_sender| {
                        let (sender, receiver) = oneshot::channel();
                        event_sender
                            .sender()
                            .send(gatt::event::Event::ReadRequest(gatt::event::ReadRequest {
                                offset,
                                response: sender,
                            }))
                            .map_err(move |_| MethodErr::from((BLUEZ_ERROR_FAILED, "")))
                            .and_then(move |_| {
                                receiver.map_err(move |_| MethodErr::from((BLUEZ_ERROR_FAILED, "")))
                            })
                    })
                    .and_then(move |response| match response {
                        gatt::event::Response::Success(value) => Ok(vec![mret.append1(value)]),
                        _ => Err(MethodErr::from((BLUEZ_ERROR_FAILED, ""))),
                    })
            }))
            .add_m(factory.amethod("WriteValue", (), move |method_info| {
                let (data, flags) = method_info
                    .msg
                    .get2::<Vec<u8>, HashMap<String, Variant<MessageItem>>>();
                let data = data.unwrap_or_else(Vec::new);
                let offset = flags
                    .unwrap_or_else(HashMap::new)
                    .get("offset")
                    .and_then(|offset| offset.as_u64())
                    .unwrap_or(0) as u16;
                let mret = method_info.msg.method_return();

                write_value
                    .clone()
                    .ok_or_else(|| MethodErr::from((BLUEZ_ERROR_NOTSUPPORTED, "")))
                    .into_future()
                    .and_then(move |event_sender| {
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
                            .map_err(|_| MethodErr::from((BLUEZ_ERROR_FAILED, "")))
                            .and_then(move |_| {
                                receiver.map_err(move |_| MethodErr::from((BLUEZ_ERROR_FAILED, "")))
                            })
                    })
                    .and_then(move |response| match response {
                        gatt::event::Response::Success(value) => Ok(vec![mret.append1(value)]),
                        _ => Err(MethodErr::from((BLUEZ_ERROR_FAILED, ""))),
                    })
            }))
            .add_m(factory.amethod("StartNotify", (), move |_| {
                let (sender, receiver) = mpsc::channel(1);
                let notify_subscribe = gatt::event::NotifySubscribe {
                    notification: sender,
                };
                let connection = connection.clone();
                let object_path_name = object_path_name.clone();
                notify_value_start
                    .clone()
                    .ok_or_else(|| MethodErr::from((BLUEZ_ERROR_NOTSUPPORTED, "")))
                    .into_future()
                    .and_then(move |event_sender| {
                        event_sender
                            .send(gatt::event::Event::NotifySubscribe(notify_subscribe))
                            .map_err(|_| {
                                MethodErr::from((BLUEZ_ERROR_FAILED, ""))
                            })
                    })
                    .and_then(move |_| {
                        receiver.for_each(move |notification| {
                            let message = Message::new_method_call(
                                BLUEZ_SERVICE_NAME,
                                object_path_name.clone(),
                                DBUS_PROPERTIES_IFACE,
                                "Set",
                            )
                            .unwrap()
                            .append3(
                                GATT_CHARACTERISTIC_IFACE,
                                "Value",
                                notification,
                            );

                            connection
                                .clone()
                                .default
                                .method_call(message)
                                .unwrap()
                                .then(|_| Ok(()))
                        })
                        .map_err(|_| {
                            MethodErr::from((BLUEZ_ERROR_FAILED, ""))
                        })
                    })
                    .and_then(|_| Ok(vec![]))
            }))
            .add_m(factory.amethod("StopNotify", (), move |_| {
                notify_value_stop
                    .clone()
                    .ok_or_else(|| MethodErr::from((BLUEZ_ERROR_NOTSUPPORTED, "")))
                    .into_future()
                    .and_then(move |event_sender| {
                        event_sender
                            .send(gatt::event::Event::NotifyUnsubscribe)
                            .map_err(|_| {
                                MethodErr::from((BLUEZ_ERROR_FAILED, ""))
                            })
                    })
                    .and_then(|_| Ok(vec![]))
            }))
            .add_p(
                factory
                    .property::<&str, _>("UUID", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        i.append(&uuid_value);
                        Ok(())
                    }),
            )
            .add_p(
                factory
                    .property::<Path<'static>, _>("Service", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        i.append(&service_value);
                        Ok(())
                    }),
            )
            .add_p(
                factory
                    .property::<&[&str], _>("Flags", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        i.append(&flags_value);
                        Ok(())
                    }),
            );

        if characteristic.properties.is_read_only() && characteristic.value.is_some() {
            gatt_characteristic = gatt_characteristic.add_p(
                factory
                    .property::<&[u8], _>("Value", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        i.append(&initial_value);
                        Ok(())
                    }),
            );
        }

        object_path = object_path
            .add(gatt_characteristic)
            .introspectable()
            .object_manager();

        let path = object_path.get_name().clone();

        tree.insert(object_path);

        Ok(Characteristic { object_path: path })
    }
}
