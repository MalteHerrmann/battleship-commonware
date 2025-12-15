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
    EndGame { winner: String },
    /// Signals to the other peer that the player is ready.
    Ready,
}

impl Message {
    /// Validates the contents of the message.
    pub fn validate(&self) -> eyre::Result<()> {
        match self {
            Message::Attack{ m } => {
                if (&m).get_x() > 8 || (&m).get_y() > 8 {
                    return Err(eyre::eyre!("invalid target: {}-{}", (&m).get_x(), (&m).get_y()));
                }
            },
            Message::EndGame { winner } => {
                if winner.is_empty() {
                    // TODO: return error instead of panic
                    return Err(eyre::eyre!("winner cannot be empty"));
                }
            }
            Message::Ready => (),
        }

        Ok(())
    }
}

impl Into<bytes::Bytes> for Message {
    fn into(self) -> bytes::Bytes {
        let serialized = serde_json::to_string(&self)
            .expect("failed to serialize message");

        bytes::Bytes::from(serialized.into_bytes())
    }
}

impl From<bytes::Bytes> for Message {
    fn from(value: bytes::Bytes) -> Self {
        serde_json::from_slice(value.iter().as_slice())
            .expect("failed to deserialize bytes")
    }
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
