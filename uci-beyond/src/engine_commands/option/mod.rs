use std::str::FromStr;

use crate::{
    command, options,
    util::{AsyncReadable, LineHandlerOutcome, StreamingLineReader, handle_next_line},
};

mod type_clause;
mod uci_option_block;

use async_trait::async_trait;
pub use type_clause::{TypeClause, TypeClauseParsingError};
pub use uci_option_block::{OptionBlockParsingError, UciOptionBlock};

/// <https://backscattering.de/chess/uci/#engine-option>
///
/// E.g.
///
/// ```text
/// option name Hash type spin default 1 min 1 max 128
/// ```
pub struct OptionCommand(pub options::UciOption);

#[derive(Debug)]
pub enum OptionCommandParsingError {
    /// The `name` token was expected. Encountered something else.
    NameTokenExpected(String),
    TypeClauseParsingError(TypeClauseParsingError),
    UnexpectedUciType {
        option_kind: options::UciOptionKind,
        found: options::UciOptionType,
    },
    KnownUciOptionDataParsingError(options::typed_uci_option_data::KnownUciOptionDataParsingError),
    UciOptionFromPartsError(options::UciOptionFromPartsError),
    UnexpectedTrailingTokens(String),
}

impl OptionCommand {
    fn parse_name_token(
        s: &str,
    ) -> Result<&str, command::parsing::Error<OptionCommandParsingError>> {
        debug_assert_eq!(s, s.trim_start());

        let Some(name_token) = s.split_whitespace().next() else {
            return Err(command::parsing::Error::UnexpectedEndOfTokens);
        };

        if name_token != "name" {
            return Err(command::parsing::Error::CustomError(
                OptionCommandParsingError::NameTokenExpected(name_token.to_string()),
            ));
        };

        Ok(s.trim_start_matches("name").trim_start())
    }

    fn parse_name_info(
        mut s: &str,
    ) -> Result<
        (options::UciOptionNameInfo, &str),
        command::parsing::Error<OptionCommandParsingError>,
    > {
        debug_assert_eq!(s, s.trim_start());
        let orig = s;

        let mut tokens = s.split_whitespace();

        loop {
            debug_assert_eq!(s, s.trim_start());

            let Some(token) = tokens.next() else {
                return Err(command::parsing::Error::UnexpectedEndOfTokens);
            };

            if token == "type" {
                s = s.trim_start();
                break;
            }

            s = s.trim_start_matches(token).trim_start();
        }

        debug_assert_eq!(s, s.trim_start());

        let option_kind = orig.trim_end_matches(s).trim_end();

        let s = s.trim_start_matches(option_kind).trim_start();

        let option_kind: options::UciOptionKind = match option_kind.parse() {
            Ok(kind) => kind,
            Err(e) => {
                return Ok((
                    options::UciOptionNameInfo::Custom {
                        name: option_kind.to_string(),
                    },
                    s,
                ));
            }
        };

        Ok((options::UciOptionNameInfo::Standard(option_kind), s))
    }

    fn validate_uci_type(
        name_info: &options::UciOptionNameInfo,
        uci_type: options::UciOptionType,
    ) -> Result<(), command::parsing::Error<OptionCommandParsingError>> {
        let option_kind = match name_info {
            options::UciOptionNameInfo::Standard(kind) => *kind,
            options::UciOptionNameInfo::Custom { .. } => {
                // Can't validate custom options
                return Ok(());
            }
        };

        let expected_uci_type = option_kind.r#type();

        if expected_uci_type != uci_type {
            return Err(command::parsing::Error::CustomError(
                OptionCommandParsingError::UnexpectedUciType {
                    option_kind,
                    found: uci_type,
                },
            ));
        }

        Ok(())
    }
}

impl command::Command for OptionCommand {
    type ParsingError = OptionCommandParsingError;

    const NAME: &'static str = "option";
}

impl std::fmt::Display for OptionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let uci_option = &self.0;

        let name = uci_option.name();
        let r#type = uci_option.r#type();

        match uci_option.r#type() {
            options::UciOptionType::Button => {
                write!(f, "option name {name} type {type}")
            }
            options::UciOptionType::Check
            | options::UciOptionType::String
            | options::UciOptionType::Spin
            | options::UciOptionType::Combo => {
                write!(f, "option name {name} type {type} {uci_option}")
            }
        }
    }
}

