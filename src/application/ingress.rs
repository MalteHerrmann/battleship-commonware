use super::gamestate;

use eyre;
use futures::channel::mpsc;
use serde::{Deserialize, Serialize};

/// Message describes the available messages to be sent between
/// the participants.
#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    /// Attack an oppenent's field at the given coordinate.
    Attack { m: gamestate::Move },
    /// Signals the end of the game based on all ships of one player being hit.
    EndGame,
    /// Signals that a move has successfully hit a ship.
    Hit { m: gamestate::Move },
    /// Signals that a move has failed to hit a target.
    Miss { m: gamestate::Move },
    /// Signals to the other peer that the player is ready.
    Ready,
}

impl Message {
    /// Validates the contents of the message.
    pub fn validate(&self) -> eyre::Result<()> {
        match self {
            Message::Attack { m } => m.validate()?,
            Message::EndGame => (),
            Message::Hit { m } => m.validate()?,
            Message::Miss { m } => m.validate()?,
            Message::Ready => (),
        }

        Ok(())
    }
}

impl From<Message> for bytes::Bytes {
    fn from(val: Message) -> Self {
        let serialized = serde_yaml::to_string(&val).expect("failed to serialize message");

        bytes::Bytes::from(serialized.into_bytes())
    }
}

impl From<bytes::Bytes> for Message {
    fn from(value: bytes::Bytes) -> Self {
        serde_yaml::from_slice(value.iter().as_slice()).expect("failed to deserialize bytes")
    }
}

/// The application's mailbox that handles incoming messages.
///
/// TODO: remove if there are no other actors? I think this is only used for communication between different actors
pub struct Mailbox {
    sender: mpsc::Sender<Message>,
}

impl Mailbox {
    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        Self { sender }
    }
}
