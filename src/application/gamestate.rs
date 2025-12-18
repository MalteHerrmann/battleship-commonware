use serde::{Deserialize, Serialize};

use crate::game;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Move {
    /// The move number in the game.
    number: u16,
    /// The public key of the player.
    ///
    /// TODO: this really doesn't matter and is not used anywhere.. remove?
    player: String,
    /// The x-coordinate that is attacked.
    x: u8,
    /// The y-coordinate that is attacked.
    y: u8,
}

impl Move {
    pub fn new(number: u16, player: String, x: u8, y: u8) -> Self {
        Self {
            number,
            player,
            x,
            y,
        }
    }

    pub fn get_number(&self) -> u16 {
        self.number
    }

    pub fn get_x(&self) -> u8 {
        self.x
    }

    pub fn get_y(&self) -> u8 {
        self.y
    }

    pub fn validate(&self) -> eyre::Result<()> {
        if self.x > game::GRID_SIZE || self.y > game::GRID_SIZE {
            return Err(eyre::eyre!("invalid move: {}-{}", self.x, self.y));
        }

        Ok(())
    }
}
