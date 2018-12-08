use super::characteristic::Characteristic;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Service {
    pub(crate) uuid: Uuid,
    pub(crate) primary: bool,
    pub(crate) characteristics: HashSet<Characteristic>,
}

impl Service {
    pub fn new(uuid: Uuid, primary: bool, characteristics: HashSet<Characteristic>) -> Self {
        Service {
            uuid,
            primary,
            characteristics,
        }
    }
}
