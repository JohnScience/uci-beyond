use std::fmt::Display;

use crate::gui_commands::UciCommandTrait;

/// This is used to synchronize the engine with the GUI.
// When the GUI has sent a command or multiple commands that can take some time to complete, this command can be used to wait for the engine to be ready again or to ping the engine to find out if it is still alive.
// e.g. this should be sent after setting the path to the tablebases as this can take some time.
// This command is also required once, before the engine is asked to do any searching, to wait for the engine to finish initializing.
// This command will always be answered with `readyok` and can be sent also when the engine is calculating in which case the engine will also immediately answer with `readyok` without stopping the search.
///
/// See in Stockfish UCI documentation: <https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html#isready>.
pub struct IsReadyCommand;

impl Display for IsReadyCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "isready")
    }
}

impl UciCommandTrait for IsReadyCommand {
    type Response = ();
}