impl FromStr for OptionCommand {
    type Err = command::parsing::Error<OptionCommandParsingError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::command::Command as _;

        let s = OptionCommand::parse_cmd_name(s)?;

        // TODO: consider defining a "name clause"
        let s = OptionCommand::parse_name_token(s)?;
        let (name_info, s) = match OptionCommand::parse_name_info(s) {
            Ok(res) => res,
            Err(e) => return Err(e),
        };

        let (TypeClause { uci_type }, s) = TypeClause::parse_clause(s)?;

        OptionCommand::validate_uci_type(&name_info, uci_type)?;

        let (typed_data, s) = options::TypedUciOptionData::parse_for_type(uci_type, s)?;

        let uci_option = options::UciOption::from_parts(name_info, typed_data)?;

        if !s.is_empty() {
            return Err(command::parsing::Error::CustomError(
                OptionCommandParsingError::UnexpectedTrailingTokens(s.to_string()),
            ));
        }

        Ok(OptionCommand(uci_option))
    }
}

#[async_trait(?Send)]
impl AsyncReadable for OptionCommand {
    type Err = command::parsing::Error<OptionCommandParsingError>;

    async fn read_from<R>(reader: &mut R) -> Result<Option<Result<Self, Self::Err>>, R::Error>
    where
        R: StreamingLineReader,
    {
        let f = |line: &str| -> LineHandlerOutcome<OptionCommand, <OptionCommand as FromStr>::Err> {
            match line.parse::<OptionCommand>() {
                Ok(cmd) => LineHandlerOutcome::Read(cmd),
                Err(e) => LineHandlerOutcome::Error(e),
            }
        };

        match handle_next_line(reader, f).await? {
            Some(LineHandlerOutcome::Read(cmd)) => Ok(Some(Ok(cmd))),
            Some(LineHandlerOutcome::Error(e)) => Ok(Some(Err(e))),
            Some(LineHandlerOutcome::Peeked) => {
                return command::parsing::Error::UnexpectedPeekOutput.wrap();
            }
            None => Ok(None),
        }
    }
}

impl From<TypeClauseParsingError> for command::parsing::Error<OptionCommandParsingError> {
    fn from(e: TypeClauseParsingError) -> Self {
        command::parsing::Error::CustomError(OptionCommandParsingError::TypeClauseParsingError(e))
    }
}

impl From<command::parsing::Error<options::typed_uci_option_data::KnownUciOptionDataParsingError>>
    for command::parsing::Error<OptionCommandParsingError>
{
    fn from(
        value: command::parsing::Error<
            options::typed_uci_option_data::KnownUciOptionDataParsingError,
        >,
    ) -> Self {
        value.map_custom(OptionCommandParsingError::KnownUciOptionDataParsingError)
    }
}

