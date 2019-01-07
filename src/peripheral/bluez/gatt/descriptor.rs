use dbus::{
    arg::{RefArg, Variant},
    tree::{Access, EmitsChangedSignal, MethodErr},
    MessageItem, Path,
};
use dbus_tokio::tree::AFactory;
use futures::{prelude::*, sync::oneshot::channel};
use std::{collections::HashMap, sync::{Arc, Mutex}};

use super::{
    super::{
        common,
        constants::{BLUEZ_ERROR_FAILED, BLUEZ_ERROR_NOTSUPPORTED, GATT_DESCRIPTOR_IFACE},
    },
    flags::Flags,
};
use crate::{gatt, Error};

#[derive(Debug, Clone)]
pub struct Descriptor {
    pub object_path: Path<'static>,
}

impl Descriptor {
    pub fn new(
        tree: &mut common::Tree,
        descriptor: &Arc<gatt::descriptor::Descriptor>,
        characteristic: &Path<'static>,
    ) -> Result<Self, Error> {
        let factory = AFactory::new_afn::<common::TData>();

        // Setup value property for read / write by other methods
        let value = Arc::new(Mutex::new(descriptor.value.clone()));
        let value_property = {
            let property = factory
                .property::<&[u8], _>("Value", ())
                .emits_changed(EmitsChangedSignal::True)
                .access({
                    let is_read_only_value =
                        descriptor.properties.is_read_only() && descriptor.value.is_some();
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

        let gatt_descriptor = factory
            .interface(GATT_DESCRIPTOR_IFACE, ())
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
                    .get_descriptor()
                    .properties
                    .read
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

                method_info
                    .path
                    .get_data()
                    .get_descriptor()
                    .properties
                    .write
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
                    .on_get(move |i, prop_info| {
                        i.append(&prop_info
                            .path
                            .get_data()
                            .get_descriptor()
                            .uuid
                            .to_string());
                        Ok(())
                    }),
            )
            .add_p({
                let characteristic = characteristic.clone();
                factory
                    .property::<Path<'static>, _>("Characteristic", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        i.append(&characteristic);
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
                                .get_descriptor()
                                .properties
                                .flags(),
                        );
                        Ok(())
                    }),
            ).add_p(Arc::clone(&value_property));

        let object_path = factory
            .object_path(
                format!("{}/descriptor{:04}", characteristic.to_string(), 0),
                common::GattDataType::Descriptor(Arc::clone(descriptor)),
            )
            .add(gatt_descriptor)
            .introspectable()
            .object_manager();

        let path = object_path.get_name().clone();

        tree.insert(object_path);

        Ok(Descriptor { object_path: path })
    }
}
