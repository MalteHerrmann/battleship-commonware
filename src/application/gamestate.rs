use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Move {
    /// The move number in the game.
    number: u8,
    /// The public key of the player.
    player: String,
    /// The x-coordinate that is attacked.
    x: u8,
    /// The y-coordinate that is attacked.
    y: u8,
}

impl Move {
    pub fn new(number: u8, player: String, x: u8, y: u8) -> Self {
        Self {
            number,
            player,
            x,
            y,
        }
    }

    pub fn get_number(&self) -> u8 {
        self.number
    }

    pub fn get_x(&self) -> u8 {
        self.x
    }

    pub fn get_y(&self) -> u8 {
        self.y
    }
}
