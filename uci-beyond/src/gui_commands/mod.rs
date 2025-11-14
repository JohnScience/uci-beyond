//! The module for the types that represent [UCI] commands sent by the GUI, e.g. [`SetOptionCommand`] (for `setoption` command) and [`GoCommand`] (for `go` command).
//!
//! The module documentation is largely copied from <https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html>.
//!
//! # Note on [`std::fmt::Display`] implementation for commands
//!
//! Each command implements the [`std::fmt::Display`] trait to convert the command into a string that can be sent to the engine.
//! The resulting string does **NOT** include a newline character at the end.
//!
//! [UCI]: https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html

use std::fmt::Display;

mod go;
mod isready;
mod position;
mod quit;
mod setoption;
mod stop;
mod uci;
mod ucinewgame;

pub use go::GoCommand;
pub use isready::IsReadyCommand;
pub use position::PositionCommand;
pub use quit::QuitCommand;
pub use setoption::SetOptionCommand;
pub use stop::StopCommand;
pub use uci::UciCommand;
pub use ucinewgame::UciNewGameCommand;

use crate::util::AsyncReadable;

/// The trait that all GUI UCI commands implement.
pub trait UciCommandTrait: Display + Send {
    type Response: AsyncReadable + std::fmt::Debug;
}

// TODO: consider adding non-standard commands (e.g. for Stockfish)
