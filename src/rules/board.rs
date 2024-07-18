use crate::rules::Player;
use crate::Error;
use serde::{Deserialize, Serialize};

/// Represents the Backgammon board
///
/// A Backgammon board consists of 24 fields, each of which can hold 0 or more checkers. In
/// addition there is a bar to hold checkers that have been hit and an off area to hold checkers
/// that have been removed from the board.
///
/// ```
/// # fn foo() {}
/// //        +12-11-10--9--8--7-------6--5--4--3--2--1-+
/// //        | X           O    |   | O              X | +-------+
/// //        | X           O    |   | O              X | | OFF O |
/// //        | X           O    |   | O                | +-------+
/// //        | X                |   | O                |
/// //        | X                |   | O                |
/// //        |                  |BAR|                  |
/// //        | O                |   | X                |
/// //        | O                |   | X                |
/// //        | O           X    |   | X                | +-------+
/// //        | O           X    |   | X              O | | OFF X |
/// //        | O           X    |   | X              O | +-------+
/// //        +13-14-15-16-17-18------19-20-21-22-23-24-+
/// ```

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize, Default)]
pub struct Board {
    raw_board: (PlayerBoard, PlayerBoard),
}

/// Represents the Backgammon board for both players (to be used for graphical representation).
#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct BoardDisplay {
    /// The board represented as an array of 24 fields, each of which can hold 0 or more checkers.
    /// Positive amounts represent checkers of player 0, negative amounts represent checkers of
    /// player 1.
    pub board: [i8; 24],
    /// The bar for both players
    pub bar: (u8, u8),
    /// The off for both players
    pub off: (u8, u8),
}

impl Board {
    /// Create a new board
    pub fn new() -> Self {
        Board::default()
    }

    /// Get the board for both players. Use for graphical representation of the board.
    ///
    /// This method outputs a tuple with three values:
    ///
    /// 1. the board represented as an array of 24 fields, each of which can hold 0 or more
    /// checkers. Positive amounts represent checkers of player 0, negative amounts represent
    /// checkers of player 1.
    /// 2. the bar for both players
    /// 3. the off for both players
    pub fn get(&self) -> BoardDisplay {
        let mut board: [i8; 24] = [0; 24];

        for (i, val) in board.iter_mut().enumerate() {
            *val = self.raw_board.0.board[i] as i8 - self.raw_board.1.board[23 - i] as i8;
        }

        BoardDisplay {
            board,
            bar: self.get_bar(),
            off: self.get_off(),
        }
    }

    /// Get the bar for both players
    fn get_bar(&self) -> (u8, u8) {
        (self.raw_board.0.bar, self.raw_board.1.bar)
    }

    /// Get the off for both players
    fn get_off(&self) -> (u8, u8) {
        (self.raw_board.0.off, self.raw_board.1.off)
    }

    /// Set checkers for a player on a field
    ///
    /// This method adds the amount of checkers for a player on a field. The field is numbered from
    /// 0 to 23, starting from the last field of each player in the home board, the most far away
    /// field for each player (where there are 2 checkers to start with) is number 23.
    ///
    /// If the field is blocked for the player, an error is returned. If the field is not blocked,
    /// but there is already one checker from the other player on the field, that checker is hit and
    /// moved to the bar.
    pub fn set(&mut self, player: Player, field: usize, amount: i8) -> Result<(), Error> {
        if field > 23 {
            return Err(Error::FieldInvalid);
        }

        if self.blocked(player, field)? {
            return Err(Error::FieldBlocked);
        }
        let player_board = self.get_mut_raw_board_for_player(player)?;
        let new = player_board.board[field] as i8 + amount;
        if new < 0 {
            return Err(Error::MoveInvalid);
        }
        player_board.board[field] = new as u8;
        let opponent = self.get_mut_raw_board_for_opponent(player)?;
        opponent.bar += opponent.board[23 - field];
        opponent.board[23 - field] = 0;
        Ok(())
    }

