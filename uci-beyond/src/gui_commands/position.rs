use std::fmt::Display;

use crate::{gui_commands::UciCommandTrait, model};

/// Set up the position described in `fenstring`.
/// If the game was played from the start position the string `startpos` must be sent.
///
/// See in Stockfish UCI documentation: <https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html#position>.
pub struct PositionCommand {
    pub startpos: model::Position,
    pub moves: Vec<model::MoveString>,
}

impl PositionCommand {
    /// Create a `PositionCommand` from a FEN string.
    pub fn from_fen(fen: model::FenString) -> Self {
        Self {
            startpos: model::Position::Fen(fen),
            moves: Vec::new(),
        }
    }
}

impl Display for PositionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "position {}", self.startpos)?;
        if !self.moves.is_empty() {
            write!(f, " moves")?;
            for m in &self.moves {
                write!(f, " {}", m.0)?;
            }
        }
        Ok(())
    }
}

impl UciCommandTrait for PositionCommand {
    type Response = ();
}
