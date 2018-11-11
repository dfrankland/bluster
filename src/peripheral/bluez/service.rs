use super::constants::{GATT_SERVICE_IFACE, PATH_BASE};
use crate::gatt;
use dbus::{
    tree::{Access, Factory, MTFn, Tree},
    Connection, Path,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Service {
    pub object_path: Path<'static>,
    tree: Arc<Tree<MTFn, ()>>,
}

impl Service {
    pub fn new(factory: &Factory<MTFn>, service: &Arc<gatt::service::Service>) -> Self {
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
            .add(get_all);

        let path = object_path.get_name().clone();
        let tree = Arc::new(factory.tree(()).add(object_path));

        Service {
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
