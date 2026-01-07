use async_trait::async_trait;

use crate::{
    command,
    engine_commands::{
        IdBlock, IdBlockParsingError, OptionBlockParsingError, UciOkCommand,
        UciOkCommandParsingError, UciOptionBlockBuilder,
    },
    util::{AsyncReadable, LineHandlerOutcome, handle_next_line},
};

#[derive(Debug)]
pub struct UciCommandResponse {
    pub id_block: IdBlock,
    pub option_block: UciOptionBlockBuilder,
    pub uciok: UciOkCommand,
}

#[derive(Debug, thiserror::Error)]
pub enum UciCommandResponseParsingError {
    #[error("IdBlock parsing error: {0:?}")]
    IdBlockParsingError(IdBlockParsingError),
    // That's the behavior of Stockfish. It's kinda hacky but I don't know how to do better for now.
    #[error("Expected empty line after IdBlock.")]
    ExpectedEmptyLineAfterIdBlock,
    #[error("OptionBlock parsing error: {0:?}")]
    OptionBlockParsingError(OptionBlockParsingError),
    #[error("UciOkCommand parsing error: {0:?}")]
    UciOkCommandParsingError(UciOkCommandParsingError),
    #[error("Incomplete UCI command response.")]
    IncompleteResponse,
}

impl UciCommandResponseParsingError {
    fn wrap<RR>(
        self,
    ) -> Result<Option<Result<UciCommandResponse, command::parsing::Error<Self>>>, RR> {
        command::parsing::Error::from(self).wrap()
    }
}

#[async_trait(?Send)]
impl AsyncReadable for UciCommandResponse {
    type Err = command::parsing::Error<UciCommandResponseParsingError>;

    async fn read_from<R>(reader: &mut R) -> Result<Option<Result<Self, Self::Err>>, R::Error>
    where
        R: crate::util::StreamingLineReader,
    {
        let id_block = match IdBlock::read_from(reader).await? {
            Some(Ok(block)) => block,
            Some(Err(e)) => {
                return e
                    .map_custom(UciCommandResponseParsingError::IdBlockParsingError)
                    .wrap();
            }
            None => {
                return UciCommandResponseParsingError::IncompleteResponse.wrap();
            }
        };

        match handle_next_line(
            reader,
            |line: &str| -> LineHandlerOutcome<(), UciCommandResponseParsingError> {
                if line.trim().is_empty() {
                    LineHandlerOutcome::Read(())
                } else {
                    LineHandlerOutcome::Error(
                        UciCommandResponseParsingError::ExpectedEmptyLineAfterIdBlock,
                    )
                }
            },
        )
        .await?
        {
            Some(LineHandlerOutcome::Read(())) => (),
            Some(LineHandlerOutcome::Error(e)) => {
                return e.wrap();
            }
            Some(LineHandlerOutcome::Peeked) => {
                return command::parsing::Error::UnexpectedPeekOutput.wrap();
            }
            None => {
                return UciCommandResponseParsingError::IncompleteResponse.wrap();
            }
        };

        let option_block_builder = match UciOptionBlockBuilder::read_from(reader).await? {
            Some(Ok(builder)) => builder,
            Some(Err(e)) => {
                return e
                    .map_custom(UciCommandResponseParsingError::OptionBlockParsingError)
                    .wrap();
            }
            None => {
                return UciCommandResponseParsingError::IncompleteResponse.wrap();
            }
        };

        // Since all UCI options are optional, we just use the builder directly
        let option_block = option_block_builder;

        let uciok = match UciOkCommand::read_from(reader).await? {
            Some(Ok(cmd)) => cmd,
            Some(Err(e)) => {
                return e
                    .map_custom(UciCommandResponseParsingError::UciOkCommandParsingError)
                    .wrap();
            }
            None => {
                return UciCommandResponseParsingError::IncompleteResponse.wrap();
            }
        };

        Ok(Some(Ok(UciCommandResponse {
            id_block,
            option_block,
            uciok,
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{model, options::Spin};

    #[tokio::test]
    async fn test_read_uci_command_response() {
        let input = "id name Stockfish 17.1\n\
id author the Stockfish developers (see AUTHORS file)\n\
\n\
option name Debug Log File type string default <empty>\n\
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
uciok\n";

        let mut reader = tokio::io::BufReader::new(input.as_bytes());
        let uci_command_response = UciCommandResponse::read_from(&mut reader)
            .await
            .unwrap()
            .unwrap()
            .unwrap();

        assert_eq!(uci_command_response.id_block.name, "Stockfish 17.1");
        assert_eq!(
            uci_command_response.id_block.author,
            "the Stockfish developers (see AUTHORS file)"
        );
        assert_eq!(
            uci_command_response.option_block.debug_log_file,
            Some(model::UciString(String::new()))
        );
        assert_eq!(
            uci_command_response.option_block.numa_policy,
            Some(model::NumaPolicy::Auto)
        );
        assert_eq!(
            uci_command_response.option_block.threads,
            Some(Spin {
                default: 1,
                min: 1,
                max: 1024,
            })
        );
        assert_eq!(
            uci_command_response.option_block.hash,
            Some(Spin {
                default: 16,
                min: 1,
                max: 33554432,
            })
        );
        assert_eq!(uci_command_response.option_block.clear_hash, Some(()));
        assert_eq!(
            uci_command_response.option_block.ponder,
            Some(model::Check(false))
        );
        assert_eq!(
            uci_command_response.option_block.multi_pv,
            Some(Spin {
                default: 1,
                min: 1,
                max: 256,
            })
        );
        assert_eq!(
            uci_command_response.option_block.skill_level,
            Some(Spin {
                default: 20,
                min: 0,
                max: 20,
            })
        );
        assert_eq!(
            uci_command_response.option_block.move_overhead,
            Some(Spin {
                default: 10,
                min: 0,
                max: 5000,
            })
        );
        assert_eq!(
            uci_command_response.option_block.nodestime,
            Some(Spin {
                default: 0,
                min: 0,
                max: 10000,
            })
        );
        assert_eq!(
            uci_command_response.option_block.uci_chess_960,
            Some(model::Check(false))
        );
        assert_eq!(
            uci_command_response.option_block.uci_limit_strength,
            Some(model::Check(false))
        );
        assert_eq!(
            uci_command_response.option_block.uci_elo,
            Some(Spin {
                default: 1320,
                min: 1320,
                max: 3190,
            })
        );
        assert_eq!(
            uci_command_response.option_block.uci_show_wdl,
            Some(model::Check(false))
        );
        assert_eq!(
            uci_command_response.option_block.syzygy_path,
            Some(model::UciString(String::new()))
        );
        assert_eq!(
            uci_command_response.option_block.syzygy_probe_depth,
            Some(Spin {
                default: 1,
                min: 1,
                max: 100,
            })
        );
        assert_eq!(
            uci_command_response.option_block.syzygy_50_move_rule,
            Some(model::Check(true))
        );
        assert_eq!(
            uci_command_response.option_block.syzygy_probe_limit,
            Some(Spin {
                default: 7,
                min: 0,
                max: 7,
            })
        );
        assert_eq!(
            uci_command_response.option_block.eval_file,
            Some(model::UciString("nn-1c0000000000.nnue".to_string()))
        );
        assert_eq!(
            uci_command_response.option_block.eval_file_small,
            Some(model::UciString("nn-37f18f62d772.nnue".to_string()))
        );
    }
}
