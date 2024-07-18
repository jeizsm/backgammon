use crate::Error;
use rand::distributions::{Distribution, Uniform};
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

use super::Player;


/// Represents a players in the game.
#[derive(Debug, Clone)]
pub struct Players {
    player1: PlayerWithDice,
    player2: PlayerWithDice,
    /// The current player
    pub current: PlayerWithDice,
}

impl Players {
    /// Create a new player with a dice
    pub fn new(first_seed: [u8; 32], second_seed: [u8; 32]) -> Self {
        let mut player1 = PlayerWithDice::new(Player::Player0, first_seed);
        let mut player2 = PlayerWithDice::new(Player::Player1, second_seed);
        let player1_dices = player1.roll();
        let player2_dices = player2.roll();
        let current = if player1_dices.values > player2_dices.values {
            player1.dices = Some(player1_dices);
            player1.clone()
        } else {
            player2.dices = Some(player2_dices);
            player2.clone()
        };
        Self {
            player1,
            player2,
            current
        }
    }

    /// Switch the player
    pub fn switch(&mut self) {
        self.current = if self.current.player == self.player1.player {
            self.player1.dices = None;
            self.player2.dices = Some(self.player2.roll());
            self.player2.clone()
        } else {
            self.player2.dices = None;
            self.player1.dices = Some(self.player1.roll());
            self.player1.clone()
        }
    }
}

#[derive(Debug, Clone)]
/// Represents a player with a dice
pub struct PlayerWithDice {
    /// The player
    pub player: Player,
    /// The dice
    pub rng: StdRng,
    /// The dices
    pub dices: Option<Dices>,
}

impl PlayerWithDice {
    /// Create a new player with a dice
    pub fn new(player: Player, seed: [u8; 32]) -> Self {
        let rng = StdRng::from_seed(seed);
        Self { player, rng, dices: None }
    }

    /// Roll the dice
    pub fn roll(&mut self) -> Dices {
        let between = Uniform::new_inclusive(1, 6);
        let values = (between.sample(&mut self.rng), between.sample(&mut self.rng));
        let consumed = if values.0 == values.1 {
            (false, false, false, false)
        } else {
            (false, false, true, true)
        };

        Dices {
            values,
            consumed,
        }
    }
}

/// Represents the two dices
///
/// Backgammon is always played with two dices.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Deserialize, Default)]
pub struct Dices {
    /// The two dice values
    pub values: (u8, u8),
    /// Boolean indicating whether the dices have been consumed already. We use a tuple
    /// of four booleans in case the dices are equal, in which case we have four dices
    /// to play.
    pub consumed: (bool, bool, bool, bool),
}

impl Dices {
    /// Roll the dices which generates two random numbers between 1 and 6, replicating a perfect
    /// dice. We use the operating system's random number generator.
    pub fn roll(self) -> Self {
        let between = Uniform::new_inclusive(1, 6);
        let mut rng = rand::thread_rng();

        let v = (between.sample(&mut rng), between.sample(&mut rng));

        // if both dices are equal, we have four dices to play
        if v.0 == v.1 {
            Dices {
                values: (v.0, v.1),
                consumed: (false, false, false, false),
            }
        } else {
            Dices {
                values: (v.0, v.1),
                consumed: (false, false, true, true),
            }
        }
    }
}

/// Trait to roll the dices
pub trait Roll {
    /// Roll the dices
    fn roll(&mut self) -> Result<&mut Self, Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roll() {
        let dices = Dices::default().roll();
        assert!(dices.values.0 >= 1 && dices.values.0 <= 6);
        assert!(dices.values.1 >= 1 && dices.values.1 <= 6);
    }

    #[test]
    fn test_roll_consumed() {
        let dices = Dices::default().roll();
        if dices.values.0 == dices.values.1 {
            assert_eq!(dices.consumed, (false, false, false, false));
        } else {
            assert_eq!(dices.consumed, (false, false, true, true));
        }
    }

    #[test]
    fn test_roll_consumed1() {
        for _i in 0..100 {
            let dices = Dices::default().roll();
            if dices.values.0 == dices.values.1 {
                assert_eq!(dices.consumed, (false, false, false, false));
            } else {
                assert_eq!(dices.consumed, (false, false, true, true));
            }
        }
    }
}
