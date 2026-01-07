use async_trait::async_trait;

use crate::{
    command,
    engine_commands::{self, OptionCommand},
    options::UciOption,
    util::{AsyncReadable, LineHandlerOutcome, StreamingLineReader, handle_next_line},
};

// UciOptionBlock is defined there because the UciOption enum is in the options module
pub use crate::options::{UciOptionBlock, UciOptionBlockBuilder};

#[derive(Debug)]
pub enum OptionBlockParsingError {
    CommandErrorParsingError(engine_commands::OptionCommandParsingError),
    EmptyLineWasNotFoundUntilEndOfStream,
    RepeatedOption,
}

impl OptionBlockParsingError {
    fn wrap<RR>(
        self,
    ) -> Result<Option<Result<UciOptionBlockBuilder, command::parsing::Error<Self>>>, RR> {
        command::parsing::Error::from(self).wrap()
    }
}

#[async_trait(?Send)]
impl AsyncReadable for UciOptionBlockBuilder {
    type Err = command::parsing::Error<OptionBlockParsingError>;

    async fn read_from<R>(reader: &mut R) -> Result<Option<Result<Self, Self::Err>>, R::Error>
    where
        R: StreamingLineReader,
    {
        let mut i = 0;
        let mut b = UciOptionBlockBuilder::default();

        loop {
            let opt = handle_next_line(reader, |line: &str| {
                if !line.trim_start().starts_with("option") {
                    return LineHandlerOutcome::Peeked;
                }
                match line.parse::<OptionCommand>() {
                    Ok(cmd) => LineHandlerOutcome::Read(cmd),
                    Err(e) => LineHandlerOutcome::Error(e),
                }
            })
            .await?;

            let Some(outcome) = opt else {
                return OptionBlockParsingError::EmptyLineWasNotFoundUntilEndOfStream.wrap();
            };

            let cmd: OptionCommand = match outcome {
                LineHandlerOutcome::Peeked => {
                    if i == 0 {
                        // No OptionCommands were read; return None
                        return Ok(None);
                    } else {
                        // End of OptionBlock - return the builder with whatever options were found
                        return Ok(Some(Ok(b)));
                    }
                }
                LineHandlerOutcome::Read(cmd) => cmd,
                LineHandlerOutcome::Error(e) => {
                    return e.map_custom(OptionBlockParsingError::from).wrap();
                }
            };

            let uci_option: UciOption = cmd.0;

            match uci_option {
                UciOption::ClearHash => {
                    if b.clear_hash.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.clear_hash = Some(());
                }
                UciOption::DebugLogFile { default } => {
                    if b.debug_log_file.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.debug_log_file = Some(default);
                }
                UciOption::EvalFile { default } => {
                    if b.eval_file.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.eval_file = Some(default);
                }
                UciOption::EvalFileSmall { default } => {
                    if b.eval_file_small.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.eval_file_small = Some(default);
                }
                UciOption::Hash(spin) => {
                    if b.hash.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.hash = Some(spin);
                }
                UciOption::MoveOverhead(spin) => {
                    if b.move_overhead.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.move_overhead = Some(spin);
                }
                UciOption::MultiPV(spin) => {
                    if b.multi_pv.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.multi_pv = Some(spin);
                }
                UciOption::Nodestime(spin) => {
                    if b.nodestime.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.nodestime = Some(spin);
                }
                UciOption::NumaPolicy { default } => {
                    if b.numa_policy.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.numa_policy = Some(default);
                }
                UciOption::Ponder { default } => {
                    if b.ponder.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.ponder = Some(default);
                }
                UciOption::SkillLevel(spin) => {
                    if b.skill_level.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.skill_level = Some(spin);
                }
                UciOption::Syzygy50MoveRule { default } => {
                    if b.syzygy_50_move_rule.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.syzygy_50_move_rule = Some(default);
                }
                UciOption::SyzygyPath { default } => {
                    if b.syzygy_path.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.syzygy_path = Some(default);
                }
                UciOption::SyzygyProbeDepth(spin) => {
                    if b.syzygy_probe_depth.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.syzygy_probe_depth = Some(spin);
                }
                UciOption::SyzygyProbeLimit(spin) => {
                    if b.syzygy_probe_limit.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.syzygy_probe_limit = Some(spin);
                }
                UciOption::Threads(spin) => {
                    if b.threads.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.threads = Some(spin);
                }
                UciOption::UCIChess960 { default } => {
                    if b.uci_chess_960.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.uci_chess_960 = Some(default);
                }
                UciOption::UCIElo(spin) => {
                    if b.uci_elo.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.uci_elo = Some(spin);
                }
                UciOption::UCILimitStrength { default } => {
                    if b.uci_limit_strength.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.uci_limit_strength = Some(default);
                }
                UciOption::UCIShowWDL { default } => {
                    if b.uci_show_wdl.is_some() {
                        return OptionBlockParsingError::RepeatedOption.wrap();
                    };
                    b.uci_show_wdl = Some(default);
                }
                UciOption::Custom { name, typed_data } => {
                    use std::collections::hash_map::Entry::{Occupied, Vacant};

                    match b.custom.entry(name) {
                        Occupied(_) => {
                            return OptionBlockParsingError::RepeatedOption.wrap();
                        }
                        Vacant(e) => {
                            e.insert(typed_data);
                        }
                    }
                }
            };

            i += 1;
        }
    }
}

