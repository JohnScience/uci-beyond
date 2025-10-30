use std::fmt::Display;

use crate::gui_commands::UciCommandTrait;

/// Quit the program as soon as possible.
///
/// See in Stockfish UCI documentation: <https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html#quit>.
pub struct QuitCommand;

impl Display for QuitCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "quit")
    }
}

impl UciCommandTrait for QuitCommand {
    type Response = ();
}
