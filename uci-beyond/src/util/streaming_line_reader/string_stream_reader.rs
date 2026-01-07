use futures::stream::Stream;

use crate::util::StreamingLineReader;

pub struct StringStreamReader<E, S>
where
    S: Stream<Item = Result<String, E>> + Unpin + Send,
    E: Send,
{
    stream: S,
    /// Buffer to hold the current line that may be peeked at without consuming.
    /// When a line is read but not consumed (peeked), it remains here for the next read.
    current_line: Option<String>,
}

impl<E, S> StringStreamReader<E, S>
where
    S: Stream<Item = Result<String, E>> + Unpin + Send,
    E: Send,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            current_line: None,
        }
    }
}

impl<E, S> StreamingLineReader for StringStreamReader<E, S>
where
    S: Stream<Item = Result<String, E>> + Unpin + Send,
    E: Send,
{
    type Error = E;

    const AUTO_CONSUMING: bool = false;

    type Line<'a>
        = &'a String
    where
        Self: 'a;

    fn next_line<'a>(
        self: &'a mut Self,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<Option<Self::Line<'a>>, Self::Error>> {
        use futures::TryStreamExt as _;

        // If we already have a buffered line (from a previous peek), return it
        if self.current_line.is_some() {
            return core::task::Poll::Ready(Ok(self.current_line.as_ref()));
        }

        // Otherwise, poll the stream for the next line
        match futures::ready!(self.stream.try_poll_next_unpin(cx)) {
            Some(Ok(line)) => {
                self.current_line = Some(line);
                core::task::Poll::Ready(Ok(self.current_line.as_ref()))
            }
            Some(Err(e)) => core::task::Poll::Ready(Err(e)),
            None => core::task::Poll::Ready(Ok(None)),
        }
    }

    fn consume_line_manually(&mut self, _line_len: usize) {
        // Clear the current line buffer to advance to the next line
        self.current_line = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;

    #[tokio::test]
    async fn test_peek_behavior() {
        use crate::util::LineHandlerOutcome;
        use crate::util::handle_next_line;

        let lines: Vec<Result<String, std::io::Error>> = vec![
            Ok("line1".to_string()),
            Ok("line2".to_string()),
            Ok("line3".to_string()),
        ];
        let stream = stream::iter(lines);
        let mut reader = StringStreamReader::new(stream);

        // Read first line normally
        let result = handle_next_line(&mut reader, |line| -> LineHandlerOutcome<String, &str> {
            if line == "line1" {
                LineHandlerOutcome::Read(line.to_string())
            } else {
                LineHandlerOutcome::Error("unexpected")
            }
        })
        .await
        .unwrap();
        assert!(matches!(result, Some(LineHandlerOutcome::Read(_))));

        // Peek at second line (don't consume)
        let result = handle_next_line(&mut reader, |line| -> LineHandlerOutcome<(), &str> {
            if line == "line2" {
                LineHandlerOutcome::Peeked
            } else {
                LineHandlerOutcome::Error("unexpected")
            }
        })
        .await
        .unwrap();
        assert!(matches!(result, Some(LineHandlerOutcome::Peeked)));

        // Read second line again (should still be there)
        let result = handle_next_line(&mut reader, |line| -> LineHandlerOutcome<String, &str> {
            if line == "line2" {
                LineHandlerOutcome::Read(line.to_string())
            } else {
                LineHandlerOutcome::Error("unexpected")
            }
        })
        .await
        .unwrap();
        assert!(matches!(result, Some(LineHandlerOutcome::Read(_))));

        // Read third line normally
        let result = handle_next_line(&mut reader, |line| -> LineHandlerOutcome<String, &str> {
            if line == "line3" {
                LineHandlerOutcome::Read(line.to_string())
            } else {
                LineHandlerOutcome::Error("unexpected")
            }
        })
        .await
        .unwrap();
        assert!(matches!(result, Some(LineHandlerOutcome::Read(_))));
    }
}