    /// Check if a field is blocked for a player
    pub fn blocked(&self, player: Player, field: usize) -> Result<bool, Error> {
        if field > 23 {
            return Err(Error::FieldInvalid);
        }

        let opponent = self.get_raw_board_for_opponent(player)?;
        if opponent.board[23 - field] > 1 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Set checkers for a player on the bar. This method adds amount to the already existing
    /// checkers there.
    pub fn set_bar(&mut self, player: Player, amount: i8) -> Result<(), Error> {
        let raw_board = self.get_mut_raw_board_for_player(player)?;
        let new = raw_board.bar as i8 + amount;
        if new < 0 {
            return Err(Error::MoveInvalid);
        }
        raw_board.bar = new as u8;
        Ok(())
    }

    /// Set checkers for a player off the board. This method adds amount to the already existing
    /// checkers there.
    pub fn set_off(&mut self, player: Player, amount: u8) -> Result<(), Error> {
        self.get_mut_raw_board_for_player(player)?.off += amount;
        Ok(())
    }

    /// apply move from move checker
    pub fn apply_move(&mut self, move_checker: &MoveChecker) -> Result<(), Error> {
        match (&move_checker.from, &move_checker.to) {
            (BoardPosition::Bar, BoardPosition::Field(to)) => {
                self.set_bar(move_checker.player, -1)?;
                self.set(move_checker.player, *to, 1)?;
            }
            (BoardPosition::Field(from), BoardPosition::Field(to)) => {
                self.set(move_checker.player, *from, -1)?;
                self.set(move_checker.player, *to, 1)?;
            }
            (BoardPosition::Field(from), BoardPosition::Off) => {
                self.set(move_checker.player, *from, -1)?;
                self.set_off(move_checker.player, 1)?;
            }
            _ => return Err(Error::MoveInvalid),
        }
        Ok(())
    }

    /// check if game is finished
    pub fn is_finished(&self) -> bool {
        self.is_winner(Player::Player0) || self.is_winner(Player::Player1)
    }

    /// check if player is winner
    pub fn is_winner(&self, player: Player) -> bool {
        self.get_raw_board_for_player(player).expect("for player").off == 15
    }

    /// generate a move from dice roll for player
    pub fn generate_a_possible_moves(&self, player: Player, dice: usize) -> Result<Vec<MoveChecker>, Error> {
        let player_board = self.get_raw_board_for_player(player)?;
        if player_board.bar > 0 && self.blocked(player, 24 - dice)? {
            let move_checker = MoveChecker {
                player,
                from: BoardPosition::Bar,
                to: BoardPosition::Field(dice - 1),
            };
            if !self.blocked(player, dice - 1)? {
                return Ok(vec![move_checker]);
            } else {
                return Err(Error::MoveInvalid);
            }
        } else {
            let all_fields = player_board.board.iter().enumerate().filter(|(_, &x)| x > 0).collect::<Vec<(usize, &u8)>>();
            let all_moves = all_fields.into_iter().filter_map(|(i, _field)| {
                if let Some(new) = i.checked_sub(dice) {
                    let move_checker = MoveChecker {
                        player,
                        from: BoardPosition::Field(i),
                        to: BoardPosition::Field(new),
                    };
                    if !self.blocked(player, new).ok()? {
                        Some(move_checker)
                    } else {
                        None
                    }
                } else {
                    let move_checker = MoveChecker {
                        player,
                        from: BoardPosition::Field(i),
                        to: BoardPosition::Off,
                    };
                    Some(move_checker)
                }
            }).collect();
            return Ok(all_moves);
        }
    }

    fn get_raw_board_for_player(&self, player: Player) -> Result<&PlayerBoard, Error> {
        match player {
            Player::Player0 => Ok(&self.raw_board.0),
            Player::Player1 => Ok(&self.raw_board.1),
            Player::Nobody => Err(Error::PlayerInvalid),
        }
    }

    fn get_raw_board_for_opponent(&self, player: Player) -> Result<&PlayerBoard, Error> {
        match player {
            Player::Player0 => Ok(&self.raw_board.1),
            Player::Player1 => Ok(&self.raw_board.0),
            Player::Nobody => Err(Error::PlayerInvalid),
        }
    }

    fn get_mut_raw_board_for_player(&mut self, player: Player) -> Result<&mut PlayerBoard, Error> {
        match player {
            Player::Player0 => Ok(&mut self.raw_board.0),
            Player::Player1 => Ok(&mut self.raw_board.1),
            Player::Nobody => Err(Error::PlayerInvalid),
        }
    }

    fn get_mut_raw_board_for_opponent(&mut self, player: Player) -> Result<&mut PlayerBoard, Error> {
        match player {
            Player::Player0 => Ok(&mut self.raw_board.1),
            Player::Player1 => Ok(&mut self.raw_board.0),
            Player::Nobody => Err(Error::PlayerInvalid),
        }
    }

}

/// Represents the Backgammon board for one player
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PlayerBoard {
    board: [u8; 24],
    bar: u8,
    off: u8,
}

impl Default for PlayerBoard {
    fn default() -> Self {
        PlayerBoard {
            board: [
                0, 0, 0, 0, 0, 5, 0, 3, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2,
            ],
            bar: 0,
            off: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct MoveChecker {
    player: Player,
    from: BoardPosition,
    to: BoardPosition,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
enum BoardPosition {
    Bar,
    Off,
    Field(usize),
}

/// Trait to move checkers
pub trait Move {
    /// Move a checker
    fn move_checker(&mut self, player: Player, dice: u8, from: usize) -> Result<&mut Self, Error>
    where
        Self: Sized;

    /// Move a checker from bar
    fn move_checker_from_bar(&mut self, player: Player, dice: u8) -> Result<&mut Self, Error>
    where
        Self: Sized;

    /// Move permitted
    fn move_permitted(&mut self, player: Player, dice: u8) -> Result<&mut Self, Error>
    where
        Self: Sized;
}

// Unit Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_board() {
        assert_eq!(Board::new(), Board::default());
    }

    #[test]
    fn default_player_board() {
        assert_eq!(
            PlayerBoard::default(),
            PlayerBoard {
                board: [0, 0, 0, 0, 0, 5, 0, 3, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2,],
                bar: 0,
                off: 0
            }
        );
    }

    #[test]
    fn get_board() {
        let board = Board::new();
        assert_eq!(
            board.get(),
            BoardDisplay {
                board: [
                    -2, 0, 0, 0, 0, 5, 0, 3, 0, 0, 0, -5, 5, 0, 0, 0, -3, 0, -5, 0, 0, 0, 0, 2,
                ],
                bar: (0, 0),
                off: (0, 0)
            }
        );
    }

    #[test]
    fn get_bar() {
        let board = Board::new();
        assert_eq!(board.get_bar(), (0, 0));
    }

    #[test]
    fn get_off() {
        let board = Board::new();
        assert_eq!(board.get_off(), (0, 0));
    }

    #[test]
    fn set_player0() -> Result<(), Error> {
        let mut board = Board::new();
        board.set(Player::Player0, 1, 1)?;
        assert_eq!(board.get().board[1], 1);
        Ok(())
    }

    #[test]
    fn set_player1() -> Result<(), Error> {
        let mut board = Board::new();
        board.set(Player::Player1, 2, 1)?;
        assert_eq!(board.get().board[21], -1);
        Ok(())
    }

    #[test]
    fn set_player0_bar() -> Result<(), Error> {
        let mut board = Board::new();
        board.set_bar(Player::Player0, 1)?;
        assert_eq!(board.get().bar.0, 1);
        Ok(())
    }

    #[test]
    fn set_player1_bar() -> Result<(), Error> {
        let mut board = Board::new();
        board.set_bar(Player::Player1, 1)?;
        assert_eq!(board.get().bar.1, 1);
        Ok(())
    }

    #[test]
    fn set_player0_off() -> Result<(), Error> {
        let mut board = Board::new();
        board.set_off(Player::Player0, 1)?;
        assert_eq!(board.get().off.0, 1);
        Ok(())
    }

    #[test]
    fn set_player1_off() -> Result<(), Error> {
        let mut board = Board::new();
        board.set_off(Player::Player1, 1)?;
        assert_eq!(board.get().off.1, 1);
        Ok(())
    }

    #[test]
    fn set_player1_off1() -> Result<(), Error> {
        let mut board = Board::new();
        board.set_off(Player::Player1, 1)?;
        board.set_off(Player::Player1, 1)?;
        assert_eq!(board.get().off.1, 2);
        Ok(())
    }

    #[test]
    fn set_invalid_player() {
        let mut board = Board::new();
        assert!(board.set(Player::Nobody, 0, 1).is_err());
        assert!(board.set_bar(Player::Nobody, 1).is_err());
        assert!(board.set_off(Player::Nobody, 1).is_err());
    }

    #[test]
    fn blocked_player0() -> Result<(), Error> {
        let board = Board::new();
        assert!(board.blocked(Player::Player0, 0)?);
        Ok(())
    }

    #[test]
    fn blocked_player1() -> Result<(), Error> {
        let board = Board::new();
        assert!(board.blocked(Player::Player1, 0)?);
        Ok(())
    }

    #[test]
    fn blocked_player0_a() -> Result<(), Error> {
        let mut board = Board::new();
        board.set(Player::Player1, 1, 2)?;
        assert!(board.blocked(Player::Player0, 22)?);
        Ok(())
    }

    #[test]
    fn blocked_player1_a() -> Result<(), Error> {
        let mut board = Board::new();
        board.set(Player::Player0, 1, 2)?;
        assert!(board.blocked(Player::Player1, 22)?);
        Ok(())
    }

    #[test]
    fn blocked_invalid_player() {
        let board = Board::new();
        assert!(board.blocked(Player::Nobody, 0).is_err());
    }

    #[test]
    fn blocked_invalid_field() {
        let board = Board::new();
        assert!(board.blocked(Player::Player0, 24).is_err());
    }

    #[test]
    fn set_field_with_1_checker_player0_a() -> Result<(), Error> {
        let mut board = Board::new();
        board.set(Player::Player0, 1, 1)?;
        board.set(Player::Player1, 22, 1)?;
        assert_eq!(board.get().board[1], -1);
        assert_eq!(board.get().bar.0, 1);
        Ok(())
    }

    #[test]
    fn set_field_with_1_checker_player0_b() -> Result<(), Error> {
        let mut board = Board::new();
        board.set(Player::Player0, 1, 1)?;
        board.set_bar(Player::Player0, 5)?;
        board.set(Player::Player1, 22, 1)?;
        assert_eq!(board.get().board[1], -1);
        assert_eq!(board.get().bar.0, 6);
        Ok(())
    }

    #[test]
    fn set_field_with_1_checker_player1_a() -> Result<(), Error> {
        let mut board = Board::new();
        board.set(Player::Player1, 1, 1)?;
        board.set(Player::Player0, 22, 1)?;
        assert_eq!(board.get().board[22], 1);
        assert_eq!(board.get().bar.1, 1);
        Ok(())
    }

    #[test]
    fn set_field_with_1_checker_player1_b() -> Result<(), Error> {
        let mut board = Board::new();
        board.set(Player::Player1, 1, 1)?;
        board.set_bar(Player::Player1, 5)?;
        board.set(Player::Player0, 22, 1)?;
        assert_eq!(board.get().board[22], 1);
        assert_eq!(board.get().bar.1, 6);
        Ok(())
    }

    #[test]
    fn set_field_with_2_checkers_player0_a() -> Result<(), Error> {
        let mut board = Board::new();
        board.set(Player::Player0, 23, 2)?;
        assert_eq!(board.get().board[23], 4);
        Ok(())
    }

    #[test]
    fn set_field_with_2_checkers_player0_b() -> Result<(), Error> {
        let mut board = Board::new();
        board.set(Player::Player0, 23, -1)?;
        assert_eq!(board.get().board[23], 1);
        Ok(())
    }

    #[test]
    fn set_field_blocked() {
        let mut board = Board::new();
        assert!(board.set(Player::Player0, 0, 2).is_err());
    }

    #[test]
    fn set_wrong_field1() {
        let mut board = Board::new();
        assert!(board.set(Player::Player0, 50, 2).is_err());
    }

    #[test]
    fn set_wrong_amount0() {
        let mut board = Board::new();
        assert!(board.set(Player::Player0, 23, -3).is_err());
    }

    #[test]
    fn set_wrong_amount1() {
        let mut board = Board::new();
        assert!(board.set(Player::Player1, 23, -3).is_err());
    }

    #[test]
    fn generate_a_move() {
        let board = Board::new();
        let move_checker = board.generate_a_possible_moves(Player::Player0, 1).unwrap();
        assert_eq!(move_checker.len(), 3);
        assert_eq!(move_checker, vec![MoveChecker {
            player: Player::Player0,
            from: BoardPosition::Field(5),
            to: BoardPosition::Field(4),
        }, MoveChecker {
            player: Player::Player0,
            from: BoardPosition::Field(7),
            to: BoardPosition::Field(6),
        }, MoveChecker {
            player: Player::Player0,
            from: BoardPosition::Field(23),
            to: BoardPosition::Field(22),
        }]);
    }
}
