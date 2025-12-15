/// This crate contains a simple implementation
/// of the battleship Rust implementation by Orhun on GitHub:
/// https://github.com/orhun/battleship-rs.
///
/// This is intended to be a hands-on exercise of integrating
/// an existing codebase in Rust with the Commonware set of
/// utilities.
///
/// # Implementation Steps
///
/// 1. Start a simple setup where commonware-p2p communicates between two nodes (two keys).
///    They should send simple messages with an increasing counter variable
///    for starters.
///
///    - This can use hardcoded information first but should be extended to take input arguments
///      to define the key of the running instance.
///    - Real projects mostly use clap for this so that can be added here, too.
///
/// 2. This can be extended to incorporate moves for the battleship game.
///    As a first iteration, just shoot at increasing fields A1, B1, ... .
///
/// 3. Finally, e.g. using hashing operations, the players could play
///    against each other automatically and one could watch in the terminal.
///
mod application;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use bytes::Bytes;
use commonware_cryptography::{PrivateKeyExt as _, Signer, ed25519};
use commonware_p2p::{Manager, Sender, authenticated::discovery};
use commonware_runtime::{Metrics, Runner, deterministic};
use commonware_utils::NZU32;
use governor::Quota;

const ENDPOINT_1: u16 = 5670;
const ENDPOINT_2: u16 = 5671;
const MAX_MESSAGE_SIZE: u16 = 1024;

fn main() {
    // We're creating the private keys here that will communicate over the p2p
    // connection, in order to exchange messages about the intended moves in the game.
    let signer = ed25519::PrivateKey::from_seed(0);
    let peer1 = ed25519::PrivateKey::from_seed(1).public_key();

    let bootstrappers = vec![(
        peer1.clone(),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), ENDPOINT_2),
    )];

    // The p2p setup uses the local config for this proof-of-concept.
    // TODO: does this have to be adjusted to check which key is running the binary?
    // we'll have to support running two instances via CLI flags.
    println!("setting up p2p config");
    let p2p_config = discovery::Config::local(
        signer.clone(),
        b"BATTLESHIP_NAMESPACE",
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), ENDPOINT_1.into()),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), ENDPOINT_1.into()),
        bootstrappers,
        MAX_MESSAGE_SIZE.into(),
    );

    // TODO: use tokio runner here - ideally abstract at some point to use the deterministic runner in tests / simulations (should be good for debugging).
    let runner_config = deterministic::Config::new()
        .with_seed(0)
        .with_timeout(Some(Duration::from_secs(10)));

    let executor = deterministic::Runner::new(runner_config);

    executor.start(|context| async move {
        let (mut network, mut oracle) =
            discovery::Network::new(context.with_label("network"), p2p_config);

        // We set the peers in the oracle (which in the context of commonware-p2p is
        // the central entity to manage the list of connected peers).
        let peers_index = 0;
        oracle.update(peers_index, vec![peer1].into()).await;

        // This registes the channel over which communication
        // about the game state will be implemented.
        let (gamestate_sender, gamestate_receiver) =
            network.register(0, Quota::per_second(NZU32!(1)), 1);

        // Here we're setting up the actor that updates the game state.
        // After the initial setup we have to start the actor, providing
        // the registered channels for p2p communication.
        let (gamestate_actor, _gamestate_mailbox) =
            application::actor::GameStateActor::new(context, signer);

        gamestate_actor.start(gamestate_sender, gamestate_receiver);

        // // NOTE: This currently uses ::All but ::One would also work since there's
        // // only one peer.
        // //
        // // TODO: refactor this to be sent upon calling a different command, to control the game.
        // // TODO: this should be sent in a different process?
        // let message_bytes = Bytes::from_static(b"hello");
        // gamestate_sender
        //     .send(commonware_p2p::Recipients::All, message_bytes, false)
        //     .await
        //     .expect("failed to send via p2p");

        network.start().await.expect("Network failed");
    });

    println!("done.");
}
