use futures::channel::mpsc;

/// Message describes the available messages to be sent between
/// the participants.
pub enum Message {
    /// Sink an oppenents ship at the given coordinate.
    Sink { x: u8, y: u8 },
}

/// The application's mailbox that handles incoming messages.
pub struct Mailbox {
    sender: mpsc::Sender<Message>,
}

impl Mailbox {
    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        Self { sender }
    }
}
