/// The application's actor controls the message flow
/// between the two participating nodes.
use super::{
    gamestate::Move,
    ingress::{Mailbox, Message},
};

use commonware_cryptography::Signer;
use commonware_macros::select;
use commonware_p2p::{Receiver, Recipients, Sender};
use commonware_runtime::{ContextCell, Spawner, spawn_cell};
use eyre::Context;
use futures::channel::mpsc;
use rand::{CryptoRng, Rng};
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info};

// TODO: use bigger mailbox size here?
const MAILBOX_SIZE: usize = 1;

pub struct GameStateActor<R: Rng + CryptoRng + Spawner, C: Signer> {
    context: ContextCell<R>,
    crypto: C,
    // TODO: remove if not used?
    namespace: Vec<u8>,
    // TODO: what should mailbox be used for again? currently not in use.. is this only for messages to the actor from other actors in a more complex setup?
    mailbox: mpsc::Receiver<Message>,

    /// Signals if the game is ready to start.
    game_ready: bool,

    /// Signals if it's the actor's turn (or the opponent's turn if false).
    my_turn: bool,

    /// The list of the exchanged moves.
    moves: Vec<Move>,
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
                game_ready: false,
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
        // TODO: should this really use an unbounded loop here? Or do smth. like `while let Some(msg) = self.mailbox.next()`? But it's not guaranteed that this actor will immediately get a message sent to it...
        // TODO: should there e.g. be two separate actors? One that does the connection and the P2P stuff? And the other one that's just implemented the game logic? I guess this could be implemented at a later point but isn't required for it to run.

        loop {
            select! {
                // We're waiting to receive an incoming message from the opponent
                msg = receiver.recv() => {
                    match msg {
                        Ok((_, message_bytes)) => {
                            self.handle_message(
                                sender.clone(),
                                Message::from(message_bytes)
                            ).await
                            .expect("failed to handle message");
                        },
                        Err(_) => error!("failed to receive message"),
                    }
                },
                _ = sleep(Duration::from_secs(10)) => {
                    if !self.game_ready {
                        info!("game not ready yet; sending ready message to other player");

                        let _ = self
                            .send(sender.clone(), Message::Ready)
                            .await
                            .expect("failed to send ready message");

                        self.my_turn = true; // we're having the first sender of the ready message have the first turn.

                    } else if self.my_turn {
                        let _ = &self.attack(sender.clone()).await.expect("failed to attack");
                    }
                }
                // TODO: do we need to check for messages in self.mailbox.next() here? where should that plug in? currently there's nothing sending to the mailbox?
            }
        }
    }

    /// Sends a computed move for the game via the p2p layer.
    async fn attack(&mut self, sender: impl Sender<PublicKey = C::PublicKey>) -> eyre::Result<()> {
        // TODO: compute the moves e.g. based on the hash of the game state or something.
        let x = 0;
        let y = 0;

        let current_move = Move::new(self.next_move(), self.crypto.public_key().to_string(), x, y);

        let msg = Message::Attack {
            m: current_move.clone(),
        };

        info!("sending sink message: {:?}", msg);
        self.send(sender, msg).await?;

        self.moves.push(current_move);
        self.my_turn = false;

        Ok(())
    }

    /// Updates the internal game state when receiving an incoming message.
    fn handle_attack(&mut self, message: Message) -> eyre::Result<()> {
        message.validate()?;

        match message {
            Message::Attack { m } => {
                // we're only allowing monotonically increasing numbers, incremented by 1, here
                if m.get_number() as usize != self.moves.len() + 1 {
                    Err(eyre::eyre!("invalid move number: {}; expected: {}", m.get_number(), self.moves.len()+1))
                } else {
                    self.moves.push(m);
                    self.my_turn = true;
                    info!("setting my turn to true");

                    Ok(())
                }
            }
            _ => Err(eyre::eyre!("wrong message type")),
        }
    }

    /// This method implements the main application logic for any incoming messages.
    /// This includes the attacks, information about player readiness, as well as the message
    /// to communicate the game ending.
    async fn handle_message(&mut self, sender: impl Sender<PublicKey = C::PublicKey>, msg: Message) -> eyre::Result<()> {
        match msg {
            Message::Attack { m: _ } => {
                if !self.game_ready {
                    return Err(eyre::eyre!("opponent not marked as ready yet; can't process attack"));
                }

                info!("handling attack: {:?}", msg);
                self.handle_attack(msg)
                    .expect("failed to handle attack");
            },
            Message::Ready => {
                info!("received ready message");

                assert!(!self.game_ready, "game should not be ready when receiving ready message from other player!");

                // We're sending a Ready message back so that the opponent is also informed of our readiness.
                // In case, `self.my_turn` is already true, this means that the received `Ready` message is the
                // other players' response to our initial readiness communication, so there is no need to again
                // signal readiness via P2P.

                if !self.my_turn {
                    info!("sending ready message back");
                    let _ = self
                        .send(sender.clone(), Message::Ready)
                        .await
                        .expect("failed to send ready message");
                }

                self.game_ready = true;
            },
            Message::EndGame { winner: _ } => unimplemented!("end game logic not yet implemented"),
            _ => unimplemented!("other message types received"),
        }

        Ok(())
    }

    /// Incrememnts the latest seen move number to yield the next number to play.
    fn next_move(&self) -> u8 {
        if self.moves.is_empty() {
            return 1;
        }

        self.moves[self.moves.len() - 1].get_number() + 1
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
