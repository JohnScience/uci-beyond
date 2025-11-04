use async_trait::async_trait;
use tokio::io::AsyncRead;

use crate::{
    command,
    engine_commands::{self, OptionCommand},
    options::UciOption,
    util::{AsyncReadable, UciBufReadError},
};

// UciOptionBlock is defined there because the UciOption enum is in the options module
pub use crate::options::{UciOptionBlock, UciOptionBlockBuilder};

#[derive(Debug)]
pub enum OptionBlockParsingError {
    CommandErrorParsingError(engine_commands::OptionCommandParsingError),
}

//#[async_trait]
//impl AsyncReadable for UciOptionBlock {
//    type Err = UciBufReadError<command::parsing::Error<OptionBlockParsingError>>;
//
//    async fn read_from<R: AsyncRead + Unpin + Send>(
//        reader: &mut tokio::io::BufReader<R>,
//    ) -> Result<Self, Self::Err> {
//        let mut b = UciOptionBlockBuilder::default();
//
//        loop {
//            let cmd = OptionCommand::read_from(reader)
//                .await
//                .map_err(|e| e.map_parsing_custom(From::from))?;
//
//            let uci_option: UciOption = cmd.0;
//
//            match uci_option {
//                UciOption::ClearHash => b.clear_hash = Some(()),
//                UciOption::DebugLogFile { default } => b.debug_log_file = Some(default),
//                UciOption::EvalFile { default } => b.eval_file = Some(default),
//                UciOption::EvalFileSmall { default } => b.eval_file_small = Some(default),
//                UciOption::Hash(spin) => b.hash = Some(spin),
//                UciOption::MoveOverhead(spin) => b.move_overhead = Some(spin),
//                UciOption::MultiPV(spin) => b.multi_pv = Some(spin),
//                UciOption::Nodestime(spin) => b.nodestime = Some(spin),
//                UciOption::NumaPolicy { default } => b.numa_policy = Some(default),
//                UciOption::Ponder { default } => b.ponder = Some(default),
//                UciOption::SkillLevel(spin) => b.skill_level = Some(spin),
//                UciOption::Syzygy50MoveRule { default } => b.syzygy_50_move_rule = Some(default),
//                UciOption::SyzygyPath { default } => b.syzygy_path = Some(default),
//                UciOption::SyzygyProbeDepth(spin) => b.syzygy_probe_depth = Some(spin),
//                UciOption::SyzygyProbeLimit(spin) => b.syzygy_probe_limit = Some(spin),
//                UciOption::Threads(spin) => b.threads = Some(spin),
//                UciOption::UCIChess960 { default } => b.uci_chess_960 = Some(default),
//                UciOption::UCIElo(spin) => b.uci_elo = Some(spin),
//                UciOption::UCILimitStrength { default } => b.uci_limit_strength = Some(default),
//                UciOption::UCIShowWDL { default } => b.uci_show_wdl = Some(default),
//            };
//
//            b = match b.try_into() {
//                Ok(id_block) => return Ok(id_block),
//                Err(e) => e,
//            };
//        }
//    }
//}

