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
/// The game uses LLM models (via the `parrot` crate) to make strategic moves
/// instead of random attacks. The LLM analyzes past moves and suggests the next
/// tactical move to play.
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
/// RUST_LOG=info cargo run --bin setup -- --private-key b2f7f581d6de3c06a822fd6e7e8265fbc00f8401696a5bdc34f5a6d2ff3f922f --port 5670 --peer-endpoint="127.0.0.1:5671" --peer-public-key="478b8e507e0bb2b18c0f9e0824769e8562d10df9abe2e774896f82b4b4405266"
/// ```
///
/// ```shell
/// RUST_LOG=info cargo run --bin setup -- --private-key 9a3744504560639ec670b7a17d492b273e077b0a96bef58ba7760779e544546e --port 5671 --peer-endpoint="127.0.0.1:5670" --peer-public-key="edd0f6de342a1e6a7236d6244f23d83eedfcecd059a386c85055701498e77033"
/// ```
///
/// After generating the configuration files, the actual game logic can be started.
/// To do so, run the `player` binary:
///
/// ```shell
/// RUST_LOG=info cargo run --bin player -- --public-key edd0f6de342a1e6a7236d6244f23d83eedfcecd059a386c85055701498e77033
/// ```
///
/// ```shell
/// RUST_LOG=info cargo run --bin player -- --public-key 478b8e507e0bb2b18c0f9e0824769e8562d10df9abe2e774896f82b4b4405266
/// ```
///
/// ## Implementation Steps
///
/// - Start a simple setup where commonware-p2p communicates between two nodes (two keys).
///   They should send simple messages with an increasing counter variable
///   for starters.
///
///   - This can use hardcoded information first.
///
///   - Next, it should be extended to take input arguments
///     to define the key of the running instance.
///     Real projects mostly use clap for this so that can be added here, too.
///     The keys themselves could be added to e.g. a local config file and then parsed.
///
/// -  Ultimately, there should be some connection request logic implemented,
///    where a new peer is only accepted in case there is not an established peer
///    connection already, and the peer is in a list of whitelisted addresses.
///
/// - This can be extended to incorporate moves for the battleship game.
///   As a first iteration, just shoot at increasing fields A1, B1, ... .
///
/// - Finally, e.g. using hashing operations, the players could play
///   against each other automatically and one could watch in the terminal.
///
/// - There should be simulations added which make use of the deterministic runtime while
///   the main application runs on the tokio runtime of the Commonware framework.
///
/// - Eventually, it could be cool to store the game state using `commonware-storage` to understand
///   that crate better as well.
///
pub mod application;
pub mod config;
pub mod game;
pub mod gui;

pub use config::{Config, get_config_path};
