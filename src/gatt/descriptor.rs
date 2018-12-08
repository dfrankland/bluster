use super::event::EventSender;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Descriptor {
    pub(crate) uuid: Uuid,
    pub(crate) properties: Properties,
    pub(crate) value: Option<Vec<u8>>,
}

impl Descriptor {
    pub fn new(uuid: Uuid, properties: Properties, value: Option<Vec<u8>>) -> Self {
        Descriptor {
            uuid,
            properties,
            value,
        }
    }
}

impl_uuid_hash_eq!(Descriptor);

properties!(WriteWithResponse, EventSender);
