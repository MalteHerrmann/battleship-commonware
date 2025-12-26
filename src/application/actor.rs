/// The application's actor controls the message flow
/// between the two participating nodes.
use crate::game::{self, GRID_SIZE};
use crate::gui::{Mailbox as GuiMailbox, Message as GuiMessage};

use super::{
    gamestate::Move,
    ingress::{Mailbox, Message},
};

use commonware_cryptography::Signer;
use commonware_macros::select;
use commonware_p2p::{Receiver, Recipients, Sender};
use commonware_runtime::{ContextCell, Spawner, spawn_cell};
use eyre::Context;
use futures::SinkExt;
use futures::channel::mpsc;
use rand::{CryptoRng, Rng};
use tokio::time::{Duration, sleep};

// TODO: use bigger mailbox size here?
const MAILBOX_SIZE: usize = 1;

/// The main actor that drives the communication between the participants,
/// while maintaining track of the game state internally.
///
/// TODO: I guess the [crate::game::Game] could be made into its own actor
/// as well and then receive driving updates through the channels.
pub struct GameStateActor<R: Rng + CryptoRng + Spawner, C: Signer> {
    context: ContextCell<R>,
    crypto: C,
    // TODO: remove if not used?
    namespace: Vec<u8>,
    // TODO: what should mailbox be used for again? currently not in use.. is this only for messages to the actor from other actors in a more complex setup?
    mailbox: mpsc::Receiver<Message>,

    gui_mailbox: GuiMailbox,

    /// Signals if the player is ready to start.
    is_ready: bool,

    /// Signals if the opponent is ready to start.
    opponent_ready: bool,

    /// Signals if it's the actor's turn (or the opponent's turn if false).
    my_turn: bool,

    /// The list of the exchanged moves.
    /// TODO: change to hashmap
    moves: Vec<Move>,

    /// The list of the opponent's moves.
    /// TODO: change to hashmap
    ///
    /// TODO: should this be moved to the `Grid` implementation or the `Player`?
    opponent_moves: Vec<Move>,

    /// The game state (local to the actor).
    game: game::Player,
}