impl From<engine_commands::OptionCommandParsingError> for OptionBlockParsingError {
    fn from(err: engine_commands::OptionCommandParsingError) -> Self {
        OptionBlockParsingError::CommandErrorParsingError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{model, options};

    //    #[tokio::test]
    //    async fn test_parse_uci_option_block() {
    //        let input = "option name Debug Log File type string default <empty>\n\
    //        option name NumaPolicy type string default auto\n\
    //        option name Threads type spin default 1 min 1 max 1024\n\
    //        option name Hash type spin default 16 min 1 max 33554432\n\
    //        option name Clear Hash type button\n\
    //        option name Ponder type check default false\n\
    //        option name MultiPV type spin default 1 min 1 max 256\n\
    //        option name Skill Level type spin default 20 min 0 max 20\n\
    //        option name Move Overhead type spin default 10 min 0 max 5000\n\
    //        option name nodestime type spin default 0 min 0 max 10000\n\
    //        option name UCI_Chess960 type check default false\n\
    //        option name UCI_LimitStrength type check default false\n\
    //        option name UCI_Elo type spin default 1320 min 1320 max 3190\n\
    //        option name UCI_ShowWDL type check default false\n\
    //        option name SyzygyPath type string default <empty>\n\
    //        option name SyzygyProbeDepth type spin default 1 min 1 max 100\n\
    //        option name Syzygy50MoveRule type check default true\n\
    //        option name SyzygyProbeLimit type spin default 7 min 0 max 7\n\
    //        option name EvalFile type string default nn-1c0000000000.nnue\n\
    //        option name EvalFileSmall type string default nn-37f18f62d772.nnue\n";
    //
    //        let mut reader = tokio::io::BufReader::new(input.as_bytes());
    //        let uci_option_block = UciOptionBlock::read_from(&mut reader)
    //            .await
    //            .expect("Failed to parse UciOptionBlock");
    //
    //        assert_eq!(
    //            uci_option_block.debug_log_file,
    //            model::UciString(String::new())
    //        );
    //        assert_eq!(uci_option_block.numa_policy, model::NumaPolicy::Auto);
    //        assert_eq!(
    //            uci_option_block.threads,
    //            options::Spin {
    //                default: 1,
    //                min: 1,
    //                max: 1024,
    //            }
    //        );
    //        assert_eq!(
    //            uci_option_block.hash,
    //            options::Spin {
    //                default: 16,
    //                min: 1,
    //                max: 33554432,
    //            }
    //        );
    //        assert_eq!(uci_option_block.clear_hash, ());
    //        assert_eq!(uci_option_block.ponder, model::Check(false));
    //        assert_eq!(
    //            uci_option_block.multi_pv,
    //            options::Spin {
    //                default: 1,
    //                min: 1,
    //                max: 256,
    //            }
    //        );
    //        assert_eq!(
    //            uci_option_block.skill_level,
    //            options::Spin {
    //                default: 20,
    //                min: 0,
    //                max: 20,
    //            }
    //        );
    //        assert_eq!(
    //            uci_option_block.move_overhead,
    //            options::Spin {
    //                default: 10,
    //                min: 0,
    //                max: 5000,
    //            }
    //        );
    //        assert_eq!(
    //            uci_option_block.nodestime,
    //            options::Spin {
    //                default: 0,
    //                min: 0,
    //                max: 10000,
    //            }
    //        );
    //        assert_eq!(uci_option_block.uci_chess_960, model::Check(false));
    //        assert_eq!(uci_option_block.uci_limit_strength, model::Check(false));
    //        assert_eq!(
    //            uci_option_block.uci_elo,
    //            options::Spin {
    //                default: 1320,
    //                min: 1320,
    //                max: 3190,
    //            }
    //        );
    //        assert_eq!(uci_option_block.uci_show_wdl, model::Check(false));
    //        assert_eq!(
    //            uci_option_block.syzygy_path,
    //            model::UciString(String::new())
    //        );
    //        assert_eq!(
    //            uci_option_block.syzygy_probe_depth,
    //            options::Spin {
    //                default: 1,
    //                min: 1,
    //                max: 100,
    //            }
    //        );
    //        assert_eq!(uci_option_block.syzygy_50_move_rule, model::Check(true));
    //        assert_eq!(
    //            uci_option_block.syzygy_probe_limit,
    //            options::Spin {
    //                default: 7,
    //                min: 0,
    //                max: 7,
    //            }
    //        );
    //        assert_eq!(
    //            uci_option_block.eval_file,
    //            model::UciString("nn-1c0000000000.nnue".to_string())
    //        );
    //        assert_eq!(
    //            uci_option_block.eval_file_small,
    //            model::UciString("nn-37f18f62d772.nnue".to_string())
    //        );
    //    }
}
