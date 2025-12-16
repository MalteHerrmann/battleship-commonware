use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    time::Duration,
};

use battleship_commonware::{
    Config, application::actor::GameStateActor, config::config::parse_public_key, get_config_path,
};

use clap::arg;
use commonware_p2p::{Manager, authenticated::discovery};
use commonware_runtime::{Metrics, Runner, tokio};
use commonware_utils::NZU32;
use governor::Quota;
use tracing::info;

const MAX_MESSAGE_SIZE: u16 = 1024;

fn main() {
    // Initialize the tracing subscriber to print to stdout.
    tracing_subscriber::fmt::init();

    let command = clap::Command::new("battleship-commonware-player")
        .args([arg!(--id <ID> "the player id (decides which config to use)")]);

    let args = command.get_matches();
    let id = args
        .get_one::<String>("id")
        .expect("must provide --id")
        .parse::<u16>()
        .expect("id must be valid u16");

    // We're creating the private keys here that will communicate over the p2p
    // connection, in order to exchange messages about the intended moves in the game.

    let config = Config::read(&get_config_path(id)).expect("failed to read config");

    let peer_public_key = parse_public_key(&config.peer_public_key).expect("invalid public key");
    let signer = config.get_private_key();

    let bootstrappers = vec![(
        peer_public_key.clone(),
        SocketAddr::from_str(&config.peer_endpoint).expect("invalid peer endpoint"),
    )];

    // The p2p setup uses the local config for this proof-of-concept.
    //
    // TODO: does this have to be adjusted to check which key is running the binary?
    // we'll have to support running two instances via CLI flags.
    info!("setting up p2p config");
    let p2p_config = discovery::Config::local(
        signer.clone(),
        b"BATTLESHIP_NAMESPACE",
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.port.into()),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), config.port.into()),
        bootstrappers,
        MAX_MESSAGE_SIZE.into(),
    );

    // // TODO: ideally abstract at some point to use the deterministic runner in tests / simulations (should be good for debugging).
    // let runner_config = deterministic::Config::new()
    //     .with_seed(0)
    //     .with_timeout(Some(Duration::from_secs(10)));

    let runner_config = tokio::Config::new().with_read_write_timeout(Duration::from_secs(10));

    let executor = tokio::Runner::new(runner_config);

    executor.start(|context| async move {
        let (mut network, mut oracle) =
            discovery::Network::new(context.with_label("network"), p2p_config);

        // We set the peers in the oracle (which in the context of commonware-p2p is
        // the central entity to manage the list of connected peers).
        let peers_index = 0;
        oracle
            .update(peers_index, vec![peer_public_key].into())
            .await;

        // This registes the channel over which communication
        // about the game state will be implemented.
        let (gamestate_sender, gamestate_receiver) =
            network.register(0, Quota::per_second(NZU32!(1)), 1);

        // Here we're setting up the actor that updates the game state.
        // After the initial setup we have to start the actor, providing
        // the registered channels for p2p communication.
        //
        // TODO: where to use the `gamestate_mailbox`? Shouldn't that be used to be passed into the start method maybe?
        // do we even need the mailbox? Isn't that only for the case where the game state actor is passed into another actor?
        let (gamestate_actor, _gamestate_mailbox) = GameStateActor::new(context, signer.clone());

        gamestate_actor.start(gamestate_sender, gamestate_receiver);

        network.start().await.expect("Network failed");
    });

    info!("done.");
}
