use super::constants::{DBUS_PROP_IFACE, PATH_BASE};
use crate::gatt;
use dbus::{
    arg::Variant,
    tree::{Factory, MTFn, Tree},
    Connection, MessageItem, Path, Signature,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct Service {
    pub object_path: Path<'static>,
    tree: Arc<Tree<MTFn, ()>>,
}

impl Service {
    pub fn new(factory: &Factory<MTFn>, service: &Arc<gatt::service::Service>) -> Self {
        let service = service.clone();
        let get_all = factory.interface(DBUS_PROP_IFACE, ()).add_m(
            factory
                .method("GetAll", (), move |method_info| {
                    let mut props = HashMap::new();

                    let gatt::service::Service { uuid, primary, .. } = *service.clone();
                    props.insert("UUID", Variant(MessageItem::Str(uuid.to_string())));
                    props.insert("Primary", Variant(MessageItem::Bool(primary)));

                    Ok(vec![method_info.msg.method_return().append1(props)])
                })
                .in_arg(Signature::make::<String>())
                .out_arg(Signature::make::<HashMap<String, Variant<MessageItem>>>()),
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
