use std::str::FromStr;

use async_trait::async_trait;
use kinded::Kinded;
use optional_struct::optional_struct;
use variants_data_struct::VariantsDataStruct;

use crate::command;
use crate::command::Command;
use crate::util::{AsyncReadable, LineHandlerOutcome, StreamingLineReader, handle_next_line};

// TODO: reimplement parsing using better abstractions

/// <https://backscattering.de/chess/uci/#engine-id>
#[derive(VariantsDataStruct, Debug, Kinded)]
#[variants_data_struct(
    name=IdBlock,
    attrs(
        #[derive(Debug)]
        #[optional_struct]
        /// The block of [`IdCommand`]s sent by the engine to identify itself.
    )
)]
#[kinded(
    kind = IdCommandKind
)]
pub enum IdCommand {
    /// This must be sent after receiving the UCI command to identify the engine, e.g. `id name Shredder X.Y\n`
    #[variants_data_struct_field(field_ty_override = String)]
    Name(String),
    /// This must be sent after receiving the UCI command to identify the engine, e.g. `id author Stefan MK\n`
    #[variants_data_struct_field(field_ty_override = String)]
    Author(String),
}

#[derive(thiserror::Error, Debug)]
pub enum IdCommandParsingError {
    #[error("Wrong field: `{0}`.")]
    WrongField(String),
}

#[derive(thiserror::Error, Debug)]
pub enum IdBlockParsingError {
    #[error("Command error: {0:?}")]
    CommandError(#[from] command::parsing::Error<IdCommandParsingError>),
    #[error("Repeated field: {0}")]
    RepeatedField(IdCommandKind),
    #[error("Incomplete IdBlock")]
    IncompleteBlock,
}

impl IdBlockParsingError {
    pub fn wrap<RR>(self) -> Result<Option<Result<IdBlock, command::parsing::Error<Self>>>, RR> {
        command::parsing::Error::from(self).wrap()
    }
}

impl command::Command for IdCommand {
    type ParsingError = IdCommandParsingError;

    const NAME: &'static str = "id";
}

impl std::fmt::Display for IdCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdCommand::Name(name) => write!(f, "id name {name}"),
            IdCommand::Author(author) => write!(f, "id author {author}"),
        }
    }
}

impl FromStr for IdCommand {
    type Err = command::parsing::Error<IdCommandParsingError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = IdCommand::parse_cmd_name(s)?;

        let (field, value) = s
            .split_once(' ')
            .ok_or(command::parsing::Error::UnexpectedFormat)?;

        let value = value.trim_end();

        match field {
            "name" => Ok(IdCommand::Name(value.to_string())),
            "author" => Ok(IdCommand::Author(value.to_string())),
            _ => Err(command::parsing::Error::CustomError(
                IdCommandParsingError::WrongField(field.to_string()),
            )),
        }
    }
}

#[async_trait(?Send)]
impl AsyncReadable for IdCommand {
    // Only the inner parsing error type
    type Err = command::parsing::Error<IdCommandParsingError>;
    async fn read_from<R>(reader: &mut R) -> Result<Option<Result<Self, Self::Err>>, R::Error>
    where
        R: StreamingLineReader,
    {
        let f = |line: &str| -> LineHandlerOutcome<IdCommand, <IdCommand as FromStr>::Err> {
            match line.parse::<IdCommand>() {
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

#[async_trait(?Send)]
impl AsyncReadable for IdBlock {
    // Only the inner parsing error type
    type Err = command::parsing::Error<IdBlockParsingError>;

    async fn read_from<R>(reader: &mut R) -> Result<Option<Result<Self, Self::Err>>, R::Error>
    where
        R: StreamingLineReader,
    {
        let mut opt_id_block = OptionalIdBlock::default();
        let mut i = 0;

        loop {
            let cmd: Option<Result<IdCommand, <IdCommand as FromStr>::Err>> =
                IdCommand::read_from(reader).await?;

            let Some(cmd) = cmd else {
                if i == 0 {
                    // No IdCommands were read; return None
                    return Ok(None);
                }
                // EOF reached before completing the block
                return IdBlockParsingError::IncompleteBlock.wrap();
            };

            let cmd = match cmd {
                Ok(cmd) => cmd,
                Err(e) => {
                    return IdBlockParsingError::from(e).wrap();
                }
            };

            match cmd {
                IdCommand::Name(name) => {
                    if opt_id_block.name.is_some() {
                        return IdBlockParsingError::RepeatedField(IdCommandKind::Name).wrap();
                    }
                    opt_id_block.name = Some(name);
                }
                IdCommand::Author(author) => {
                    if opt_id_block.author.is_some() {
                        return IdBlockParsingError::RepeatedField(IdCommandKind::Author).wrap();
                    }
                    opt_id_block.author = Some(author);
                }
            }

            // When OptionalIdBlock becomes complete, convert it to IdBlock.
            opt_id_block = match opt_id_block.try_into() {
                Ok(id_block) => return Ok(Some(Ok(id_block))),
                Err(e) => e,
            };

            i += 1;
        }
    }
}

impl From<IdCommandParsingError> for IdBlockParsingError {
    fn from(err: IdCommandParsingError) -> Self {
        IdBlockParsingError::CommandError(command::parsing::Error::CustomError(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_stockfish_output_imitation() {
        let id_name = IdCommand::Name("Stockfish 17.1".to_string());
        let id_author =
            IdCommand::Author("the Stockfish developers (see AUTHORS file)".to_string());

        assert_eq!(id_name.to_string(), "id name Stockfish 17.1");
        assert_eq!(
            id_author.to_string(),
            "id author the Stockfish developers (see AUTHORS file)"
        );
    }

    #[tokio::test]
    async fn test_parse_stockfish_output_imitation_as_id_commands() {
        use assert_matches::assert_matches;

        let id_block_str: &str = "id name Stockfish 17.1\n\
               id author the Stockfish developers (see AUTHORS file)\n";

        let mut reader = tokio::io::BufReader::new(id_block_str.as_bytes());

        let parsed_id_name = IdCommand::read_from(&mut reader)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let parsed_id_author = IdCommand::read_from(&mut reader)
            .await
            .unwrap()
            .unwrap()
            .unwrap();

        assert_matches!(
            parsed_id_name,
            IdCommand::Name(name) if name == "Stockfish 17.1"
        );
        assert_matches!(
            parsed_id_author,
            IdCommand::Author(author) if author == "the Stockfish developers (see AUTHORS file)"
        );
    }

    #[tokio::test]
    async fn test_parse_stockfish_output_imitation_as_id_block() {
        use assert_matches::assert_matches;

        let id_block_str: &str = "id name Stockfish 17.1\n\
               id author the Stockfish developers (see AUTHORS file)\n";

        let mut reader = tokio::io::BufReader::new(id_block_str.as_bytes());

        let parsed_id_block = IdBlock::read_from(&mut reader)
            .await
            .unwrap()
            .unwrap()
            .unwrap();

        assert_matches!(
            parsed_id_block,
            IdBlock {
                name,
                author,
            } if name == "Stockfish 17.1" && author == "the Stockfish developers (see AUTHORS file)"
        );
    }
}
