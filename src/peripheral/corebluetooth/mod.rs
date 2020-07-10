mod characteristic_flags;
mod constants;
mod error;
mod events;
mod ffi;
mod into_bool;
mod into_cbuuid;
mod peripheral_manager;

use uuid::Uuid;

use self::peripheral_manager::PeripheralManager;
use crate::{gatt::service::Service, Error};

pub struct Peripheral {
    peripheral_manager: PeripheralManager,
}

impl Peripheral {
    #[allow(clippy::new_ret_no_self)]
    pub async fn new() -> Result<Self, Error> {
        Ok(Peripheral {
            peripheral_manager: PeripheralManager::new(),
        })
    }

    pub async fn is_powered(&self) -> Result<bool, Error> {
        Ok(self.peripheral_manager.is_powered())
    }

    pub async fn register_gatt(&self) -> Result<(), Error> {
        Ok(())
    }

    pub async fn unregister_gatt(&self) -> Result<(), Error> {
        Ok(())
    }

    pub async fn start_advertising(self: &Self, name: &str, uuids: &[Uuid]) -> Result<(), Error> {
        self.peripheral_manager.start_advertising(name, uuids);
        Ok(())
    }

    pub async fn stop_advertising(&self) -> Result<(), Error> {
        self.peripheral_manager.stop_advertising();
        Ok(())
    }

    pub async fn is_advertising(&self) -> Result<bool, Error> {
        Ok(self.peripheral_manager.is_advertising())
    }

    pub fn add_service(&self, service: &Service) -> Result<(), Error> {
        self.peripheral_manager.add_service(service);
        Ok(())
    }
}
