/// This binary prepares the testing setup for two parties that can be
/// playing the battleship game.
use battleship_commonware::config::config::get_config_path;

use clap::{Command, arg};
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt::init();

    let command = Command::new("battleship-commonware-setup").args([
        arg!(--id <ID> "the id to use for the player"), // TODO: eventually we'll want to use a Rng instead of a fixed seed.
        arg!(--"peer-endpoint" <PEER_ENDPOINT> "the peer to connect with for the game"),
        arg!(--"peer-public-key" <PEER_PK> "the other player's public key"),
    ]);

    let args = command.get_matches();
    let id = args
        .get_one::<String>("id")
        .expect("must set --id")
        .parse::<u16>()
        .expect("failed to parse id");

    let peer_endpoint = args
        .get_one::<String>("peer-endpoint")
        .expect("must set --peer-endpoint");

    let peer_public_key = args
        .get_one::<String>("peer-public-key")
        .expect("must set --peer-public-key");

    let config = battleship_commonware::Config::new(id, peer_endpoint, peer_public_key);
    config.validate().expect("invalid config");

    config
        .export(&get_config_path(id))
        .expect("failed to export config");
}
