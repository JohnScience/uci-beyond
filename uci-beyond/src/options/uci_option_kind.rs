use strum::EnumIter;

use crate::options::UciOptionType;

#[derive(Debug)]
pub struct UnknownUciOptionKind(pub String);

/// Represents the standard UCI option kinds.
///
/// Any UCI option has a respective [`UciOptionType`].
///
/// Some engines may also define custom options that are not part of this enum.
///
/// To handle that, there is [`UciOption::Custom`](crate::options::UciOption::Custom).
#[derive(EnumIter, Debug, PartialEq, Eq, Clone, Copy)]
pub enum UciOptionKind {
    Threads,
    Hash,
    MultiPV,
    NumaPolicy,
    ClearHash,
    Ponder,
    EvalFile,
    EvalFileSmall,
    UCIChess960,
    UCIShowWDL,
    UCILimitStrength,
    UCIElo,
    SkillLevel,
    SyzygyPath,
    SyzygyProbeDepth,
    Syzygy50MoveRule,
    SyzygyProbeLimit,
    MoveOverhead,
    Nodestime,
    DebugLogFile,
}

impl UciOptionKind {
    pub fn name(self) -> &'static str {
        match self {
            UciOptionKind::Threads => "Threads",
            UciOptionKind::Hash => "Hash",
            UciOptionKind::MultiPV => "MultiPV",
            UciOptionKind::NumaPolicy => "NumaPolicy",
            UciOptionKind::ClearHash => "Clear Hash",
            UciOptionKind::Ponder => "Ponder",
            UciOptionKind::EvalFile => "EvalFile",
            UciOptionKind::EvalFileSmall => "EvalFileSmall",
            UciOptionKind::UCIChess960 => "UCI_Chess960",
            UciOptionKind::UCIShowWDL => "UCI_ShowWDL",
            UciOptionKind::UCILimitStrength => "UCI_LimitStrength",
            UciOptionKind::UCIElo => "UCI_Elo",
            UciOptionKind::SkillLevel => "Skill Level",
            UciOptionKind::SyzygyPath => "SyzygyPath",
            UciOptionKind::SyzygyProbeDepth => "SyzygyProbeDepth",
            UciOptionKind::Syzygy50MoveRule => "Syzygy50MoveRule",
            UciOptionKind::SyzygyProbeLimit => "SyzygyProbeLimit",
            UciOptionKind::MoveOverhead => "Move Overhead",
            UciOptionKind::Nodestime => "nodestime",
            UciOptionKind::DebugLogFile => "Debug Log File",
        }
    }

    pub fn r#type(self) -> UciOptionType {
        match self {
            UciOptionKind::Threads => UciOptionType::Spin,
            UciOptionKind::Hash => UciOptionType::Spin,
            UciOptionKind::MultiPV => UciOptionType::Spin,
            UciOptionKind::NumaPolicy => UciOptionType::String,
            UciOptionKind::ClearHash => UciOptionType::Button,
            UciOptionKind::Ponder => UciOptionType::Check,
            UciOptionKind::EvalFile => UciOptionType::String,
            UciOptionKind::EvalFileSmall => UciOptionType::String,
            UciOptionKind::UCIChess960 => UciOptionType::Check,
            UciOptionKind::UCIShowWDL => UciOptionType::Check,
            UciOptionKind::UCILimitStrength => UciOptionType::Check,
            UciOptionKind::UCIElo => UciOptionType::Spin,
            UciOptionKind::SkillLevel => UciOptionType::Spin,
            UciOptionKind::SyzygyPath => UciOptionType::String,
            UciOptionKind::SyzygyProbeDepth => UciOptionType::Spin,
            UciOptionKind::Syzygy50MoveRule => UciOptionType::Check,
            UciOptionKind::SyzygyProbeLimit => UciOptionType::Spin,
            UciOptionKind::MoveOverhead => UciOptionType::Spin,
            UciOptionKind::Nodestime => UciOptionType::Spin,
            UciOptionKind::DebugLogFile => UciOptionType::String,
        }
    }
}

impl std::str::FromStr for UciOptionKind {
    type Err = UnknownUciOptionKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use strum::IntoEnumIterator as _;

        for kind in UciOptionKind::iter() {
            if kind.name() == s {
                return Ok(kind);
            }
        }

        Err(UnknownUciOptionKind(s.to_string()))
    }
}
