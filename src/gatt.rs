pub mod primary_service {
    use uuid::Uuid;
    use super::characteristic::Characteristic;

    #[derive(Debug, Clone)]
    pub struct PrimaryService {
        pub(crate) uuid: Uuid,
        pub(crate) characteristics: Vec<Characteristic>,
    }

    impl PrimaryService {
        pub fn new(uuid: Uuid, characteristics: Vec<Characteristic>) -> Self {
            PrimaryService {
                uuid,
                characteristics,
            }
        }
    }
}

pub mod characteristic {
    use std::collections::HashSet;
    use uuid::Uuid;
    // use futures::channel::oneshot::Sender;
    use super::descriptor::Descriptor;

    #[derive(Debug, Clone)]
    pub struct Characteristic {
        pub(crate) uuid: Uuid,
        pub(crate) properties: HashSet<Property>,
        pub(crate) secure: HashSet<Property>,
        pub(crate) value: Option<Vec<u8>>,
        pub(crate) descriptors: Vec<Descriptor>,
    }

    impl Characteristic {
        pub fn new(
            uuid: Uuid,
            properties: HashSet<Property>,
            secure: HashSet<Property>,
            value: Option<Vec<u8>>,
            descriptors: Vec<Descriptor>,
        ) -> Self {
            Characteristic {
                uuid,
                properties,
                secure,
                value,
                descriptors,
            }
        }
    }

    #[derive(Debug, Clone, Hash, Eq, PartialEq)]
    pub enum Property {
        Read,
        Write,
        WriteWithoutResponse,
        Notify,
        Indicate,
    }

    // TODO: Implement events that will be sent
    //
    // #[derive(Debug)]
    // pub enum Event {
    //     ReadRequest(ReadRequest),
    //     WriteRequest(WriteRequest),
    //     NotifySubscribe(NotifySubscribe),
    //     NotifyUnsubscribe,
    //     Notify,
    //     Indicate,
    // }
    //
    // #[derive(Debug)]
    // pub struct ReadRequest {
    //     pub(crate) offset: u16,
    //     pub(crate) callback: Sender<(ResultCode, Vec<u8>)>,
    // }
    //
    // impl ReadRequest {
    //     pub fn new(
    //         offset: u16,
    //         callback: Sender<(ResultCode, Vec<u8>)>,
    //     ) -> Self {
    //         ReadRequest {
    //             offset,
    //             callback,
    //         }
    //     }
    // }
    //
    // #[derive(Debug)]
    // pub struct WriteRequest {
    //     pub(crate) data: Vec<u8>,
    //     pub(crate) offset: u16,
    //     pub(crate) without_response: bool,
    //     pub(crate) callback: Sender<ResultCode>,
    // }
    //
    // impl WriteRequest {
    //     pub fn new(
    //         data: Vec<u8>,
    //         offset: u16,
    //         without_response: bool,
    //         callback: Sender<ResultCode>,
    //     ) -> Self {
    //         WriteRequest {
    //             data,
    //             offset,
    //             without_response,
    //             callback,
    //         }
    //     }
    // }
    //
    // #[derive(Debug)]
    // pub struct NotifySubscribe {
    //     pub(crate) max_value_size: u16,
    //     pub(crate) update_value_callback: Sender<Vec<u8>>,
    // }
    //
    // impl NotifySubscribe {
    //     pub fn new(
    //         max_value_size: u16,
    //         update_value_callback: Sender<Vec<u8>>,
    //     ) -> Self {
    //         NotifySubscribe {
    //             max_value_size,
    //             update_value_callback,
    //         }
    //     }
    // }
    //
    // #[derive(Debug)]
    // pub enum ResultCode {
    //     Success,
    //     InvalidOffset,
    //     InvalidAttributeLength,
    //     UnlikelyError,
    // }
}

pub mod descriptor {
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    pub struct Descriptor {
      pub(crate) uuid: Uuid,
      pub(crate) value: Vec<u8>,
    }

    impl Descriptor {
        pub fn new(
            uuid: Uuid,
            value: Vec<u8>,
        ) -> Self {
            Descriptor {
                uuid,
                value,
            }
        }
    }
}
