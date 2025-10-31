use std::str::FromStr;

use async_trait::async_trait;
use kinded::Kinded;
use optional_struct::optional_struct;
use tokio::io::AsyncRead;
use variants_data_struct::VariantsDataStruct;

use crate::command;
use crate::command::Command;
use crate::util::{AsyncReadable, UciBufReadError};

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

#[derive(Debug)]
pub enum IdCommandParsingError {
    WrongField(String),
}

#[derive(Debug)]
pub enum IdBlockParsingError {
    CommandError(command::parsing::Error<IdCommandParsingError>),
    RepeatedField(IdCommandKind),
    IncompleteBlock,
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
        let s = IdCommand::parse_name(s)?;

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

#[async_trait]
impl AsyncReadable for IdCommand {
    type Err = UciBufReadError<command::parsing::Error<IdCommandParsingError>>;

    async fn read_from<R: AsyncRead + Unpin + Send>(
        reader: &mut tokio::io::BufReader<R>,
    ) -> Result<Self, Self::Err> {
        use crate::util::UciBufReadExtAsync;

        let res: Option<IdCommand> = reader
            .with_next_line(|line| {
                let cmd = line
                    .parse::<IdCommand>()
                    .map_err(|e| UciBufReadError::CustomError(e))?;
                Ok(Some(cmd))
            })
            .await?;

        let Some(cmd) = res else {
            return Err(UciBufReadError::CustomError(
                command::parsing::Error::UnexpectedEof,
            ));
        };

        Ok(cmd)
    }
}

impl From<command::parsing::Error<IdCommandParsingError>> for IdBlockParsingError {
    fn from(err: command::parsing::Error<IdCommandParsingError>) -> Self {
        IdBlockParsingError::CommandError(err)
    }
}

impl From<IdCommandParsingError> for IdBlockParsingError {
    fn from(err: IdCommandParsingError) -> Self {
        IdBlockParsingError::CommandError(command::parsing::Error::CustomError(err))
    }
}

#[async_trait]
impl AsyncReadable for IdBlock {
    type Err = UciBufReadError<command::parsing::Error<IdBlockParsingError>>;

    async fn read_from<R: AsyncRead + Unpin + Send>(
        reader: &mut tokio::io::BufReader<R>,
    ) -> Result<Self, Self::Err> {
        let mut opt_id_block = OptionalIdBlock::default();

        loop {
            let cmd = IdCommand::read_from(reader)
                .await
                .map_err(|e| e.map_parsing_custom(From::from))?;

            match cmd {
                IdCommand::Name(name) => {
                    if opt_id_block.name.is_some() {
                        return Err(UciBufReadError::from_parsing_custom(
                            IdBlockParsingError::RepeatedField(IdCommandKind::Name),
                        ));
                    }
                    opt_id_block.name = Some(name);
                }
                IdCommand::Author(author) => {
                    if opt_id_block.author.is_some() {
                        return Err(UciBufReadError::from_parsing_custom(
                            IdBlockParsingError::RepeatedField(IdCommandKind::Author),
                        ));
                    }
                    opt_id_block.author = Some(author);
                }
            }

            opt_id_block = match opt_id_block.try_into() {
                Ok(id_block) => return Ok(id_block),
                Err(e) => e,
            };
        }
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

        let parsed_id_name = IdCommand::read_from(&mut reader).await.unwrap();
        let parsed_id_author = IdCommand::read_from(&mut reader).await.unwrap();

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

        let parsed_id_block = IdBlock::read_from(&mut reader).await.unwrap();

        assert_matches!(
            parsed_id_block,
            IdBlock {
                name,
                author,
            } if name == "Stockfish 17.1" && author == "the Stockfish developers (see AUTHORS file)"
        );
    }
}
