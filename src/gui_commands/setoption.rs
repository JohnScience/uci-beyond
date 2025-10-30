use std::fmt::Display;

use crate::{gui_commands::UciCommandTrait, model};

/// This is sent to the engine when the user wants to change the internal parameters of the engine. For the button type no value is needed.
/// One string will be sent for each parameter and this will only be sent when the engine is waiting.
///
/// See in Stockfish UCI documentation: <https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html#setoption>.
///
/// Also see [options::UciOption](crate::options::UciOption).
pub enum SetOptionCommand {
    Threads { value: u32 },
    Hash { value: u32 },
    MultiPV { value: u32 },
    NumaPolicy { value: model::NumaPolicy },
    ClearHash,
    Ponder { value: bool },
    EvalFile { value: String },
    EvalFileSmall { value: String },
    UCIChess960 { value: bool },
    UCIShowWDL { value: bool },
    UCILimitStrength { value: bool },
    UCIElo { value: u32 },
    SkillLevel { value: u32 },
    SyzygyPath { value: Option<String> },
    SyzygyProbeDepth { value: u32 },
    Syzygy50MoveRule { value: bool },
    SyzygyProbeLimit { value: u32 },
    MoveOverhead { value: u32 },
    Nodestime { value: u32 },
    DebugLogFile { value: String },
}

impl Display for SetOptionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "setoption name ")?;
        match self {
            SetOptionCommand::Threads { value } => {
                write!(f, "Threads value {value}")
            }
            SetOptionCommand::Hash { value } => write!(f, "Hash value {value}"),
            SetOptionCommand::MultiPV { value } => {
                write!(f, "MultiPV value {value}")
            }
            SetOptionCommand::NumaPolicy { value } => {
                write!(f, "NumaPolicy value {value}")
            }
            SetOptionCommand::ClearHash => write!(f, "Clear Hash"),
            SetOptionCommand::Ponder { value } => {
                write!(f, "Ponder value {value}")
            }
            SetOptionCommand::EvalFile { value } => {
                write!(f, "EvalFile value {value}")
            }
            SetOptionCommand::EvalFileSmall { value } => {
                write!(f, "EvalFileSmall value {value}")
            }
            SetOptionCommand::UCIChess960 { value } => {
                write!(f, "UCI_Chess960 value {value}")
            }
            SetOptionCommand::UCIShowWDL { value } => {
                write!(f, "UCI_ShowWDL value {value}")
            }
            SetOptionCommand::UCILimitStrength { value } => {
                write!(f, "UCI_LimitStrength value {value}")
            }
            SetOptionCommand::UCIElo { value } => {
                write!(f, "UCI_Elo value {value}")
            }
            SetOptionCommand::SkillLevel { value } => {
                write!(f, "Skill Level value {value}")
            }
            SetOptionCommand::SyzygyPath { value } => {
                let val_str = match value {
                    Some(v) => v.as_str(),
                    None => "<empty>",
                };
                write!(f, "SyzygyPath value {val_str}")
            }
            SetOptionCommand::SyzygyProbeDepth { value } => {
                write!(f, "SyzygyProbeDepth value {value}")
            }
            SetOptionCommand::Syzygy50MoveRule { value } => {
                write!(f, "Syzygy50MoveRule value {value}")
            }
            SetOptionCommand::SyzygyProbeLimit { value } => {
                write!(f, "SyzygyProbeLimit value {value}")
            }
            SetOptionCommand::MoveOverhead { value } => {
                write!(f, "MoveOverhead value {value}")
            }
            SetOptionCommand::Nodestime { value } => {
                write!(f, "nodestime value {value}")
            }
            SetOptionCommand::DebugLogFile { value } => {
                write!(f, "DebugLogFile value {value}")
            }
        }
    }
}

impl UciCommandTrait for SetOptionCommand {
    // TODO: Define a proper response type
    type Response = ();
}
