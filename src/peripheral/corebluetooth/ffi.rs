#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use std::os::raw::c_char;

use objc::runtime::Object;

pub const nil: *mut Object = 0 as *mut Object;

pub enum dispatch_object_s {}
pub type dispatch_queue_t = *mut dispatch_object_s;
pub type dispatch_queue_attr_t = *const dispatch_object_s;
pub const DISPATCH_QUEUE_SERIAL: dispatch_queue_attr_t = 0 as dispatch_queue_attr_t;

#[link(name = "AppKit", kind = "framework")]
#[link(name = "Foundation", kind = "framework")]
#[link(name = "CoreBluetooth", kind = "framework")]
extern "C" {
    pub fn dispatch_queue_create(
        label: *const c_char,
        attr: dispatch_queue_attr_t,
    ) -> dispatch_queue_t;
    pub static CBAdvertisementDataServiceUUIDsKey: *mut Object;
    pub static CBAdvertisementDataLocalNameKey: *mut Object;
}

#[allow(dead_code)]
#[repr(C)]
pub enum CBManagerState {
    CBManagerStateUnknown = 0x00,
    CBManagerStateResetting = 0x01,
    CBManagerStateUnsupported = 0x02,
    CBManagerStateUnauthorized = 0x03,
    CBManagerStatePoweredOff = 0x04,
    CBManagerStatePoweredOn = 0x05,
}

#[allow(dead_code)]
#[repr(C)]
pub enum CBCharacteristicProperties {
    CBCharacteristicPropertyBroadcast = 0x01,
    CBCharacteristicPropertyRead = 0x02,
    CBCharacteristicPropertyWriteWithoutResponse = 0x04,
    CBCharacteristicPropertyWrite = 0x08,
    CBCharacteristicPropertyNotify = 0x10,
    CBCharacteristicPropertyIndicate = 0x20,
    CBCharacteristicPropertyAuthenticatedSignedWrites = 0x40,
    CBCharacteristicPropertyExtendedProperties = 0x80,
    CBCharacteristicPropertyNotifyEncryptionRequired = 0x100,
    CBCharacteristicPropertyIndicateEncryptionRequired = 0x200,
}

#[allow(dead_code)]
#[repr(C)]
pub enum CBAttributePermissions {
    CBAttributePermissionsReadable = 0x01,
    CBAttributePermissionsWriteable = 0x02,
    CBAttributePermissionsReadEncryptionRequired = 0x04,
    CBAttributePermissionsWriteEncryptionRequired = 0x08,
}

#[allow(dead_code)]
#[repr(C)]
pub enum CBATTError {
    CBATTErrorSuccess = 0x00,
    CBATTErrorInvalidHandle = 0x01,
    CBATTErrorReadNotPermitted = 0x02,
    CBATTErrorWriteNotPermitted = 0x03,
    CBATTErrorInvalidPdu = 0x04,
    CBATTErrorInsufficientAuthentication = 0x05,
    CBATTErrorRequestNotSupported = 0x06,
    CBATTErrorInvalidOffset = 0x07,
    CBATTErrorInsufficientAuthorization = 0x08,
    CBATTErrorPrepareQueueFull = 0x09,
    CBATTErrorAttributeNotFound = 0x0A,
    CBATTErrorAttributeNotLong = 0x0B,
    CBATTErrorInsufficientEncryptionKeySize = 0x0C,
    CBATTErrorInvalidAttributeValueLength = 0x0D,
    CBATTErrorUnlikelyError = 0x0E,
    CBATTErrorInsufficientEncryption = 0x0F,
    CBATTErrorUnsupportedGroupType = 0x10,
    CBATTErrorInsufficientResources = 0x11,
}
