use super::constants::{BLUEZ_ERROR_FAILED, BLUEZ_ERROR_NOTSUPPORTED, GATT_DESCRIPTOR_IFACE};
use crate::gatt;
use dbus::{
    arg::{RefArg, Variant},
    tree::{Access, Factory, MTFn, Tree},
    MessageItem, Path,
};
use futures::{channel::oneshot::channel, executor::block_on};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct Descriptor {
    pub object_path: Path<'static>,
}

impl Descriptor {
    pub fn new(
        factory: &Factory<MTFn>,
        tree: &mut Tree<MTFn, ()>,
        descriptor: &Arc<gatt::descriptor::Descriptor>,
        characteristic: &Arc<Path<'static>>,
    ) -> Result<Self, dbus::Error> {
        let descriptor_read_value = descriptor.clone();
        let descriptor_write_value = descriptor.clone();
        let descriptor_uuid = descriptor.clone();
        let descriptor_characteristic = characteristic.clone();
        let descriptor_flags = descriptor.clone();
        let descriptor_value = descriptor.clone();

        let mut gatt_descriptor = factory
            .interface(GATT_DESCRIPTOR_IFACE, ())
            .add_m(factory.method("ReadValue", (), move |method_info| {
                if let Some(ref event_sender) = &(*descriptor_read_value).properties.read {
                    let (sender, receiver) = channel();
                    let read_request = gatt::event::Event::ReadRequest(gatt::event::ReadRequest {
                        offset: method_info
                            .msg
                            .get1::<HashMap<String, Variant<MessageItem>>>()
                            .unwrap()["offset"]
                            .clone()
                            .as_u64()
                            .unwrap() as u16,
                        response: sender,
                    });
                    event_sender
                        .clone()
                        .sender()
                        .try_send(read_request)
                        .unwrap();
                    return match block_on(receiver) {
                        Ok(response) => match response {
                            gatt::event::Response::Success(value) => {
                                Ok(vec![method_info.msg.method_return().append1(value)])
                            }
                            _ => Err((BLUEZ_ERROR_FAILED, "").into()),
                        },
                        Err(_) => Err((BLUEZ_ERROR_FAILED, "").into()),
                    };
                }

                Err((BLUEZ_ERROR_NOTSUPPORTED, "").into())
            }))
            .add_m(factory.method("WriteValue", (), move |method_info| {
                if let Some(ref event_sender) = &(*descriptor_write_value).properties.write {
                    let (sender, receiver) = channel();
                    let (data, flags) = method_info
                        .msg
                        .get2::<Vec<u8>, HashMap<String, Variant<MessageItem>>>();
                    let write_request =
                        gatt::event::Event::WriteRequest(gatt::event::WriteRequest {
                            data: data.unwrap(),
                            offset: flags.unwrap()["offset"].clone().as_u64().unwrap() as u16,
                            without_response: false,
                            response: sender,
                        });
                    event_sender
                        .clone()
                        .sender()
                        .try_send(write_request)
                        .unwrap();
                    return match block_on(receiver) {
                        Ok(response) => match response {
                            gatt::event::Response::Success(value) => {
                                Ok(vec![method_info.msg.method_return().append1(value)])
                            }
                            _ => Err((BLUEZ_ERROR_FAILED, "").into()),
                        },
                        Err(_) => Err((BLUEZ_ERROR_FAILED, "").into()),
                    };
                }

                Err((BLUEZ_ERROR_NOTSUPPORTED, "").into())
            }))
            .add_p(
                factory
                    .property::<&str, _>("UUID", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        i.append((*descriptor_uuid).uuid.to_string());
                        Ok(())
                    }),
            )
            .add_p(
                factory
                    .property::<Path<'static>, _>("Characteristic", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        i.append(&*descriptor_characteristic);
                        Ok(())
                    }),
            )
            .add_p(
                factory
                    .property::<&[&str], _>("Flags", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        let gatt::descriptor::Properties { read, write, .. } =
                            &(*descriptor_flags).properties;

                        let mut flags = vec![];

                        if let Some(read) = read {
                            let read_flag = match read {
                                gatt::descriptor::Secure::Secure(_) => "secure-read",
                                gatt::descriptor::Secure::Insecure(_) => "read",
                            };
                            flags.push(read_flag);
                        }

                        if let Some(write) = write {
                            let write_flag = match write {
                                gatt::descriptor::Secure::Secure(_) => "secure-read",
                                gatt::descriptor::Secure::Insecure(_) => "read",
                            };
                            flags.push(write_flag);
                        }

                        i.append(flags);
                        Ok(())
                    }),
            );

        if descriptor.properties.is_read_only() && descriptor.value.is_some() {
            gatt_descriptor = gatt_descriptor.add_p(
                factory
                    .property::<&[u8], _>("Value", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        i.append((*descriptor_value).value.as_ref().unwrap());
                        Ok(())
                    }),
            );
        }

        let object_path = factory
            .object_path(
                format!("{}/descriptor{:04}", characteristic.to_string(), 0),
                (),
            )
            .add(gatt_descriptor)
            .introspectable()
            .object_manager();

        let path = object_path.get_name().clone();

        tree.insert(object_path);

        Ok(Descriptor { object_path: path })
    }
}
