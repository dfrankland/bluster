macro_rules! _write_type {
    (WriteWithAndWithoutResponse, $event_sender:ident, $secure:ident) => {
        #[derive(Debug, Clone)]
        pub enum Write {
            WithResponse($secure),
            WithoutResponse($event_sender),
        }

        impl Write {
            pub fn sender(self: Self) -> $event_sender {
                match self {
                    Write::WithResponse(event_sender) => event_sender.sender(),
                    Write::WithoutResponse(event_sender) => event_sender,
                }
            }
        }
    };
    (WriteWithResponse, $event_sender:ident, $secure:ident) => {
        #[derive(Debug, Clone)]
        pub struct Write(pub $secure);

        impl Write {
            pub fn sender(self: Self) -> $event_sender {
                self.0.sender()
            }
        }

        impl std::ops::Deref for Write {
            type Target = $secure;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

macro_rules! _properties {
    ($event_sender:ident, { $($member:ident: $member_type:ty,)* }) => {
        #[derive(Debug, Clone)]
        pub struct Properties {
            pub(crate) read: Option<Read>,
            pub(crate) write: Option<Write>,
            $(pub(crate) $member: Option<$member_type>,)*
        }

        impl Properties {
            pub fn new(
                read: Option<Read>,
                write: Option<Write>,
                $($member: Option<$member_type>,)*
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

        #[derive(Debug, Clone)]
        pub struct Read(pub Secure);

        impl Read {
            pub fn sender(self: Self) -> $event_sender {
                self.0.sender()
            }
        }

        impl std::ops::Deref for Read {
            type Target = Secure;

            fn deref(&self) -> &Secure {
                &self.0
            }
        }

        #[derive(Debug, Clone)]
        pub enum Secure {
            Secure($event_sender),
            Insecure($event_sender),
        }

        impl Secure {
            pub fn sender(self: Self) -> $event_sender {
                match self {
                    Secure::Secure(event_sender) => event_sender,
                    Secure::Insecure(event_sender) => event_sender,
                }
            }
        }
    }
}

macro_rules! properties {
    (WriteWithResponse, $event_sender:ident, { $($member:ident: $member_type:ty,)* }) => {
        _write_type!(WriteWithResponse, $event_sender, Secure);
        _properties!($event_sender, { $($member: $member_type,)* });
    };
    (WriteWithResponse, $event_sender:ident, { $($member:ident: $member_type:ty),* }) => {
        _write_type!(WriteWithResponse, $event_sender, Secure);
        _properties!($event_sender, { $($member: $member_type,)* });
    };
    (WriteWithResponse, $event_sender:ident) => {
        _write_type!(WriteWithResponse, $event_sender, Secure);
        _properties!($event_sender, {});
    };
    (WriteWithAndWithoutResponse, $event_sender:ident, { $($member:ident: $member_type:ty,)* }) => {
        _write_type!(WriteWithAndWithoutResponse, $event_sender, Secure);
        _properties!($event_sender, { $($member: $member_type,)* });
    };
    (WriteWithAndWithoutResponse, $event_sender:ident, { $($member:ident: $member_type:ty),* }) => {
        _write_type!(WriteWithAndWithoutResponse, $event_sender, Secure);
        _properties!($event_sender, { $($member: $member_type,)* });
    };
    (WriteWithAndWithoutResponse, $event_sender:ident) => {
        _write_type!(WriteWithAndWithoutResponse, $event_sender:ident, Secure);
        properties!($event_sender, {});
    };
}
