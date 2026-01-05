# Battleship Commonware

This repository contains an example implementation of Battleship
built on top of Commonware primitives.
It serves as an exercise to familiarize myself with the framework.
The Battleship logic is using https://github.com/orhun/battleship-rs.

The game uses LLM models (via the `parrot` crate) to make strategic moves
instead of random attacks, making it an AI-powered battleship game.

## Usage

Concrete usage instructions can be found in the Rust doc strings in [src/lib.rs](./src/lib.rs).

**Note**: This implementation requires an LLM model to be available via the `parrot` crate.
The game will automatically select an available model at runtime to compute moves.
