use objc::runtime::{Object, BOOL, NO, YES};

pub trait IntoBool {
    fn into_bool(self) -> bool;
}

impl IntoBool for BOOL {
    fn into_bool(self) -> bool {
        match self {
            YES => true,
            NO => false,
            _ => panic!("Unknown Objective-C BOOL value."),
        }
    }
}

impl IntoBool for *mut Object {
    fn into_bool(self) -> bool {
        (self as BOOL).into_bool()
    }
}

pub trait IntoObjcBool {
    fn into_objc_bool(self) -> BOOL;
}

impl IntoObjcBool for bool {
    fn into_objc_bool(self) -> BOOL {
        if self {
            YES
        } else {
            NO
        }
    }
}
