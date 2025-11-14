use std::fmt::Display;

use async_trait::async_trait;

use crate::{
    command,
    util::{AsyncReadable, StreamingLineReader, handle_next_line},
};

#[derive(Debug)]
pub struct UciOkCommand;

impl Display for UciOkCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "uciok")
    }
}

#[derive(Debug, thiserror::Error)]
#[error("UciOkCommand parsing error. Unexpected input: {0}")]
pub struct UciOkCommandParsingError(String);

#[async_trait(?Send)]
impl AsyncReadable for UciOkCommand {
    type Err = command::parsing::Error<UciOkCommandParsingError>;

    async fn read_from<R>(reader: &mut R) -> Result<Option<Result<Self, Self::Err>>, R::Error>
    where
        R: StreamingLineReader,
    {
        let f = |line: &str| {
            if line.trim() == "uciok" {
                Ok(UciOkCommand)
            } else {
                Err(command::parsing::Error::CustomError(
                    UciOkCommandParsingError(line.to_string()),
                ))
            }
        };

        let res = handle_next_line(reader, f).await?;

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_uciok_command() {
        let input = "uciok\n";

        let cursor = std::io::Cursor::new(input);
        let mut reader = tokio::io::BufReader::new(cursor);

        let uciok_command = UciOkCommand::read_from(&mut reader)
            .await
            .unwrap()
            .unwrap()
            .unwrap();

        assert!(matches!(uciok_command, UciOkCommand));
    }
}
