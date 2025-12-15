/// The application's actor controls the message flow
/// between the two participating nodes.
use super::ingress::{Mailbox, Message};

use commonware_cryptography::Signer;
use commonware_p2p::{Receiver, Recipients, Sender};
use commonware_runtime::{ContextCell, Spawner, spawn_cell};
use futures::channel::mpsc;
use rand::{CryptoRng, Rng};

// TODO: use bigger mailbox size here?
const MAILBOX_SIZE: usize = 1;

pub struct GameStateActor<R: Rng + CryptoRng + Spawner, C: Signer> {
    context: ContextCell<R>,
    crypto: C,
    last_seen_move: u16,
    namespace: Vec<u8>,
    mailbox: mpsc::Receiver<Message>,
}

impl<R: Rng + CryptoRng + Spawner, C: Signer> GameStateActor<R, C> {
    /// Create new application actor.
    pub fn new(context: R, crypto: C) -> (Self, Mailbox) {
        let (sender, mailbox) = mpsc::channel(MAILBOX_SIZE);
        (
            Self {
                context: ContextCell::new(context),
                crypto,
                last_seen_move: 0,
                namespace: Vec::from("NAMESPACE"),
                mailbox,
            },
            Mailbox::new(sender),
        )
    }

    pub fn start(
        mut self,
        sender: impl Sender<PublicKey = C::PublicKey>,
        receiver: impl Receiver<PublicKey = C::PublicKey>,
    ) {
        spawn_cell!(self.context, self.run(sender, receiver).await);
    }

    pub async fn run(
        mut self,
        sender: impl Sender<PublicKey = C::PublicKey>,
        receiver: impl Receiver<PublicKey = C::PublicKey>,
    ) {
        loop {
            // TODO: here goes the handling of incoming and outgoing messages
            println!("round done");
            // TODO: is clone fine here for the sender? should be since it's mpsc so multiple producers should be fine.
            &self.send_move(sender.clone()).await;

            continue;
        }
    }

    /// Sends a computed move for the game via the p2p layer.
    async fn send_move(&mut self, sender: impl Sender<PublicKey = C::PublicKey>) {
        let x = 0;
        let y = 0;

        sender.send(Recipients::All, Message::Sink { x, y }.into(), false);
    }
}
