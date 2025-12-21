use futures::channel::mpsc::Sender;

/// TODO: check if lifetime would better be removed?
pub struct Mailbox {
    pub sender: Sender<Message>
}

impl Mailbox {
    pub fn new(sender: Sender<Message>) -> Self {
        Self { sender }
    }
}

pub enum Message {
    Draw { grid: String },
    Log { logs: Vec<String>}
}
