use dbus::{
    arg::{RefArg, Variant},
    tree::{Access, MethodErr},
    MessageItem, Path,
};
use dbus_tokio::tree::AFactory;
use futures::{prelude::*, sync::oneshot::channel};
use std::{collections::HashMap, sync::Arc};

use super::super::{
    common,
    constants::{BLUEZ_ERROR_FAILED, BLUEZ_ERROR_NOTSUPPORTED, GATT_CHARACTERISTIC_IFACE},
};
use crate::{gatt, Error};

#[derive(Debug, Clone)]
pub struct Characteristic {
    pub object_path: Path<'static>,
}

impl Characteristic {
    pub fn new(
        tree: &mut common::Tree,
        characteristic: &Arc<gatt::characteristic::Characteristic>,
        service: &Path<'static>,
    ) -> Result<Self, Error> {
        let factory = AFactory::new_afn::<()>();

        let read_value = characteristic.properties.read.clone();
        let write_value = characteristic.properties.write.clone();
        let uuid_value = characteristic.uuid.to_string();
        let service_value = service.clone();
        let flags_value = characteristic.properties.clone();
        let initial_value = characteristic.value.clone();

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
                        let (sender, receiver) = channel();
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
                        let (sender, receiver) = channel();
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
                        let gatt::characteristic::Properties { read, write, .. } = &flags_value;

                        let mut flags = vec![];

                        if let Some(read) = read {
                            let read_flag = match read {
                                gatt::characteristic::Secure::Secure(_) => "secure-read",
                                gatt::characteristic::Secure::Insecure(_) => "read",
                            };
                            flags.push(read_flag);
                        }

                        if let Some(write) = write {
                            let write_flag = match write {
                                gatt::characteristic::Write::WithResponse(secure) => match secure {
                                    gatt::characteristic::Secure::Secure(_) => "secure-write",
                                    gatt::characteristic::Secure::Insecure(_) => "write",
                                },
                                gatt::characteristic::Write::WithoutResponse(_) => {
                                    "write-without-response"
                                }
                            };
                            flags.push(write_flag);
                        }

                        i.append(flags);
                        Ok(())
                    }),
            );

        if characteristic.properties.is_read_only() && characteristic.value.is_some() {
            gatt_characteristic = gatt_characteristic.add_p(
                factory
                    .property::<&[u8], _>("Value", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        i.append(initial_value.clone().unwrap_or_else(Vec::new));
                        Ok(())
                    }),
            );
        }

        let object_path = factory
            .object_path(format!("{}/characteristic{:04}", service, 0), ())
            .add(gatt_characteristic)
            .introspectable()
            .object_manager();

        let path = object_path.get_name().clone();

        tree.insert(object_path);

        Ok(Characteristic { object_path: path })
    }
}