impl<R: Rng + CryptoRng + Spawner, C: Signer> GameStateActor<R, C> {
    /// Create new application actor.
    pub fn new(context: R, gui_mailbox: GuiMailbox, crypto: C) -> (Self, Mailbox) {
        let (sender, mailbox) = mpsc::channel(MAILBOX_SIZE);
        (
            Self {
                context: ContextCell::new(context),
                crypto,
                namespace: Vec::from("GAMESTATE_NAMESPACE"),
                mailbox,

                gui_mailbox,

                // Game logic
                my_turn: false,

                is_ready: false,
                moves: Vec::new(),

                opponent_ready: false,
                opponent_moves: Vec::new(),

                game: game::Player::new(),
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
                        Err(_) => self.log("failed to receive message")
                            .await
                            .expect("failed to log"),
                    }
                },
                _ = sleep(Duration::from_secs(4)) => {
                    if !self.game_ready() {
                        self.log("game not ready yet; sending ready message to other player")
                            .await
                            .expect("failed to log");

                        self
                            .send(sender.clone(), Message::Ready)
                            .await
                            .expect("failed to send ready message");

                        self.my_turn = true; // we're having the first sender of the ready message have the first turn.
                        self.is_ready = true;

                    } else if self.my_turn {
                        let _ = &self.attack(sender.clone()).await.expect("failed to attack");
                    }
                }
                // TODO: do we need to check for messages in self.mailbox.next() here? where should that plug in? currently there's nothing sending to the mailbox?
            }
        }
    }

    /// Sends a computed move for the game via the p2p layer.
    ///
    /// TODO: should this take in the sender and receiver? Or rather use the mailbox of the actor here?
    async fn attack(&mut self, sender: impl Sender<PublicKey = C::PublicKey>) -> eyre::Result<()> {
        let mut unused = false;
        let mut x: u8 = 0;
        let mut y: u8 = 0;

        while !unused {
            x = fastrand::u8(0..=GRID_SIZE);
            y = fastrand::u8(0..=GRID_SIZE);
            self.log(&format!("generated new attack point: ({},{})", x, y)).await?;

            unused = !self.moves.iter().any(|m| m.get_x() == x && m.get_y() == y)
        }

        let current_move = Move::new(self.next_move(), self.crypto.public_key().to_string(), x, y);

        let msg = Message::Attack {
            m: current_move.clone(),
        };

        self.log(&format!("sending attack message: {:?}", msg)).await?;
        self.send(sender, msg).await?;

        self.moves.push(current_move);
        self.my_turn = false;

        Ok(())
    }

    async fn draw_grid(&mut self) -> eyre::Result<()> {
        let full_grid = [
            self.game.opponent_grid.as_string(false)?,
            self.game.grid.as_string(true)?,
        ].join("\n");

        self.gui_mailbox.sender.send(GuiMessage::Draw { grid: full_grid }).await?;

        Ok(())
    }

    /// Checks if the game is ready to be played.
    fn game_ready(&self) -> bool {
        self.is_ready && self.opponent_ready
    }

    /// Updates the internal game state when receiving an incoming message.
    async fn handle_attack(
        &mut self,
        message: Message,
        sender: impl Sender<PublicKey = C::PublicKey>,
    ) -> eyre::Result<()> {
        message.validate()?;

        match message {
            Message::Attack { m } => {
                // we're only allowing monotonically increasing numbers, incremented by 1, here
                if m.get_number() as usize != self.moves.len() + self.opponent_moves.len() + 1 {
                    return Err(eyre::eyre!(
                        "invalid move number: {}; expected: {}",
                        m.get_number(),
                        self.moves.len() + self.opponent_moves.len() + 1
                    ));
                }

                // Check if the move was already played.
                if self
                    .opponent_moves
                    .iter()
                    .any(|previous| m.get_x() == previous.get_x() && m.get_y() == previous.get_y())
                {
                    return Err(eyre::eyre!("move already played"));
                }

                self.opponent_moves.push(m.clone());
                let is_hit = self.game.handle_attack(m.get_x(), m.get_y());
                self.my_turn = true;

                // Upon handling an attack we're sending the instruction
                // for the GUI actor to draw the grids.
                self.draw_grid().await?;

                // we're sending the message back with the information if the attack was a hit or miss.
                let _ = match is_hit {
                    true => {
                        self.log(&format!("ship was hit: {}-{}", m.get_x(), m.get_y())).await?;
                        self.send(sender.clone(), Message::Hit { m: m.clone() })
                            .await?;
                        if self.game.lost() {
                            self.send(sender, Message::EndGame).await?;
                            panic!("lost the game!")
                        }

                        Ok(())
                    }
                    false => self.send(sender, Message::Miss { m }).await,
                };

                Ok(())
                // self.game.print_attacks()?;
                // self.game.print_grid()
            }
            _ => Err(eyre::eyre!("wrong message type")),
        }
    }

    /// This method implements the main application logic for any incoming messages.
    /// This includes the attacks, information about player readiness, as well as the message
    /// to communicate the game ending.
    async fn handle_message(
        &mut self,
        sender: impl Sender<PublicKey = C::PublicKey>,
        msg: Message,
    ) -> eyre::Result<()> {
        match msg {
            Message::Attack { m: _ } => {
                if !self.game_ready() {
                    return Err(eyre::eyre!("game not ready yet; can't process attack"));
                }

                self.log(&format!("handling attack: {:?}", msg)).await?;
                self.handle_attack(msg, sender).await?;
            }
            Message::EndGame => panic!("you won the game!"),
            Message::Hit { m } => self.update_opponent_grid(m, true).await?,
            Message::Miss { m } => self.update_opponent_grid(m, false).await?,
            Message::Ready => {
                self.log("received ready message").await?;
                assert!(!self.game_ready(), "game is already marked as ready");

                // We're sending a Ready message back so that the opponent is also informed of our readiness.
                // In case, `self.my_turn` is already true, this means that the received `Ready` message is the
                // other players' response to our initial readiness communication, so there is no need to again
                // signal readiness via P2P.

                if !self.my_turn {
                    self.log("sending ready message back").await?;
                    self.send(sender.clone(), Message::Ready)
                        .await
                        .expect("failed to send ready message");
                }

                self.opponent_ready = true;
                self.is_ready = true;
            }
        }

        Ok(())
    }

    async fn log(&mut self, content: &str) -> eyre::Result<()> {
        self.gui_mailbox.sender.send(
            GuiMessage::Log { content: content.into() }
        )
        .await
        .map_err(|e| e.into())
    }

    /// Incrememnts the latest seen move number to yield the next number to play.
    fn next_move(&self) -> u16 {
        (self.moves.len() + self.opponent_moves.len() + 1) as u16
    }

    /// Sends a given message to all recipients.
    async fn send(
        &mut self,
        mut sender: impl Sender<PublicKey = C::PublicKey>,
        message: Message,
    ) -> eyre::Result<()> {
        self.log(&format!("sending message to peers: {:?}", message)).await?;

        if let Err(e) = sender.send(Recipients::All, message.into(), false).await {
            Err(e).wrap_err("failed to send message")
        } else {
            Ok(())
        }
    }

    /// Update the opponent's grid with a new attack.
    async fn update_opponent_grid(&mut self, mv: Move, is_hit: bool) -> eyre::Result<()> {
        if mv.validate().is_err() {
            return Err(eyre::eyre!("invalid move: {:?}", mv));
        }

        self.log("updating opponent grid").await?;
        self.game.attack(mv.get_x(), mv.get_y(), is_hit)
    }
}
