//! The module for chess engine [UCI] options (see [`UciOption`]) and related types,
//!
//! [UCI]: https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html

use std::{collections::HashMap, fmt::Display};

use optional_struct::optional_struct;
use variants_data_struct::VariantsDataStruct;

use crate::model;

mod spin;
pub mod typed_uci_option_data;
mod uci_option_basic_info;
mod uci_option_kind;

pub use spin::Spin;
pub use typed_uci_option_data::{TypedUciOptionData, UciOptionType, UnknownUciOptionType};
pub use uci_option_basic_info::UciOptionBasicInfo;
pub use uci_option_kind::UciOptionKind;

#[derive(Debug)]
pub struct UciOptionDataTypeMismatchError {
    pub option_kind: UciOptionKind,
    pub found: UciOptionType,
}

#[derive(Debug)]
pub enum UciOptionFromPartsError {
    UciOptionDataTypeMismatchError(UciOptionDataTypeMismatchError),
    NumaPolicyParsingError(model::NumaPolicyParsingError),
}

/// The enumeration of known UCI options.

#[derive(
    VariantsDataStruct,
    // Kinded,
    PartialEq,
    Eq,
    Debug,
    Clone,
)]
#[variants_data_struct(
    name=UciOptionBlock,
    attrs(
        #[optional_struct(UciOptionBlockBuilder)]
        #[derive(Debug)]
    ),
    variants_tys_attrs(
        #[derive(Debug, PartialEq, Eq, Clone)]
    )
)]
// #[kinded(kind = UciOptionKind, opt_outs=[from_str_impl], derive(EnumIter))]
pub enum UciOption {
    /// The number of CPU threads used for searching a position. For best performance, set this equal to the number of CPU cores available.
    #[variants_data_struct_field(field_ty_override = Spin)]
    Threads(Spin),
    /// The size of the hash table in MB. It is recommended to set Hash after setting Threads.
    #[variants_data_struct_field(field_ty_override = Spin)]
    Hash(Spin),
    /// Output the N best lines (principal variations, PVs) when searching. Leave at 1 for the best performance.
    #[variants_data_struct_field(field_ty_override = Spin)]
    MultiPV(Spin),
    /// Binds threads to a specific [NUMA] node to enhance performance on multi-CPU or multi-[NUMA] domain systems.
    ///
    /// [NUMA]: https://www.chessprogramming.org/NUMA
    #[variants_data_struct_field(field_ty_override = model::NumaPolicy)]
    NumaPolicy { default: model::NumaPolicy },
    /// Clear the hash table.
    ClearHash,
    /// Let Stockfish ponder its next move while the opponent is thinking.
    #[variants_data_struct_field(field_ty_override = model::Check)]
    Ponder { default: model::Check },
    /// The name of the file of the NNUE evaluation parameters. Depending on the GUI the filename might have to include the full path to the folder/directory that contains the file. Other locations, such as the directory that contains the binary and the working directory, are also searched.
    #[variants_data_struct_field(field_ty_override = model::UciString)]
    EvalFile { default: model::UciString },
    /// For Stockfish, same as [`UciOption::EvalFile`].
    #[variants_data_struct_field(field_ty_override = model::UciString)]
    EvalFileSmall { default: model::UciString },
    /// An option handled by your GUI. If true, Stockfish will play Chess960.
    #[variants_data_struct_field(field_ty_override = model::Check)]
    UCIChess960 { default: model::Check },
    /// If enabled, show approximate WDL statistics as part of the engine output. These WDL numbers model expected game outcomes for a given evaluation and game ply for engine self-play at fishtest LTC conditions (60+0.6s per game).
    #[variants_data_struct_field(field_ty_override = model::Check)]
    UCIShowWDL { default: model::Check },
    /// Enable weaker play aiming for an Elo rating as set by `UCI_Elo`. This option overrides `Skill Level`.
    #[variants_data_struct_field(field_ty_override = model::Check)]
    UCILimitStrength { default: model::Check },
    /// If UCI_LimitStrength is enabled, it aims for an engine strength of the given Elo.
    /// This Elo rating has been calibrated at a time control of 120s+1s and anchored to CCRL 40/4.
    /// It takes precedence over Skill Level if both are set.
    /// See also [How do Skill Level and UCI_Elo work](https://official-stockfish.github.io/docs/stockfish-wiki/Stockfish-FAQ.html#how-do-skill-level-and-uci-elo-work).
    #[variants_data_struct_field(field_ty_override = Spin)]
    UCIElo(Spin),
    /// Lower the `Skill Level` in order to make Stockfish play weaker (see also `UCI_LimitStrength`).
    /// Internally, `MultiPV` is enabled, and with a certain probability depending on the `Skill Level`, a weaker move will be played.
    #[variants_data_struct_field(field_ty_override = Spin)]
    SkillLevel(Spin),
    /// Path to the folders/directories storing the Syzygy tablebase files. Multiple directories are to be separated by `;` on Windows and by :` on Unix-based operating systems. Do not use spaces around the `;` or `:`.
    ///
    /// Example:
    ///
    /// ```text
    /// C:\tablebases\wdl345;C:\tablebases\wdl6;D:\tablebases\dtz345;D:\tablebases\dtz6
    /// ```
    ///
    /// It is recommended to store .rtbw files on an SSD. There is no loss in storing the .rtbz files on a regular HDD. It is recommended to verify all md5 checksums of the downloaded tablebase files (`md5sum -c checksum.md5`) as corruption will lead to engine crashes.
    #[variants_data_struct_field(field_ty_override = model::UciString)]
    SyzygyPath {
        /// None means the string literal is `<empty>`
        default: model::UciString,
    },
    /// Minimum remaining search depth for which a position is probed. Set this option to a higher value to probe less aggressively if you experience too much slowdown (in terms of nps) due to tablebase probing.
    #[variants_data_struct_field(field_ty_override = Spin)]
    SyzygyProbeDepth(Spin),
    /// Disable to let fifty-move rule draws detected by Syzygy tablebase probes count as wins or losses. This is useful for ICCF correspondence games.
    #[variants_data_struct_field(field_ty_override = model::Check)]
    Syzygy50MoveRule { default: model::Check },
    /// Limit Syzygy tablebase probing to positions with at most this many pieces left (including kings and pawns).
    #[variants_data_struct_field(field_ty_override = Spin)]
    SyzygyProbeLimit(Spin),
    /// Assume a time delay of x ms due to network and GUI overheads. Specifying a value larger than the default is needed to avoid time losses or near instantaneous moves, in particular for time controls without increment (e.g. sudden death). The default is suitable for engine-engine matches played locally on dedicated hardware, while it needs to be increased on a loaded system, when playing over a network, or when using certain GUIs such as Arena or ChessGUI.
    #[variants_data_struct_field(field_ty_override = Spin)]
    MoveOverhead(Spin),
    /// Tells the engine to use nodes searched instead of wall time to account for elapsed time. Useful for engine testing. When this option is set, the engine is only limited by the total amount of nodes searched per game; this limit is calculated once per game. The initial time control values in milliseconds (time time and increment per move inc) are used as input values to calculate the total number of nodes per game (totalnodes). The increment per move inc is used as if it was just one move per game. The formula is totalnodes = (time + inc * 1) * nodestime. Suppose you specified nodestime = 600, and the time control per game is 300 seconds plus 3 seconds increment per move ("300+3s"), or 300000 milliseconds plus 3000 milliseconds increment per move. In that case, the maximum total number of nodes searched per game by the engine is totalnodes = (300000 + 3000 * 1) * 600 = 181800000 (one hundred eighty-one million, eight hundred thousand) nodes, regardless of how much wall time it will actually take.
    #[variants_data_struct_field(field_ty_override = Spin)]
    Nodestime(Spin),
    /// Write all communication to and from the engine into a text file.
    #[variants_data_struct_field(field_ty_override = model::UciString)]
    DebugLogFile { default: model::UciString },
    #[variants_data_struct_field(
        field_ty_override = HashMap<String, TypedUciOptionData>,
        field_attrs(#[optional_skip_wrap])
    )]
    Custom {
        name: String,
        typed_data: TypedUciOptionData,
    },
}

