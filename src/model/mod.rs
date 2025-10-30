//! The module for common types used in [UCI], such as [`MoveString`] (e.g. for `e2e4`) and [`FenString`] (e.g. for `rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1`).
//!
//! [UCI]: https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html

use std::fmt::Display;

mod check;
mod numa_policy;
mod uci_string;

pub use check::{Check, CheckParsingError};
pub use numa_policy::{NumaPolicy, NumaPolicyParsingError};
pub use uci_string::UciString;

/// [Forsyth-Edwards Notation (FEN)](https://www.chess.com/terms/fen-chess)
/// string representing a chess position.
pub struct FenString(pub String);

/// Either a starting position or a [`FenString`].
///
/// See [`gui_commands::PositionCommand`](crate::gui_commands::PositionCommand).
pub enum Position {
    StartPos,
    Fen(FenString),
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Position::StartPos => write!(f, "startpos"),
            Position::Fen(fen) => write!(f, "fen {}", fen.0),
        }
    }
}

/// A move in UCI long algebraic notation.
///
/// More about UCI long algebraic notation:
///
/// * <https://en.wikipedia.org/wiki/Algebraic_notation_(chess)#Long_algebraic_notation:~:text=A%20form%20of%20long%20algebraic,)%2C%20e7e8q%20(promotion)>
/// * <https://en.wikipedia.org/wiki/Universal_Chess_Interface#Design:~:text=long%20algebraic%20notation>
pub struct MoveString(pub String);

impl Display for MoveString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
