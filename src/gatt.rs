/// Generic Attributes (GATT)

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

macro_rules! impl_uuid_hash_eq {
    ($struct_with_uuid_member:ident) => {
        impl Hash for $struct_with_uuid_member {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.uuid.hash(state);
            }
        }

        impl PartialEq for $struct_with_uuid_member {
            fn eq(&self, other: &$struct_with_uuid_member) -> bool {
                self.uuid == other.uuid
            }
        }

        impl Eq for $struct_with_uuid_member {}
    };
}

pub mod service {
    use super::characteristic::Characteristic;
    use std::collections::HashSet;
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    pub struct Service {
        pub(crate) uuid: Uuid,
        pub(crate) primary: bool,
        pub(crate) characteristics: HashSet<Characteristic>,
    }

    impl Service {
        pub fn new(uuid: Uuid, primary: bool, characteristics: HashSet<Characteristic>) -> Self {
            Service {
                uuid,
                primary,
                characteristics,
            }
        }
    }
}

pub mod characteristic {
    use super::{descriptor::Descriptor, event::EventSender};
    use std::{
        collections::HashSet,
        hash::{Hash, Hasher},
    };
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    pub struct Characteristic {
        pub(crate) uuid: Uuid,
        pub(crate) properties: Properties,
        pub(crate) value: Option<Vec<u8>>,
        pub(crate) descriptors: HashSet<Descriptor>,
    }

    impl Characteristic {
        pub fn new(
            uuid: Uuid,
            properties: Properties,
            value: Option<Vec<u8>>,
            descriptors: HashSet<Descriptor>,
        ) -> Self {
            Characteristic {
                uuid,
                properties,
                value,
                descriptors,
            }
        }
    }

    impl_uuid_hash_eq!(Characteristic);

    properties!(WriteWithAndWithoutResponse, EventSender, { notify: EventSender, indicate: EventSender });
}

pub mod descriptor {
    use super::event::EventSender;
    use std::hash::{Hash, Hasher};
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    pub struct Descriptor {
        pub(crate) uuid: Uuid,
        pub(crate) properties: Properties,
        pub(crate) value: Option<Vec<u8>>,
    }

    impl Descriptor {
        pub fn new(uuid: Uuid, properties: Properties, value: Option<Vec<u8>>) -> Self {
            Descriptor {
                uuid,
                properties,
                value,
            }
        }
    }

    impl_uuid_hash_eq!(Descriptor);

    properties!(WriteWithResponse, EventSender);
}

pub mod event {
    use futures::channel::{mpsc, oneshot};

    pub type EventSender = mpsc::Sender<Event>;
    pub type ResponseSender = oneshot::Sender<Response>;

    #[derive(Debug)]
    pub enum Event {
        ReadRequest(ReadRequest),
        WriteRequest(WriteRequest),
        NotifySubscribe(NotifySubscribe),
        NotifyUnsubscribe,
        Notify,
        Indicate,
    }

    #[derive(Debug)]
    pub struct ReadRequest {
        pub offset: u16,
        pub response: ResponseSender,
    }

    #[derive(Debug)]
    pub struct WriteRequest {
        pub data: Vec<u8>,
        pub offset: u16,
        pub without_response: bool,
        pub response: ResponseSender,
    }

    #[derive(Debug)]
    pub struct NotifySubscribe {
        pub max_value_size: u16,
    }

    #[derive(Debug)]
    pub enum Response {
        Success(Vec<u8>),
        InvalidOffset,
        InvalidAttributeLength,
        UnlikelyError,
    }
}
