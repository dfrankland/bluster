use ffi::CBCharacteristicProperties;
use super::super::gatt::characteristic::{Characteristic, Property};

pub fn get_properties_and_permissions(characteristic: &Characteristic) -> (u16, u8) {
    let mut properties: u16 = 0;
    let mut permissions: u8 = 0;

    if characteristic.properties.contains(&Property::Read) {
        properties |= CBCharacteristicProperties::CBCharacteristicPropertyRead;

        if characteristic.secure.contains(&Property::Read) {
            permissions |= CBAttributePermissions::CBAttributePermissionsReadEncryptionRequired;
        } else {
            permissions |= CBAttributePermissions::CBAttributePermissionsReadable;
        }
    }

    if characteristic.properties.contains(&Property::WriteWithoutResponse) {
        properties |= CBCharacteristicProperties::CBCharacteristicPropertyWriteWithoutResponse;

        // Same as secure write
        if characteristic.secure.contains(&Property::WriteWithoutResponse) {
            permissions |= CBAttributePermissions::CBAttributePermissionsWriteEncryptionRequired;
        } else {
            permissions |= CBAttributePermissions::CBAttributePermissionsWriteable;
        }
    }

    if characteristic.properties.contains(&Property::Write) {
        properties |= CBCharacteristicProperties::CBCharacteristicPropertyWrite;

        // Same as secure write without response
        if characteristic.secure.contains(&Property::Write) {
            permissions |= CBAttributePermissions::CBAttributePermissionsWriteEncryptionRequired;
        } else {
            permissions |= CBAttributePermissions::CBAttributePermissionsWriteable;
        }
    }

    if characteristic.properties.contains(&Property::Notify) {
        properties |= CBCharacteristicProperties::CBCharacteristicPropertyNotify;

        // This is mac-specific functionality, which there's no planned support for.
        // It requires that a device be "trusted" in one's iCloud account.
        // https://support.apple.com/en-us/HT204915#trustdevice
        //
        // if characteristic.secure.contains(&Property::Notify) {
        //     properties |= CBCharacteristicProperties::CBCharacteristicPropertyNotifyEncryptionRequired;
        // } else {
        //     properties |= CBCharacteristicProperties::CBCharacteristicPropertyNotify;
        // }
    }

    if characteristic.properties.contains(&Property::Indicate) {
        properties |= CBCharacteristicProperties::CBCharacteristicPropertyIndicate;

        // This is mac-specific functionality, which there's no planned support for.
        // It requires that a device be "trusted" in one's iCloud account.
        // https://support.apple.com/en-us/HT204915#trustdevice
        //
        // if characteristic.secure.contains(&Property::Indicate) {
        //     properties |= CBCharacteristicProperties::CBCharacteristicPropertyIndicateEncryptionRequired;
        // } else {
        //     properties |= CBCharacteristicProperties::CBCharacteristicPropertyIndicate;
        // }
    }

    (properties, permissions)
}
