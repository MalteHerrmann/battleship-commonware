/// # Battleship Commonware
/// 
/// This crate contains a simple implementation
/// of the battleship Rust implementation by Orhun on GitHub:
/// https://github.com/orhun/battleship-rs.
///
/// This is intended to be a hands-on exercise of integrating
/// an existing codebase in Rust with the Commonware set of
/// utilities.
/// 
/// ## Usage
/// 
/// It is required to set up two players, that will connect via P2P.
/// To generate the corresponding configurations, first we can generate
/// two private/public key pairs for each player:
/// 
/// ```shell
/// cargo run --bin keys -- --id 0
/// ```
/// 
/// ```shell
/// cargo run --bin keys -- --id 1
/// ```
/// 
/// This will output a public and private key for each of the players,
/// which can then be used to create the full configuration per player:
/// 
/// ```shell
/// cargo run --bin setup -- --id 0 --port 5670 --peer-endpoint="127.0.0.1:5671" --peer-public-key="478b8e507e0bb2b18c0f9e0824769e8562d10df9abe2e774896f82b4b4405266"
/// ```
/// 
/// ```shell
/// cargo run --bin setup -- --id 1 --port 5671 --peer-endpoint="127.0.0.1:5670" --peer-public-key="edd0f6de342a1e6a7236d6244f23d83eedfcecd059a386c85055701498e77033"
/// ```
/// 
/// After generating the configuration files, the actual game logic can be started.
/// To do so, run the `player` binary:
/// 
/// ```shell
/// cargo run --bin player -- --id 0
/// ```
/// 
/// ```shell
/// cargo run --bin player -- --id 1
/// ```
///
/// ## Implementation Steps
///
/// 1. Start a simple setup where commonware-p2p communicates between two nodes (two keys).
///    They should send simple messages with an increasing counter variable
///    for starters.
///
///    - [x] This can use hardcoded information first.
///
///    - [x] Next, it should be extended to take input arguments
///          to define the key of the running instance.
///          Real projects mostly use clap for this so that can be added here, too.
///          The keys themselves could be added to e.g. a local config file and then parsed.
///
///    - [ ] Ultimately, there should be some connection request logic implemented,
///          where a new peer is only accepted in case there is not an established peer
///          connection already, and the peer is in a list of whitelisted addresses.
///
/// 2. This can be extended to incorporate moves for the battleship game.
///    As a first iteration, just shoot at increasing fields A1, B1, ... .
///
/// 3. Finally, e.g. using hashing operations, the players could play
///    against each other automatically and one could watch in the terminal.
///
/// 4. There should be simulations added which make use of the deterministic runtime while
///    the main application runs on the tokio runtime of the Commonware framework.
///
pub mod application;
pub mod config;

pub use config::config::{Config, get_config_path};
