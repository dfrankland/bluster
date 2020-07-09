use dbus_crossroads::Crossroads;
use std::sync::Arc;

use crate::gatt;

#[derive(Debug, Clone)]
pub enum GattDataType {
    // Not needed...
    // Service(Arc<gatt::service::Service>),
    Characteristic(Arc<gatt::characteristic::Characteristic>),
    Descriptor(Arc<gatt::descriptor::Descriptor>),
}

impl GattDataType {
    // Not needed...
    // pub fn get_service(self: &Self) -> Arc<gatt::service::Service> {
    //     if let GattDataType::Service(ref service) = self {
    //         return service.clone();
    //     }
    //     panic!("GattDataType is not a Service!");
    // }

    pub fn get_characteristic(self: &Self) -> Arc<gatt::characteristic::Characteristic> {
        if let GattDataType::Characteristic(ref characteristic) = self {
            return characteristic.clone();
        }
        panic!("GattDataType is not a Characteristic!");
    }

    pub fn get_descriptor(self: &Self) -> Arc<gatt::descriptor::Descriptor> {
        if let GattDataType::Descriptor(ref descriptor) = self {
            return descriptor.clone();
        }
        panic!("GattDataType is not a Descriptor!");
    }
}

pub type Tree = Crossroads;
