use dbus::{tree::Access, Path};
use dbus_tokio::tree::AFactory;
use std::sync::Arc;

use super::super::common;
use super::super::constants::{GATT_SERVICE_IFACE, PATH_BASE};
use crate::{gatt, Error};

#[derive(Debug, Clone)]
pub struct Service {
    pub object_path: Path<'static>,
}

impl Service {
    pub fn new(
        tree: &mut common::Tree,
        service: &Arc<gatt::service::Service>,
    ) -> Result<Self, Error> {
        let factory = AFactory::new_afn::<()>();

        let service_uuid = service.clone();
        let service_primary = service.clone();

        let get_all = factory
            .interface(GATT_SERVICE_IFACE, ())
            .add_p(
                factory
                    .property::<&str, _>("UUID", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        i.append((*service_uuid).uuid.to_string());
                        Ok(())
                    }),
            )
            .add_p(
                factory
                    .property::<bool, _>("Primary", ())
                    .access(Access::Read)
                    .on_get(move |i, _| {
                        i.append((*service_primary).primary);
                        Ok(())
                    }),
            );

        let object_path = factory
            .object_path(format!("{}/service{:04}", PATH_BASE, 0), ())
            .add(get_all)
            .introspectable()
            .object_manager();

        let path = object_path.get_name().clone();

        tree.insert(object_path);

        Ok(Service { object_path: path })
    }
}