impl From<engine_commands::OptionCommandParsingError> for OptionBlockParsingError {
    fn from(err: engine_commands::OptionCommandParsingError) -> Self {
        OptionBlockParsingError::CommandErrorParsingError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{model, options};

    #[tokio::test]
    async fn test_parse_uci_option_block() {
        let input = "option name Debug Log File type string default <empty>\n\
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
           option name EvalFileSmall type string default nn-37f18f62d772.nnue\n\
           \n\
           \n";

        let mut reader = tokio::io::BufReader::new(input.as_bytes());
        let uci_option_block_builder = UciOptionBlockBuilder::read_from(&mut reader)
            .await
            .unwrap()
            .unwrap()
            .unwrap();

        // Convert the builder to a full block - this test has all options so it should succeed
        let uci_option_block: UciOptionBlock = uci_option_block_builder.try_into().unwrap();

        assert_eq!(
            uci_option_block.debug_log_file,
            model::UciString(String::new())
        );
        assert_eq!(uci_option_block.numa_policy, model::NumaPolicy::Auto);
        assert_eq!(
            uci_option_block.threads,
            options::Spin {
                default: 1,
                min: 1,
                max: 1024,
            }
        );
        assert_eq!(
            uci_option_block.hash,
            options::Spin {
                default: 16,
                min: 1,
                max: 33554432,
            }
        );
        assert_eq!(uci_option_block.clear_hash, ());
        assert_eq!(uci_option_block.ponder, model::Check(false));
        assert_eq!(
            uci_option_block.multi_pv,
            options::Spin {
                default: 1,
                min: 1,
                max: 256,
            }
        );
        assert_eq!(
            uci_option_block.skill_level,
            options::Spin {
                default: 20,
                min: 0,
                max: 20,
            }
        );
        assert_eq!(
            uci_option_block.move_overhead,
            options::Spin {
                default: 10,
                min: 0,
                max: 5000,
            }
        );
        assert_eq!(
            uci_option_block.nodestime,
            options::Spin {
                default: 0,
                min: 0,
                max: 10000,
            }
        );
        assert_eq!(uci_option_block.uci_chess_960, model::Check(false));
        assert_eq!(uci_option_block.uci_limit_strength, model::Check(false));
        assert_eq!(
            uci_option_block.uci_elo,
            options::Spin {
                default: 1320,
                min: 1320,
                max: 3190,
            }
        );
        assert_eq!(uci_option_block.uci_show_wdl, model::Check(false));
        assert_eq!(
            uci_option_block.syzygy_path,
            model::UciString(String::new())
        );
        assert_eq!(
            uci_option_block.syzygy_probe_depth,
            options::Spin {
                default: 1,
                min: 1,
                max: 100,
            }
        );
        assert_eq!(uci_option_block.syzygy_50_move_rule, model::Check(true));
        assert_eq!(
            uci_option_block.syzygy_probe_limit,
            options::Spin {
                default: 7,
                min: 0,
                max: 7,
            }
        );
        assert_eq!(
            uci_option_block.eval_file,
            model::UciString("nn-1c0000000000.nnue".to_string())
        );
        assert_eq!(
            uci_option_block.eval_file_small,
            model::UciString("nn-37f18f62d772.nnue".to_string())
        );
    }
}
