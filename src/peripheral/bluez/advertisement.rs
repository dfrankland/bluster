use dbus::{
    arg::{RefArg, Variant},
    channel::MatchingReceiver,
    message::MatchRule,
    Path,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use super::{
    common,
    connection::Connection,
    constants::{LE_ADVERTISEMENT_IFACE, LE_ADVERTISING_MANAGER_IFACE, PATH_BASE},
};
use crate::Error;

#[derive(Debug, Clone)]
pub struct Advertisement {
    connection: Arc<Connection>,
    adapter: Path<'static>,
    pub object_path: Path<'static>,
    tree: Arc<Mutex<common::Tree>>,
    is_advertising: Arc<AtomicBool>,
    name: Arc<Mutex<Option<String>>>,
    uuids: Arc<Mutex<Option<Vec<String>>>>,
}

impl Advertisement {
    pub fn new(connection: Arc<Connection>, adapter: Path<'static>) -> Self {
        let mut tree = common::Tree::new();
        let is_advertising = Arc::new(AtomicBool::new(false));
        let is_advertising_release = is_advertising.clone();

        let name = Arc::new(Mutex::new(None));
        let name_property = name.clone();

        let uuids = Arc::new(Mutex::new(None));
        let uuids_property = uuids.clone();

        let object_path: Path = format!("{}/advertisement{:04}", PATH_BASE, 0).into();

        let iface_token = tree.register(LE_ADVERTISEMENT_IFACE, |b| {
            b.method_with_cr_async("Release", (), (), move |mut ctx, _cr, ()| {
                is_advertising_release.store(false, Ordering::Relaxed);
                futures::future::ready(ctx.reply(Ok(())))
            });
            b.property("Type")
                .get(|_ctx, _cr| Ok("peripheral".to_owned()));
            b.property("LocalName").get(move |_ctx, _cr| {
                Ok(name_property
                    .lock()
                    .expect("Poisoned mutex")
                    .clone()
                    .unwrap_or_else(String::new))
            });
            b.property("ServiceUUIDs").get(move |_ctx, _cr| {
                Ok(uuids_property
                    .lock()
                    .expect("Poisoned mutex")
                    .clone()
                    .unwrap_or_else(Vec::new))
            });
        });
        let ifaces = [iface_token, tree.object_manager()];
        tree.insert(object_path.clone(), &ifaces, ());

        let tree = Arc::new(Mutex::new(tree));

        {
            let tree = tree.clone();
            let mut match_rule = MatchRule::new_method_call();
            match_rule.path = Some(object_path.clone());
            connection.default.start_receive(
                match_rule,
                Box::new(move |msg, conn| {
                    tree.lock().unwrap().handle_message(msg, conn).unwrap();
                    true
                }),
            );
        }

        Advertisement {
            connection,
            adapter,
            object_path,
            tree,
            is_advertising,
            name,
            uuids,
        }
    }

    pub fn add_name<T: Into<String>>(self: &Self, name: T) {
        self.name.lock().unwrap().replace(name.into());
    }

    pub fn add_uuids<T: Into<Vec<String>>>(self: &Self, uuids: T) {
        self.uuids.lock().unwrap().replace(uuids.into());
    }

    pub async fn register(self: &Self) -> Result<(), Error> {
        // Register with DBus
        let proxy = self.connection.get_bluez_proxy(&self.adapter);
        proxy
            .method_call(
                LE_ADVERTISING_MANAGER_IFACE,
                "RegisterAdvertisement",
                (
                    &self.object_path,
                    HashMap::<String, Variant<Box<dyn RefArg>>>::new(),
                ),
            )
            .await?;
        self.is_advertising.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub async fn unregister(self: &Self) -> Result<(), Error> {
        let proxy = self.connection.get_bluez_proxy(&self.adapter);

        let method_call = proxy.method_call(
            LE_ADVERTISING_MANAGER_IFACE,
            "UnregisterAdvertisement",
            (&self.object_path,),
        );

        self.is_advertising.store(false, Ordering::Relaxed);

        method_call.await?;
        Ok(())
    }

    pub fn is_advertising(self: &Self) -> bool {
        let is_advertising = self.is_advertising.clone();
        is_advertising.load(Ordering::Relaxed)
    }
}
