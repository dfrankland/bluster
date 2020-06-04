use std::{
    ffi::CString,
    sync::{Once, ONCE_INIT},
};

use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{Class, Object, Protocol, Sel, NO, YES},
    sel, sel_impl,
};
use objc_foundation::{
    INSArray, INSData, INSDictionary, INSString, NSArray, NSData, NSDictionary, NSObject, NSString,
};
use objc_id::{Id, Shared};

use uuid::Uuid;

use crate::gatt::service::Service;

use super::{
    characteristic_flags::get_properties_and_permissions,
    constants::{PERIPHERAL_MANAGER_DELEGATE_CLASS_NAME, PERIPHERAL_MANAGER_IVAR, POWERED_ON_IVAR},
    events::{
        peripheral_manager_did_add_service_error, peripheral_manager_did_receive_read_request,
        peripheral_manager_did_receive_write_requests,
        peripheral_manager_did_start_advertising_error, peripheral_manager_did_update_state,
    },
    ffi::{
        dispatch_queue_create, nil, CBAdvertisementDataLocalNameKey,
        CBAdvertisementDataServiceUUIDsKey, DISPATCH_QUEUE_SERIAL,
    },
    into_bool::IntoBool,
    into_cbuuid::IntoCBUUID,
};

static REGISTER_DELEGATE_CLASS: Once = ONCE_INIT;

#[derive(Debug)]
pub struct PeripheralManager {
    peripheral_manager_delegate: Id<Object, Shared>,
}

