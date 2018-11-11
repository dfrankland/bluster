use super::constants::{BLUEZ_ERROR_FAILED, BLUEZ_ERROR_NOTSUPPORTED, GATT_CHARACTERISTIC_IFACE};
use crate::gatt;
use dbus::{
    arg::{RefArg, Variant},
    tree::{Access, Factory, MTFn, Tree},
    Connection, MessageItem, Path,
};
use futures::{channel::oneshot::channel, executor::block_on};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct Characteristic {
    pub object_path: Path<'static>,
    tree: Arc<Tree<MTFn, ()>>,
}

impl Characteristic {
    pub fn new(
        factory: &Factory<MTFn>,
        characteristic: &Arc<gatt::characteristic::Characteristic>,
        service: &Arc<Path<'static>>,
    ) -> Self {
        let characteristic_read_value = characteristic.clone();
        let characteristic_write_value = characteristic.clone();
        let characteristic_uuid = characteristic.clone();
        let characteristic_service = service.clone();
        let characteristic_flags = characteristic.clone();
        let characteristic_value = characteristic.clone();

        let mut gatt_characteristic = factory
            .interface(GATT_CHARACTERISTIC_IFACE, ())
            .add_m(factory.method("ReadValue", (), move |method_info| {
                if let Some(event_sender) = &(*characteristic_read_value).properties.read {
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
                if let Some(event_sender) = &(*characteristic_write_value).properties.write {
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
                        i.append((*characteristic_uuid).uuid.to_string());
                        Ok(())
                    }),
            )
            .add_p(
                factory
                    .property::<Path<'static>, _>("Service", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        i.append(&*characteristic_service);
                        Ok(())
                    }),
            )
            .add_p(
                factory
                    .property::<&[&str], _>("Flags", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        let gatt::characteristic::Properties { read, write, .. } =
                            &(*characteristic_flags).properties;

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
                        i.append((*characteristic_value).value.as_ref().unwrap());
                        Ok(())
                    }),
            );
        }

        let object_path = factory
            .object_path(
                format!("{}/characteristic{:04}", service.to_string(), 0),
                (),
            )
            .add(gatt_characteristic);

        let path = object_path.get_name().clone();
        let tree = Arc::new(factory.tree(()).add(object_path));

        Characteristic {
            object_path: path,
            tree,
        }
    }

    pub fn register(self: &Self, connection: &Connection) -> Result<(), dbus::Error> {
        self.register_with_dbus(connection)?;
        Ok(())
    }

    fn register_with_dbus(self: &Self, connection: &Connection) -> Result<(), dbus::Error> {
        self.tree.set_registered(connection, true)?;
        connection.add_handler(self.tree.clone());
        Ok(())
    }
}