pub enum UciOptionNameInfo {
    Standard(UciOptionKind),
    Custom { name: String },
}

impl UciOption {
    pub fn basic_info(&self) -> UciOptionBasicInfo<&str> {
        let kind = match self {
            UciOption::Custom { name, typed_data } => {
                return UciOptionBasicInfo::Custom {
                    name: &name,
                    r#type: typed_data.r#type(),
                };
            }
            UciOption::Threads { .. } => UciOptionKind::Threads,
            UciOption::Hash { .. } => UciOptionKind::Hash,
            UciOption::MultiPV { .. } => UciOptionKind::MultiPV,
            UciOption::NumaPolicy { .. } => UciOptionKind::NumaPolicy,
            UciOption::ClearHash => UciOptionKind::ClearHash,
            UciOption::Ponder { .. } => UciOptionKind::Ponder,
            UciOption::EvalFile { .. } => UciOptionKind::EvalFile,
            UciOption::EvalFileSmall { .. } => UciOptionKind::EvalFileSmall,
            UciOption::UCIChess960 { .. } => UciOptionKind::UCIChess960,
            UciOption::UCIShowWDL { .. } => UciOptionKind::UCIShowWDL,
            UciOption::UCILimitStrength { .. } => UciOptionKind::UCILimitStrength,
            UciOption::UCIElo { .. } => UciOptionKind::UCIElo,
            UciOption::SkillLevel { .. } => UciOptionKind::SkillLevel,
            UciOption::SyzygyPath { .. } => UciOptionKind::SyzygyPath,
            UciOption::SyzygyProbeDepth { .. } => UciOptionKind::SyzygyProbeDepth,
            UciOption::Syzygy50MoveRule { .. } => UciOptionKind::Syzygy50MoveRule,
            UciOption::SyzygyProbeLimit { .. } => UciOptionKind::SyzygyProbeLimit,
            UciOption::MoveOverhead { .. } => UciOptionKind::MoveOverhead,
            UciOption::Nodestime { .. } => UciOptionKind::Nodestime,
            UciOption::DebugLogFile { .. } => UciOptionKind::DebugLogFile,
        };
        UciOptionBasicInfo::Standard(kind)
    }

