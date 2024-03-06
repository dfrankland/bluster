use objc::{class, msg_send, runtime::Object, sel, sel_impl};
use objc_foundation::{INSString, NSString};
use uuid::Uuid;

pub trait IntoCBUUID {
    fn into_cbuuid(self) -> *mut Object;
}

impl IntoCBUUID for Uuid {
    fn into_cbuuid(self) -> *mut Object {
        let uuid = self.hyphenated().to_string();
        let cls = class!(CBUUID);
        unsafe {
            let obj: *mut Object = msg_send![cls, alloc];
            msg_send![obj, initWithString: NSString::from_str(&uuid)]
        }
    }
}
