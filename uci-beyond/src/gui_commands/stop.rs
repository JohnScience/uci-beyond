use std::fmt::Display;

use crate::gui_commands::UciCommandTrait;

/// Stop calculating as soon as possible.
///
/// See in Stockfish UCI documentation: <https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html#stop>.
pub struct StopCommand;

impl Display for StopCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "stop")
    }
}

impl UciCommandTrait for StopCommand {
    type Response = ();
}
