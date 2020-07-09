use futures::channel::{mpsc, oneshot};

pub type EventSender = mpsc::Sender<Event>;
pub type ResponseSender = oneshot::Sender<Response>;

#[derive(Debug)]
pub enum Event {
    ReadRequest(ReadRequest),
    WriteRequest(WriteRequest),
    NotifySubscribe(NotifySubscribe),
    NotifyUnsubscribe,
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

#[derive(Debug, Clone)]
pub struct NotifySubscribe {
    pub notification: mpsc::Sender<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub enum Response {
    Success(Vec<u8>),
    InvalidOffset,
    InvalidAttributeLength,
    UnlikelyError,
}