impl From<options::UciOptionFromPartsError> for command::parsing::Error<OptionCommandParsingError> {
    fn from(value: options::UciOptionFromPartsError) -> Self {
        command::parsing::Error::CustomError(OptionCommandParsingError::UciOptionFromPartsError(
            value,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{model, options::UciOption};

    #[test]
    fn test_imitate_stockfish_output() {
        use std::fmt::Write as _;

        let mut buf = String::new();

        for cmd in [
            UciOption::DebugLogFile {
                default: model::UciString::default(),
            },
            UciOption::NumaPolicy {
                default: model::NumaPolicy::Auto,
            },
            UciOption::Threads(options::Spin {
                default: 1,
                min: 1,
                max: 1024,
            }),
            UciOption::Hash(options::Spin {
                default: 16,
                min: 1,
                max: 33_554_432,
            }),
            UciOption::ClearHash,
            UciOption::Ponder {
                default: model::Check(false),
            },
            UciOption::MultiPV(options::Spin {
                default: 1,
                min: 1,
                max: 256,
            }),
            UciOption::SkillLevel(options::Spin {
                default: 20,
                min: 0,
                max: 20,
            }),
            UciOption::MoveOverhead(options::Spin {
                default: 10,
                min: 0,
                max: 5000,
            }),
            UciOption::Nodestime(options::Spin {
                default: 0,
                min: 0,
                max: 10_000,
            }),
            UciOption::UCIChess960 {
                default: model::Check(false),
            },
            UciOption::UCILimitStrength {
                default: model::Check(false),
            },
            UciOption::UCIElo(options::Spin {
                default: 1320,
                min: 1320,
                max: 3190,
            }),
            UciOption::UCIShowWDL {
                default: model::Check(false),
            },
            UciOption::SyzygyPath {
                default: model::UciString::default(),
            },
            UciOption::SyzygyProbeDepth(options::Spin {
                default: 1,
                min: 1,
                max: 100,
            }),
            UciOption::Syzygy50MoveRule {
                default: model::Check(true),
            },
            UciOption::SyzygyProbeLimit(options::Spin {
                default: 7,
                min: 0,
                max: 7,
            }),
            UciOption::EvalFile {
                default: model::UciString("nn-1c0000000000.nnue".to_string()),
            },
            UciOption::EvalFileSmall {
                default: model::UciString("nn-37f18f62d772.nnue".to_string()),
            },
        ]
        .map(OptionCommand)
        {
            writeln!(&mut buf, "{cmd}").unwrap();
        }

        let expect = r#"option name Debug Log File type string default <empty>
option name NumaPolicy type string default auto
option name Threads type spin default 1 min 1 max 1024
option name Hash type spin default 16 min 1 max 33554432
option name Clear Hash type button
option name Ponder type check default false
option name MultiPV type spin default 1 min 1 max 256
option name Skill Level type spin default 20 min 0 max 20
option name Move Overhead type spin default 10 min 0 max 5000
option name nodestime type spin default 0 min 0 max 10000
option name UCI_Chess960 type check default false
option name UCI_LimitStrength type check default false
option name UCI_Elo type spin default 1320 min 1320 max 3190
option name UCI_ShowWDL type check default false
option name SyzygyPath type string default <empty>
option name SyzygyProbeDepth type spin default 1 min 1 max 100
option name Syzygy50MoveRule type check default true
option name SyzygyProbeLimit type spin default 7 min 0 max 7
option name EvalFile type string default nn-1c0000000000.nnue
option name EvalFileSmall type string default nn-37f18f62d772.nnue
"#;

        assert_eq!(buf, expect);
    }

    #[test]
    fn test_parse_option_commands() {
        let cmd_strs = [
            "option name Debug Log File type string default <empty>",
            "option name NumaPolicy type string default auto",
            "option name Threads type spin default 1 min 1 max 1024",
            "option name Hash type spin default 16 min 1 max 33554432",
            "option name Clear Hash type button",
            "option name Ponder type check default false",
            "option name MultiPV type spin default 1 min 1 max 256",
            "option name Skill Level type spin default 20 min 0 max 20",
            "option name Move Overhead type spin default 10 min 0 max 5000",
            "option name nodestime type spin default 0 min 0 max 10000",
            "option name UCI_Chess960 type check default false",
            "option name UCI_LimitStrength type check default false",
            "option name UCI_Elo type spin default 1320 min 1320 max 3190",
            "option name UCI_ShowWDL type check default false",
            "option name SyzygyPath type string default <empty>",
            "option name SyzygyProbeDepth type spin default 1 min 1 max 100",
            "option name Syzygy50MoveRule type check default true",
            "option name SyzygyProbeLimit type spin default 7 min 0 max 7",
            "option name EvalFile type string default nn-1c0000000000.nnue",
            "option name EvalFileSmall type string default nn-37f18f62d772.nnue",
        ];

        for cmd_str in cmd_strs {
            let option_cmd = match OptionCommand::from_str(cmd_str) {
                Ok(cmd) => cmd,
                Err(e) => panic!("Failed to parse OptionCommand from `{cmd_str}`: {e:?}"),
            };

            let uci_option: UciOption = option_cmd.0.clone();

            let reconstructed_cmd_str = option_cmd.to_string();

            assert_eq!(cmd_str, reconstructed_cmd_str.as_str());

            let reparsed_option_cmd =
                OptionCommand::from_str(&reconstructed_cmd_str).expect("Failed to reparse");

            let reparsed_uci_option: UciOption = reparsed_option_cmd.0;

            assert_eq!(uci_option, reparsed_uci_option);
        }
    }

    #[tokio::test]
    async fn test_parse_option_commands_async() {
        use tokio::io::BufReader;

        let input = b"option name Debug Log File type string default <empty>\n\
        option name NumaPolicy type string default auto\n\
        option name Threads type spin default 1 min 1 max 1024\n\
        option name Hash type spin default 16 min 1 max 33554432\n\
        option name Clear Hash type button\n\
        option name Ponder type check default false\n\
        option name MultiPV type spin default 1 min 1 max 256\n\
        option name Skill Level type spin default 20 min 0 max 20\n\
        option name Move Overhead type spin default 10 min 0 max 5000\n\
        option name nodestime type spin default 0 min 0 max 10000\n\
        option name UCI_Chess960 type check default false\n\
        option name UCI_LimitStrength type check default false\n\
        option name UCI_Elo type spin default 1320 min 1320 max 3190\n\
        option name UCI_ShowWDL type check default false\n\
        option name SyzygyPath type string default <empty>\n\
        option name SyzygyProbeDepth type spin default 1 min 1 max 100\n\
        option name Syzygy50MoveRule type check default true\n\
        option name SyzygyProbeLimit type spin default 7 min 0 max 7\n\
        option name EvalFile type string default nn-1c0000000000.nnue\n\
        option name EvalFileSmall type string default nn-37f18f62d772.nnue\n";

        let cursor = std::io::Cursor::new(input);
        let mut reader = BufReader::new(cursor);

        let expected = vec![
            UciOption::DebugLogFile {
                default: model::UciString::default(),
            },
            UciOption::NumaPolicy {
                default: model::NumaPolicy::Auto,
            },
            UciOption::Threads(options::Spin {
                default: 1,
                min: 1,
                max: 1024,
            }),
            UciOption::Hash(options::Spin {
                default: 16,
                min: 1,
                max: 33_554_432,
            }),
            UciOption::ClearHash,
            UciOption::Ponder {
                default: model::Check(false),
            },
            UciOption::MultiPV(options::Spin {
                default: 1,
                min: 1,
                max: 256,
            }),
            UciOption::SkillLevel(options::Spin {
                default: 20,
                min: 0,
                max: 20,
            }),
            UciOption::MoveOverhead(options::Spin {
                default: 10,
                min: 0,
                max: 5000,
            }),
            UciOption::Nodestime(options::Spin {
                default: 0,
                min: 0,
                max: 10_000,
            }),
            UciOption::UCIChess960 {
                default: model::Check(false),
            },
            UciOption::UCILimitStrength {
                default: model::Check(false),
            },
            UciOption::UCIElo(options::Spin {
                default: 1320,
                min: 1320,
                max: 3190,
            }),
            UciOption::UCIShowWDL {
                default: model::Check(false),
            },
            UciOption::SyzygyPath {
                default: model::UciString::default(),
            },
            UciOption::SyzygyProbeDepth(options::Spin {
                default: 1,
                min: 1,
                max: 100,
            }),
            UciOption::Syzygy50MoveRule {
                default: model::Check(true),
            },
            UciOption::SyzygyProbeLimit(options::Spin {
                default: 7,
                min: 0,
                max: 7,
            }),
            UciOption::EvalFile {
                default: model::UciString("nn-1c0000000000.nnue".to_string()),
            },
            UciOption::EvalFileSmall {
                default: model::UciString("nn-37f18f62d772.nnue".to_string()),
            },
        ];

        for expected_option in expected {
            let res = OptionCommand::read_from(&mut reader).await;

            match res {
                Ok(Some(Ok(option_cmd))) => {
                    assert_eq!(option_cmd.0, expected_option);
                }
                Ok(None) => break,
                Ok(Some(Err(e))) => panic!("Failed to parse OptionCommand: {e:?}"),
                Err(e) => panic!("I/O error: {e}"),
            }
        }

        let res = OptionCommand::read_from(&mut reader).await;
        assert!(matches!(res, Ok(None)));
    }
}
