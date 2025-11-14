use std::fmt::Display;

use crate::gui_commands::UciCommandTrait;

/// This is sent to the engine when the next search (started with `position` and `go`) will be from a different game. This can be a new game the engine should play or a new game it should analyze but also the next position from a test suite with positions only.
/// If the GUI hasn't sent a `ucinewgame` before the first `position` command, the engine won't expect any further `ucinewgame` commands as the GUI is probably not supporting the `ucinewgame` command.
/// So the engine will not rely on this command even though all new GUIs should support it.
/// As the engine's reaction to `ucinewgame` can take some time the GUI should always send `isready` after `ucinewgame` to wait for the engine to finish its operation. The engine will respond with `readyok`.
///
/// See in Stockfish UCI documentation: <https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html#ucinewgame>.
pub struct UciNewGameCommand;

impl Display for UciNewGameCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ucinewgame")
    }
}

impl UciCommandTrait for UciNewGameCommand {
    type Response = ();
}
