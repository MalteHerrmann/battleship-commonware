//! Player.
//!
//! NOTE: This implementation is based on https://github.com/orhun/battleship-rs.
//! It has been adapted to make use of the Commonware components.

use super::grid::Coordinate;
use super::grid::{GRID_SIZE, Grid};

/// Representation of a player.
#[derive(Debug)]
pub struct Player {
    /// Player's grid.
    pub grid: Grid,
    /// The opponent's grid, as viewed by the attacker (i.e. empty except for hits or misses marked).
    pub opponent_grid: Grid,
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    /// Constructs a new instance of [`Player`].
    pub fn new() -> Self {
        Self {
            grid: Grid::new_random(GRID_SIZE, GRID_SIZE),
            opponent_grid: Grid::new(GRID_SIZE, GRID_SIZE),
        }
    }

    /// Sets a given coordinate (which MUST include the hit or miss information)
    /// in the locally stored opponent grid.
    pub fn attack(&mut self, x: u8, y: u8, is_hit: bool) -> eyre::Result<()> {
        let coord = Coordinate::new(x, y, is_hit);
        self.opponent_grid.mark_hit(&coord)
    }

    // TODO: check if this is correct??
    pub fn handle_attack(&mut self, x: u8, y: u8) -> bool {
        let expected = Coordinate::new(x, y, false);

        if let Some(coordinate) = self
            .grid
            .ships
            .iter_mut()
            .find(|ship| ship.coords.contains(&expected))
            .and_then(|ship| ship.coords.iter_mut().find(|c| *c == &expected))
        {
            coordinate.is_hit = true;
            return true;
        }

        false
    }

    /// Checks if the player has lost the game.
    /// 
    /// This is the case if all coordinates of all placed ships have
    /// been hit.
    pub fn lost(&self) -> bool {
        self
            .grid
            .ships
            .iter()
            .all(|ship| ship.coords.iter().all(|c| c.is_hit))
    }

    fn print(&self, input: &str) -> eyre::Result<()> {
        println!("{}", input);
        Ok(())
    }

    /// Shows the player's own view of his ships.
    pub fn print_grid(&self) -> eyre::Result<()> {
        let grid_str = self.grid.as_string(true)?;
        self.print(&grid_str)?;
        Ok(())
    }

    /// Prints the visual output for the made attacks (including hits or misses).
    pub fn print_attacks(&self) -> eyre::Result<()> {
        let grid_str = self.opponent_grid.as_string(false)?;
        self.print(&grid_str)
    }
}
