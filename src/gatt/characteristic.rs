use super::{descriptor::Descriptor, event::EventSender};
use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Characteristic {
    pub(crate) uuid: Uuid,
    pub(crate) properties: Properties,
    pub(crate) value: Option<Vec<u8>>,
    pub(crate) descriptors: HashSet<Descriptor>,
}

impl Characteristic {
    pub fn new(
        uuid: Uuid,
        properties: Properties,
        value: Option<Vec<u8>>,
        descriptors: HashSet<Descriptor>,
    ) -> Self {
        Characteristic {
            uuid,
            properties,
            value,
            descriptors,
        }
    }
}

impl_uuid_hash_eq!(Characteristic);

properties!(WriteWithAndWithoutResponse, EventSender, { notify: EventSender, indicate: EventSender });
