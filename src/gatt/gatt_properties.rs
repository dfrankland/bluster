macro_rules! _define_operation_struct {
    ($struct:ident) => {
        #[derive(Debug, Clone)]
        pub struct $struct(pub Secure);

        impl $struct {
            pub fn sender(self: Self) -> crate::gatt::event::EventSender {
                self.0.sender()
            }
        }
    };
}
macro_rules! _write_type {
    (WriteWithAndWithoutResponse) => {
        #[derive(Debug, Clone)]
        pub enum Write {
            WithResponse(Secure),
            WithoutResponse(crate::gatt::event::EventSender),
        }

        impl Write {
            pub fn sender(self: Self) -> crate::gatt::event::EventSender {
                match self {
                    Write::WithResponse(event_sender) => event_sender.sender(),
                    Write::WithoutResponse(event_sender) => event_sender,
                }
            }
        }
    };
    (WriteWithResponse) => {
        _define_operation_struct!(Write);
    };
}

macro_rules! _properties {
    ({ $($member:ident,)* }) => {
        #[derive(Debug, Clone)]
        pub struct Properties {
            pub(crate) read: Option<Read>,
            pub(crate) write: Option<Write>,
            $(pub(crate) $member: Option<ServerInitiated>,)*
        }

        impl Properties {
            pub fn new(
                read: Option<Read>,
                write: Option<Write>,
                $($member: Option<ServerInitiated>,)*
            ) -> Self {
                Properties {
                    read,
                    write,
                    $($member,)*
                }
            }

            pub fn is_read_only(self: &Self) -> bool {
                self.read.is_some() && self.write.is_none()
            }
        }

        _define_operation_struct!(Read);
        _define_operation_struct!(ServerInitiated);

        #[derive(Debug, Clone)]
        pub enum Secure {
            Secure(crate::gatt::event::EventSender),
            Insecure(crate::gatt::event::EventSender),
        }

        impl Secure {
            pub fn sender(self: Self) -> crate::gatt::event::EventSender {
                match self {
                    Secure::Secure(event_sender) => event_sender,
                    Secure::Insecure(event_sender) => event_sender,
                }
            }
        }
    }
}
macro_rules! properties {
    (WriteWithResponse, { $($member:ident),* }) => {
        _write_type!(WriteWithResponse);
        _properties!({ $($member,)* });
    };
    (WriteWithResponse) => {
        _write_type!(WriteWithResponse);
        _properties!({});
    };
    (WriteWithAndWithoutResponse, { $($member:ident),* }) => {
        _write_type!(WriteWithAndWithoutResponse);
        _properties!({ $($member,)* });
    };
    (WriteWithAndWithoutResponse) => {
        _write_type!(WriteWithAndWithoutResponse);
        properties!({});
    };
}