impl PeripheralManager {
    pub fn new() -> Self {
        REGISTER_DELEGATE_CLASS.call_once(|| {
            let mut decl =
                ClassDecl::new(PERIPHERAL_MANAGER_DELEGATE_CLASS_NAME, class!(NSObject)).unwrap();
            decl.add_protocol(Protocol::get("CBPeripheralManagerDelegate").unwrap());

            decl.add_ivar::<*mut Object>(PERIPHERAL_MANAGER_IVAR);
            decl.add_ivar::<*mut Object>(POWERED_ON_IVAR);

            unsafe {
                decl.add_method(
                    sel!(init),
                    init as extern "C" fn(&mut Object, Sel) -> *mut Object,
                );
                decl.add_method(
                    sel!(peripheralManagerDidUpdateState:),
                    peripheral_manager_did_update_state
                        as extern "C" fn(&mut Object, Sel, *mut Object),
                );
                decl.add_method(
                    sel!(peripheralManagerDidStartAdvertising:error:),
                    peripheral_manager_did_start_advertising_error
                        as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object),
                );
                decl.add_method(
                    sel!(peripheralManager:didAddService:error:),
                    peripheral_manager_did_add_service_error
                        as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object, *mut Object),
                );
                decl.add_method(
                    sel!(peripheralManager:didReceiveReadRequest:),
                    peripheral_manager_did_receive_read_request
                        as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object),
                );
                decl.add_method(
                    sel!(peripheralManager:didReceiveWriteRequests:),
                    peripheral_manager_did_receive_write_requests
                        as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object),
                );
            }

            decl.register();
        });

        let peripheral_manager_delegate = unsafe {
            let cls = Class::get(PERIPHERAL_MANAGER_DELEGATE_CLASS_NAME).unwrap();
            let mut obj: *mut Object = msg_send![cls, alloc];
            obj = msg_send![obj, init];
            Id::from_ptr(obj).share()
        };

        PeripheralManager {
            peripheral_manager_delegate,
        }
    }

    pub fn is_powered(self: &Self) -> bool {
        unsafe {
            let powered_on = *self
                .peripheral_manager_delegate
                .get_ivar::<*mut Object>(POWERED_ON_IVAR);
            powered_on.into_bool()
        }
    }

    pub fn start_advertising(self: &Self, name: &str, uuids: &[Uuid]) {
        let peripheral_manager = unsafe {
            *self
                .peripheral_manager_delegate
                .get_ivar::<*mut Object>(PERIPHERAL_MANAGER_IVAR)
        };

        let mut keys: Vec<&NSString> = vec![];
        let mut objects: Vec<Id<NSObject>> = vec![];

        unsafe {
            keys.push(&*(CBAdvertisementDataLocalNameKey as *mut NSString));
            objects.push(Id::from_retained_ptr(msg_send![
                NSString::from_str(name),
                copy
            ]));
            keys.push(&*(CBAdvertisementDataServiceUUIDsKey as *mut NSString));
            objects.push(Id::from_retained_ptr(msg_send![
                NSArray::from_vec(
                    uuids
                        .iter()
                        .map(|u| NSString::from_str(&u.to_hyphenated().to_string()))
                        .collect::<Vec<Id<NSString>>>()
                ),
                copy
            ]));
        }

        let advertising_data = NSDictionary::from_keys_and_objects(keys.as_slice(), objects);
        unsafe {
            let _: Result<(), ()> =
                msg_send![peripheral_manager, startAdvertising: advertising_data];
        }
    }

    pub fn stop_advertising(self: &Self) {
        unsafe {
            let peripheral_manager = *self
                .peripheral_manager_delegate
                .get_ivar::<*mut Object>(PERIPHERAL_MANAGER_IVAR);
            let _: Result<(), ()> = msg_send![peripheral_manager, stopAdvertising];
        }
    }

    pub fn is_advertising(self: &Self) -> bool {
        unsafe {
            let peripheral_manager = *self
                .peripheral_manager_delegate
                .get_ivar::<*mut Object>(PERIPHERAL_MANAGER_IVAR);
            let response: *mut Object = msg_send![peripheral_manager, isAdvertising];
            response.into_bool()
        }
    }

    pub fn add_service(self: &Self, service: &Service) {
        let characteristics: Vec<Id<NSObject>> = service
            .characteristics
            .iter()
            .map(|characteristic| {
                let (properties, permissions) = get_properties_and_permissions(characteristic);
                unsafe {
                    let cls = class!(CBMutableCharacteristic);
                    let obj: *mut Object = msg_send![cls, alloc];

                    let init_with_type = characteristic.uuid.into_cbuuid();
                    let mutable_characteristic: *mut Object = match characteristic.value {
                        Some(ref value) => msg_send![obj, initWithType:init_with_type
                                                            properties:properties
                                                                 value:NSData::with_bytes(value)
                                                           permissions:permissions],
                        None => msg_send![obj, initWithType:init_with_type
                                                 properties:properties
                                                      value:nil
                                                permissions:permissions],
                    };

                    Id::from_ptr(mutable_characteristic as *mut NSObject)
                }
            })
            .collect();

        unsafe {
            let cls = class!(CBMutableService);
            let obj: *mut Object = msg_send![cls, alloc];
            let service: *mut Object = msg_send![obj, initWithType:service.uuid.into_cbuuid()
                                                           primary:YES];
            let _: Result<(), ()> = msg_send![service, setValue:NSArray::from_vec(characteristics)
                                 forKey:NSString::from_str("characteristics")];

            let peripheral_manager = *self
                .peripheral_manager_delegate
                .get_ivar::<*mut Object>(PERIPHERAL_MANAGER_IVAR);

            let _: Result<(), ()> = msg_send![peripheral_manager, addService: service];
        }
    }
}

impl Default for PeripheralManager {
    fn default() -> Self {
        PeripheralManager::new()
    }
}

extern "C" fn init(delegate: &mut Object, _cmd: Sel) -> *mut Object {
    unsafe {
        let cls = class!(CBPeripheralManager);
        let mut obj: *mut Object = msg_send![cls, alloc];

        #[allow(clippy::cast_ptr_alignment)]
        let init_with_delegate = delegate as *mut Object as *mut *mut Object;

        let label = CString::new("CBqueue").unwrap();
        let queue = dispatch_queue_create(label.as_ptr(), DISPATCH_QUEUE_SERIAL);

        obj = msg_send![obj, initWithDelegate:init_with_delegate
                                        queue:queue];
        delegate.set_ivar::<*mut Object>(PERIPHERAL_MANAGER_IVAR, obj);

        delegate.set_ivar::<*mut Object>(POWERED_ON_IVAR, NO as *mut Object);

        delegate
    }
}
