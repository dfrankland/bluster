use dbus::Path;
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
        index: u64,
    ) -> Result<Self, Error> {
        let get_all = tree.register(GATT_SERVICE_IFACE, |b| {
            let service1 = service.clone();
            b.property("UUID")
                .get(move |_ctx, _cr| Ok(service1.uuid.to_string()));
            let service1 = service.clone();
            b.property("Primary")
                .get(move |_ctx, _cr| Ok(service1.primary));
        });
        let object_path: Path = format!("{}/service{:04}", PATH_BASE, index).into();
        tree.insert(object_path.clone(), &[get_all], ());
        Ok(Service { object_path })
    }
}
