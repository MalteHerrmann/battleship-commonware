/// This binary prepares the testing setup for two parties that can be
/// playing the battleship game.
use battleship_commonware::config::{get_config_path, parse_private_key};

use clap::{Command, arg};
use commonware_cryptography::Signer;

fn main() {
    tracing_subscriber::fmt::init();

    let command = Command::new("battleship-commonware-setup").args([
        arg!(--"private-key" <PK> "the private key to use for this player"),
        arg!(--port <PORT> "the network port to use for this player"),
        arg!(--"peer-endpoint" <PEER_ENDPOINT> "the peer to connect with for the game"),
        arg!(--"peer-public-key" <PEER_PK> "the other player's public key"),
    ]);

    let args = command.get_matches();
    let private_key = parse_private_key(
        args.get_one::<String>("private-key")
            .expect("must set --private-key"),
    )
    .expect("failed to parse private key");

    let port = args
        .get_one::<String>("port")
        .expect("must set --port")
        .parse::<u16>()
        .expect("invalid port");

    let peer_endpoint = args
        .get_one::<String>("peer-endpoint")
        .expect("must set --peer-endpoint");

    let peer_public_key = args
        .get_one::<String>("peer-public-key")
        .expect("must set --peer-public-key");

    let config =
        battleship_commonware::Config::new(&private_key, port, peer_endpoint, peer_public_key);
    config.validate().expect("invalid config");

    config
        .export(&get_config_path(&private_key.public_key()))
        .expect("failed to export config");
}
