use dbus::{
    arg::{Iter, RefArg, Variant},
    tree::{Access, EmitsChangedSignal, MethodErr, PropInfo},
    Message, MessageItem, Path,
};
use dbus_tokio::tree::{AFactory, ATree};
use futures::{
    future,
    prelude::*,
    sync::{mpsc, oneshot},
};
use std::{
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
};
use tokio::runtime::current_thread::{block_on_all, Runtime};

use super::{
    super::{
        common,
        constants::{
            BLUEZ_ERROR_FAILED, BLUEZ_ERROR_NOTSUPPORTED, DBUS_PROPERTIES_IFACE,
            GATT_CHARACTERISTIC_IFACE,
        },
        Connection,
    },
    flags::Flags,
};
use crate::{gatt, Error};

#[derive(Debug, Clone)]
pub struct Characteristic {
    pub object_path: Path<'static>,
}

impl Characteristic {
    pub fn new(
        runtime: &mut Runtime,
        connection: &Arc<Connection>,
        tree: &mut common::Tree,
        characteristic: &Arc<gatt::characteristic::Characteristic>,
        service: &Path<'static>,
    ) -> Result<Self, Error> {
        let factory = AFactory::new_afn::<common::TData>();

        // Setup value property for read / write by other methods
        let value = Arc::new(Mutex::new(characteristic.value.clone()));
        let value_property = {
            let property = factory
                .property::<&[u8], _>("Value", ())
                .emits_changed(EmitsChangedSignal::True)
                .access({
                    let is_read_only_value =
                        characteristic.properties.is_read_only() && characteristic.value.is_some();
                    if is_read_only_value {
                        Access::Read
                    } else {
                        Access::ReadWrite
                    }
                })
                .on_get({
                    let on_get_value = Arc::clone(&value);
                    move |i, _| {
                        let value = on_get_value
                            .lock()
                            .unwrap()
                            .clone()
                            .unwrap_or_else(Vec::new);
                        i.append(value);
                        Ok(())
                    }
                })
                .on_set({
                    let on_set_value = Arc::clone(&value);
                    move |i, _| {
                        let value = i.read()?;
                        on_set_value.lock().unwrap().replace(value);
                        Ok(())
                    }
                });
            Arc::new(property)
        };

        // Setup a channel for notifications
        let (message_sender, message_receiver) = mpsc::channel(1);
        {
            let value_property = Arc::clone(&value_property);
            let connection = Arc::clone(connection);
            let object_path = factory.object_path(
                format!("{}/characteristic{:04}", service, 0),
                common::GattDataType::Characteristic(Arc::clone(characteristic)),
            );
            runtime.spawn(
                message_receiver
                    .map(move |notification: Vec<u8>| {
                        let message = Message::new_method_call(
                            connection.fallback.unique_name(),
                            object_path.get_name(),
                            DBUS_PROPERTIES_IFACE,
                            "Set",
                        )
                        .unwrap()
                        .append1(MessageItem::Variant(Box::new(
                            MessageItem::new_array(
                                notification
                                    .iter()
                                    .map(|b| MessageItem::Byte(*b))
                                    .collect::<Vec<MessageItem>>(),
                            )
                            .unwrap(),
                        )));

                        let factory = AFactory::new_afn::<common::TData>();
                        let prop_info = PropInfo {
                            msg: &message,
                            method: &factory.method("Set", (), |_| Ok(vec![])),
                            prop: &Arc::clone(&value_property),
                            iface: &factory.interface(GATT_CHARACTERISTIC_IFACE, ()),
                            path: &object_path,
                            tree: &factory.tree(ATree::new()),
                        };
                        let set_result =
                            value_property.set_as_variant(&mut Iter::new(&message), &prop_info);
                        if let Ok(emits_changed) = set_result {
                            if let Some(emits_changed_message) = emits_changed {
                                return future::Either::A(
                                    Rc::clone(&connection.fallback).send(emits_changed_message),
                                );
                            }
                        }
                        future::Either::B(())
                    })
                    .for_each(|_| Ok(())),
            );
        }

        let gatt_characteristic = factory
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

                method_info
                    .path
                    .get_data()
                    .get_characteristic()
                    .properties
                    .read
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

                method_info
                    .path
                    .get_data()
                    .get_characteristic()
                    .properties
                    .write
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
            .add_m(factory.amethod("StartNotify", (), move |method_info| {
                let (sender, receiver) = mpsc::channel(1);
                let notify_subscribe = gatt::event::NotifySubscribe {
                    notification: sender,
                };

                let message_sender = message_sender.clone();
                thread::spawn(move || {
                    for possible_notification in receiver.wait() {
                        if let Ok(notification) = possible_notification {
                            block_on_all(message_sender.clone().send(notification)).unwrap();
                        }
                    }
                });

                method_info
                    .path
                    .get_data()
                    .get_characteristic()
                    .properties
                    .notify
                    .clone()
                    .ok_or_else(|| MethodErr::from((BLUEZ_ERROR_NOTSUPPORTED, "")))
                    .into_future()
                    .and_then(move |event_sender| {
                        event_sender
                            .send(gatt::event::Event::NotifySubscribe(notify_subscribe))
                            .map_err(|_| MethodErr::from((BLUEZ_ERROR_FAILED, "")))
                    })
                    .and_then(|_| Ok(vec![]))
            }))
            .add_m(factory.amethod("StopNotify", (), move |method_info| {
                method_info
                    .path
                    .get_data()
                    .get_characteristic()
                    .properties
                    .notify
                    .clone()
                    .ok_or_else(|| MethodErr::from((BLUEZ_ERROR_NOTSUPPORTED, "")))
                    .into_future()
                    .and_then(move |event_sender| {
                        event_sender
                            .send(gatt::event::Event::NotifyUnsubscribe)
                            .map_err(|_| MethodErr::from((BLUEZ_ERROR_FAILED, "")))
                    })
                    .and_then(|_| Ok(vec![]))
            }))
            .add_p(
                factory
                    .property::<&str, _>("UUID", ())
                    .access(Access::Read)
                    .on_get(move |i, prop_info| {
                        i.append(
                            &prop_info
                                .path
                                .get_data()
                                .get_characteristic()
                                .uuid
                                .to_string(),
                        );
                        Ok(())
                    }),
            )
            .add_p({
                let service = service.clone();
                factory
                    .property::<Path<'static>, _>("Service", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        i.append(&service);
                        Ok(())
                    })
            })
            .add_p(
                factory
                    .property::<&[&str], _>("Flags", ())
                    .access(Access::Read)
                    .on_get(move |i, prop_info| {
                        i.append(
                            &prop_info
                                .path
                                .get_data()
                                .get_characteristic()
                                .properties
                                .flags(),
                        );
                        Ok(())
                    }),
            )
            .add_p(Arc::clone(&value_property));

        let object_path = factory
            .object_path(
                format!("{}/characteristic{:04}", service, 0),
                common::GattDataType::Characteristic(Arc::clone(characteristic)),
            )
            .add(gatt_characteristic)
            .introspectable()
            .object_manager();

        let path = object_path.get_name().clone();

        tree.insert(object_path);

        Ok(Characteristic { object_path: path })
    }
}