    pub fn name(&self) -> &str {
        self.basic_info().name()
    }

    pub fn r#type(&self) -> UciOptionType {
        self.basic_info().r#type()
    }

    pub fn from_parts(
        name_info: UciOptionNameInfo,
        typed_data: TypedUciOptionData,
    ) -> Result<Self, UciOptionFromPartsError> {
        let kind = match name_info {
            UciOptionNameInfo::Custom { name } => {
                return Ok(UciOption::Custom { name, typed_data });
            }
            UciOptionNameInfo::Standard(kind) => kind,
        };
        if kind.r#type() != typed_data.r#type() {
            return Err(UciOptionFromPartsError::UciOptionDataTypeMismatchError(
                UciOptionDataTypeMismatchError {
                    option_kind: kind,
                    found: typed_data.r#type(),
                },
            ));
        };

        match kind {
            UciOptionKind::Threads => {
                if let TypedUciOptionData::Spin(spin) = typed_data {
                    Ok(UciOption::Threads(spin))
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::Hash => {
                if let TypedUciOptionData::Spin(spin) = typed_data {
                    Ok(UciOption::Hash(spin))
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::MultiPV => {
                if let TypedUciOptionData::Spin(spin) = typed_data {
                    Ok(UciOption::MultiPV(spin))
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::NumaPolicy => {
                if let TypedUciOptionData::String(uci_string) = typed_data {
                    let numa_policy = model::NumaPolicy::try_from(uci_string)
                        .map_err(UciOptionFromPartsError::NumaPolicyParsingError)?;
                    Ok(UciOption::NumaPolicy {
                        default: numa_policy,
                    })
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::ClearHash => Ok(UciOption::ClearHash),
            UciOptionKind::Ponder => {
                if let TypedUciOptionData::Check(check) = typed_data {
                    Ok(UciOption::Ponder { default: check })
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::EvalFile => {
                if let TypedUciOptionData::String(uci_string) = typed_data {
                    Ok(UciOption::EvalFile {
                        default: uci_string,
                    })
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::EvalFileSmall => {
                if let TypedUciOptionData::String(uci_string) = typed_data {
                    Ok(UciOption::EvalFileSmall {
                        default: uci_string,
                    })
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::UCIChess960 => {
                if let TypedUciOptionData::Check(check) = typed_data {
                    Ok(UciOption::UCIChess960 { default: check })
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::UCIShowWDL => {
                if let TypedUciOptionData::Check(check) = typed_data {
                    Ok(UciOption::UCIShowWDL { default: check })
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::UCILimitStrength => {
                if let TypedUciOptionData::Check(check) = typed_data {
                    Ok(UciOption::UCILimitStrength { default: check })
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::UCIElo => {
                if let TypedUciOptionData::Spin(spin) = typed_data {
                    Ok(UciOption::UCIElo(spin))
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::SkillLevel => {
                if let TypedUciOptionData::Spin(spin) = typed_data {
                    Ok(UciOption::SkillLevel(spin))
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::SyzygyPath => {
                if let TypedUciOptionData::String(uci_string) = typed_data {
                    Ok(UciOption::SyzygyPath {
                        default: uci_string,
                    })
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::SyzygyProbeDepth => {
                if let TypedUciOptionData::Spin(spin) = typed_data {
                    Ok(UciOption::SyzygyProbeDepth(spin))
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::Syzygy50MoveRule => {
                if let TypedUciOptionData::Check(check) = typed_data {
                    Ok(UciOption::Syzygy50MoveRule { default: check })
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::SyzygyProbeLimit => {
                if let TypedUciOptionData::Spin(spin) = typed_data {
                    Ok(UciOption::SyzygyProbeLimit(spin))
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::MoveOverhead => {
                if let TypedUciOptionData::Spin(spin) = typed_data {
                    Ok(UciOption::MoveOverhead(spin))
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::Nodestime => {
                if let TypedUciOptionData::Spin(spin) = typed_data {
                    Ok(UciOption::Nodestime(spin))
                } else {
                    unreachable!()
                }
            }
            UciOptionKind::DebugLogFile => {
                if let TypedUciOptionData::String(uci_string) = typed_data {
                    Ok(UciOption::DebugLogFile {
                        default: uci_string,
                    })
                } else {
                    unreachable!()
                }
            }
        }
    }
}

impl Display for UciOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Threads(spin) => write!(f, "{spin}"),
            Self::Hash(spin) => write!(f, "{spin}"),
            Self::MultiPV(spin) => write!(f, "{spin}"),
            Self::NumaPolicy { default } => write!(f, "default {default}"),
            Self::ClearHash => Ok(()),
            Self::Ponder { default } => write!(f, "default {default}"),
            Self::EvalFile { default } => write!(f, "default {default}"),
            Self::EvalFileSmall { default } => write!(f, "default {default}"),
            Self::UCIChess960 { default } => write!(f, "default {default}"),
            Self::UCIShowWDL { default } => write!(f, "default {default}"),
            Self::UCILimitStrength { default } => write!(f, "default {default}"),
            Self::UCIElo(spin) => write!(f, "{spin}"),
            Self::SkillLevel(spin) => write!(f, "{spin}"),
            Self::SyzygyPath { default } => write!(f, "default {default}"),
            Self::SyzygyProbeDepth(spin) => write!(f, "{spin}"),
            Self::Syzygy50MoveRule { default } => write!(f, "default {default}"),
            Self::SyzygyProbeLimit(spin) => write!(f, "{spin}"),
            Self::MoveOverhead(spin) => write!(f, "{spin}"),
            Self::Nodestime(spin) => write!(f, "{spin}"),
            Self::DebugLogFile { default } => write!(f, "default {default}"),
            Self::Custom {
                name: _,
                typed_data,
            } => {
                write!(f, "{}", typed_data)
            }
        }
    }
}
