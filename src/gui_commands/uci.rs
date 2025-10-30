use std::fmt::Display;

use crate::gui_commands::UciCommandTrait;

/// Tell the engine to use the UCI (universal chess interface).
/// This will be sent once, by a GUI, as a first command after the program boots to tell the engine to switch to UCI mode.
/// After receiving the `uci` command the engine will identify itself with the `id` command and send the option commands to tell the GUI which engine settings the engine supports.
/// After that, the engine will send `uciok` to acknowledge the UCI mode.
/// If no `uciok` is sent within a certain time period, the engine task will be killed by the GUI.
///
/// See in Stockfish UCI documentation: <https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html#uci>.
pub struct UciCommand;

impl Display for UciCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "uci")
    }
}

impl UciCommandTrait for UciCommand {
    // TODO: Define a proper response type
    type Response = ();
}
