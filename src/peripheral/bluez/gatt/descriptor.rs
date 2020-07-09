use dbus::{
    arg::{RefArg, Variant},
    Path,
};
use dbus_crossroads::MethodErr;
use futures::{channel::oneshot, prelude::*};
use std::{collections::HashMap, sync::Arc};

use super::{
    super::{
        common,
        common::GattDataType,
        constants::{BLUEZ_ERROR_FAILED, BLUEZ_ERROR_NOTSUPPORTED, GATT_DESCRIPTOR_IFACE},
    },
    flags::Flags,
};
use crate::{gatt, Error};

type OptionsMap = HashMap<String, Variant<Box<dyn RefArg>>>;

#[derive(Debug, Clone)]
pub struct Descriptor {
    pub object_path: Path<'static>,
}

impl Descriptor {
    pub fn new(
        tree: &mut common::Tree,
        descriptor: &Arc<gatt::descriptor::Descriptor>,
        characteristic: &Path<'static>,
        index: u64,
    ) -> Result<Self, Error> {
        // Setup value property for read / write by other methods
        let iface_token = tree.register::<GattDataType, _, _>(GATT_DESCRIPTOR_IFACE, |b| {
            b.method_with_cr_async(
                "ReadValue",
                ("options",),
                ("value",),
                |mut ctx, cr, (options,): (OptionsMap,)| {
                    let offset = options.get("offset").and_then(RefArg::as_u64).unwrap_or(0) as u16;
                    let descriptor = cr
                        .data_mut::<GattDataType>(ctx.path())
                        .unwrap()
                        .get_descriptor();
                    async move {
                        let event_sender = descriptor
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
                    let descriptor = cr
                        .data_mut::<GattDataType>(ctx.path())
                        .unwrap()
                        .get_descriptor();
                    async move {
                        let event_sender = descriptor
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
            b.property("UUID")
                .get(|_ctx, data| Ok(data.get_descriptor().uuid.to_string()));
            let characteristic = characteristic.clone();
            b.property("Characteristic")
                .get(move |_ctx, _data| Ok(characteristic.clone()));
            b.property("Flags")
                .get(move |_ctx, data| Ok(data.get_descriptor().properties.flags()));
        });
        let object_path: Path =
            format!("{}/descriptor{:04}", characteristic.to_string(), index).into();
        let object_path_data = common::GattDataType::Descriptor(Arc::clone(descriptor));

        tree.insert(object_path.clone(), &[iface_token], object_path_data);

        Ok(Descriptor { object_path })
    }
}
