use dbus::{
    arg::{RefArg, Variant},
    tree::{Access, MethodErr},
    MessageItem, Path,
};
use dbus_tokio::tree::AFactory;
use futures::{prelude::*, sync::oneshot::channel};
use std::{collections::HashMap, sync::Arc};

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

        let read_value = descriptor.properties.read.clone();
        let write_value = descriptor.properties.write.clone();
        let flags_value = descriptor.properties.flags().clone();
        let uuid_value = descriptor.uuid.to_string();
        let characteristic_value = characteristic.clone();
        let intial_value = descriptor.value.clone().unwrap_or_else(Vec::new);

        let mut gatt_descriptor = factory
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
                    .property::<Path<'static>, _>("Characteristic", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        i.append(&characteristic_value);
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

        if descriptor.properties.is_read_only() && descriptor.value.is_some() {
            gatt_descriptor = gatt_descriptor.add_p(
                factory
                    .property::<&[u8], _>("Value", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        i.append(&intial_value);
                        Ok(())
                    }),
            );
        }

        let object_path = factory
            .object_path(
                format!("{}/descriptor{:04}", characteristic.to_string(), 0),
                common::GattDataType::None,
            )
            .add(gatt_descriptor)
            .introspectable()
            .object_manager();

        let path = object_path.get_name().clone();

        tree.insert(object_path);

        Ok(Descriptor { object_path: path })
    }
}
