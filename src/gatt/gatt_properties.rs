macro_rules! _write_type {
    ($event_sender:ident, $secure:ident) => {
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
    ($other_type:ident) => {
        pub type Write = $other_type;
    };
}

macro_rules! _properties {
    ($event_sender:ident, { $($member:ident: $member_type:ty,)* }) => {
        #[derive(Debug, Clone)]
        pub struct Properties {
            pub(crate) read: Option<Secure>,
            pub(crate) write: Option<Write>,
            $(pub(crate) $member: Option<$member_type>,)*
        }

        impl Properties {
            pub fn new(
                read: Option<Secure>,
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
        _write_type!(Secure);
        _properties!($event_sender, { $($member: $member_type,)* });
    };
    (WriteWithResponse, $event_sender:ident, { $($member:ident: $member_type:ty),* }) => {
        _write_type!(Secure);
        _properties!($event_sender, { $($member: $member_type,)* });
    };
    (WriteWithResponse, $event_sender:ident) => {
        _write_type!(Secure);
        _properties!($event_sender, {});
    };
    (WriteWithAndWithoutResponse, $event_sender:ident, { $($member:ident: $member_type:ty,)* }) => {
        _write_type!($event_sender, Secure);
        _properties!($event_sender, { $($member: $member_type,)* });
    };
    (WriteWithAndWithoutResponse, $event_sender:ident, { $($member:ident: $member_type:ty),* }) => {
        _write_type!($event_sender, Secure);
        _properties!($event_sender, { $($member: $member_type,)* });
    };
    (WriteWithAndWithoutResponse, $event_sender:ident) => {
        _write_type!($event_sender:ident, Secure);
        properties!($event_sender, {});
    };
}
