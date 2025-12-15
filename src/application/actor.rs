/// The application's actor controls the message flow
/// between the two participating nodes.
use super::{
    gamestate::{Move},
    ingress::{Mailbox, Message},
};

use commonware_cryptography::Signer;
use commonware_p2p::{Receiver, Recipients, Sender};
use commonware_runtime::{ContextCell, Spawner, spawn_cell};
use eyre::Context;
use futures::channel::mpsc;
use rand::{CryptoRng, Rng};
use tracing::{debug, error, info};

// TODO: use bigger mailbox size here?
const MAILBOX_SIZE: usize = 1;

pub struct GameStateActor<R: Rng + CryptoRng + Spawner, C: Signer> {
    context: ContextCell<R>,
    crypto: C,
    namespace: Vec<u8>,
    mailbox: mpsc::Receiver<Message>,

    // Game logic (TODO: refactor to GameState struct)

    /// Signals if the node is ready.
    is_ready: bool,

    /// Signals if the oppenent is ready to start the game.
    opponent_ready: bool,

    /// Signals if it's the actor's turn (or the opponent's turn if false).
    my_turn: bool,

    /// The list of the exchanged moves.
    moves: Vec<Move>
}

impl<R: Rng + CryptoRng + Spawner, C: Signer> GameStateActor<R, C> {
    /// Create new application actor.
    pub fn new(context: R, crypto: C) -> (Self, Mailbox) {
        let (sender, mailbox) = mpsc::channel(MAILBOX_SIZE);
        (
            Self {
                context: ContextCell::new(context),
                crypto,
                namespace: Vec::from("GAMESTATE_NAMESPACE"),
                mailbox,

                // Game logic

                is_ready: false,
                opponent_ready: false,
                my_turn: false,
                moves: Vec::new(),
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
        mut receiver: impl Receiver<PublicKey = C::PublicKey>,
    ) {
        for _ in 0..20 {
            debug!("ready: {}", self.is_ready);
            debug!("opponent ready: {}", self.opponent_ready);
            if !self.is_ready {
                info!("not ready yet; sending ready message");

                let _ = self.send(sender.clone(), Message::Ready).await.expect("failed to send ready message");
                self.is_ready = true;

                continue
            }

            // We're waiting to receive an incoming 
            info!("waiting for receiving message on receiver");
            let deserialized = match receiver.recv().await {
                Ok((_, msg)) => Message::from(msg),
                Err(_) => {
                    error!("failed to receive message");
                    continue
                }
            };

            match deserialized {
                Message::Attack{ m: _ } => {
                    if !self.opponent_ready {
                        error!("opponent not marked as ready yet; can't process attack");
                        continue
                    }

                    info!("handling attack: {:?}", deserialized);
                    self.handle_attack(deserialized).expect("failed to handle attack");
                    self.my_turn = !self.my_turn;
                },
                Message::Ready => self.opponent_ready = true,
                _ => unimplemented!("other message types received")
            }

            // TODO: is clone fine here for the sender? should be since it's mpsc so multiple producers should be fine.
            if self.my_turn {
                let _ = &self.attack(sender.clone()).await.expect("failed to attack");
            }

            continue;
        }

        info!("iterations passed");
    }

    /// Sends a computed move for the game via the p2p layer.
    async fn attack(&mut self, sender: impl Sender<PublicKey = C::PublicKey>) -> eyre::Result<()> {
        info!("attacking");

        // TODO: compute the moves e.g. based on the hash of the game state or something.
        let x = 0;
        let y = 0;

        let msg = Message::Attack{ m: Move::new(
            self.next_move(),
            self.crypto.public_key().to_string(),
            x,
            y,
        )};

        info!("sending sink message: {:?}", msg);
        self.send(sender, msg).await
    }

    /// Updates the internal game state when receiving an incoming message.
    fn handle_attack(
        &mut self,
        message: Message,
    ) -> eyre::Result<()> {
        if let Err(e) = (&message).validate() {
            return Err(e);
        }

        match message {
            Message::Attack { m } => {
                // TODO: here the game state should be updated.
                self.moves.push(m);
                Ok(())
            },
            _ => Err(eyre::eyre!("wrong message type"))
        }
    }

    fn next_move(&self) -> u8 {
        self.moves[self.moves.len()-1].get_number() + 1
    }

    /// Sends a given message to all recipients.
    async fn send(
        &mut self,
        mut sender: impl Sender<PublicKey = C::PublicKey>,
        message: Message,
    ) -> eyre::Result<()> {
        if let Err(e) = sender.send(Recipients::All, message.into(), false).await {
            Err(e).wrap_err("failed to send message")
        } else {
            Ok(())
        }
    }
}
