use serde::{Deserialize, Serialize};

use crate::game::{self, Coordinate};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Move {
    /// The move number in the game.
    number: u16,
    /// The x-coordinate that is attacked.
    x: u8,
    /// The y-coordinate that is attacked.
    y: u8,
    /// Specifies if the played move was successful or not.
    pub is_hit: bool,
}

impl Move {
    // TODO: unify with `Coordinate` from the `game` implementation?
    pub fn new(number: u16, x: u8, y: u8, is_hit: bool) -> Self {
        Self {
            number,
            x,
            y,
            is_hit,
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

    // TODO: use coordinate in this struct instead of x and y?
    pub fn get_position(&self) -> String {
        format!("{}", Coordinate::new(self.x, self.y, self.is_hit))
    }

    pub fn validate(&self) -> eyre::Result<()> {
        if self.x > game::GRID_SIZE || self.y > game::GRID_SIZE {
            return Err(eyre::eyre!("invalid move: {}-{}", self.x, self.y));
        }

        Ok(())
    }
}
