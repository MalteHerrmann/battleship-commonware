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
///    - [x] This can use hardcoded information first.
///
///    - [ ] Next, it should be extended to take input arguments
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
