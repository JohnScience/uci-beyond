use std::fmt::Display;

use async_trait::async_trait;

use crate::{
    command,
    util::{AsyncReadable, LineHandlerOutcome, StreamingLineReader, handle_next_line},
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
        // Skip empty lines before uciok
        loop {
            let f = |line: &str| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    // Skip empty lines
                    LineHandlerOutcome::Read(None)
                } else if trimmed == "uciok" {
                    LineHandlerOutcome::Read(Some(UciOkCommand))
                } else {
                    LineHandlerOutcome::Error(command::parsing::Error::CustomError(
                        UciOkCommandParsingError(line.to_string()),
                    ))
                }
            };

            match handle_next_line(reader, f).await? {
                Some(LineHandlerOutcome::Read(Some(cmd))) => return Ok(Some(Ok(cmd))),
                Some(LineHandlerOutcome::Read(None)) => continue, // Skip empty line and try next
                Some(LineHandlerOutcome::Error(e)) => return Ok(Some(Err(e))),
                Some(LineHandlerOutcome::Peeked) => {
                    return command::parsing::Error::UnexpectedPeekOutput.wrap();
                }
                None => return Ok(None),
            }
        }
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
