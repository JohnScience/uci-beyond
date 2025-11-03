use crate::util::{StreamingLineHandler, UciBufReadError, UciLinesHandler};

pub trait StreamingLineHandlerExt {
    /// The error type the handler will use (publicly exposed).
    type HandlerError<E>: Send
    where
        E: Send;

    type HandlerOutput<O>: Send
    where
        O: Send;

    /// The concrete handler type, parameterized by a line function.
    type LineHandler<'a, F, O, E>: StreamingLineHandler<Output = Self::HandlerOutput<O>, Error = Self::HandlerError<E>>
    where
        Self: 'a,
        F: FnMut(&str) -> Result<Self::HandlerOutput<O>, Self::HandlerError<E>> + Send + Unpin,
        O: Send,
        E: Send;

    /// Constructs a handler from the given function `f`.
    fn make_line_handler<'a, F, O, E>(&'a mut self, f: F) -> Self::LineHandler<'a, F, O, E>
    where
        F: FnMut(&str) -> Result<Self::HandlerOutput<O>, Self::HandlerError<E>> + Send + Unpin,
        O: Send,
        E: Send;
}

impl<R> StreamingLineHandlerExt for tokio::io::BufReader<R>
where
    R: tokio::io::AsyncRead + Unpin + Send,
{
    type HandlerError<E>
        = UciBufReadError<E>
    where
        E: Send;

    /// Each line handler may or may not produce a value.
    type HandlerOutput<O>
        = O
    where
        O: Send;

    type LineHandler<'a, F, O, E>
        = UciLinesHandler<'a, R, F, O, E>
    where
        R: 'a,
        F: FnMut(&str) -> Result<O, UciBufReadError<E>> + Send + Unpin,
        O: Send,
        E: Send;

    fn make_line_handler<'a, F, O, E>(&'a mut self, f: F) -> Self::LineHandler<'a, F, O, E>
    where
        F: FnMut(&str) -> Result<O, UciBufReadError<E>> + Send + Unpin,
        O: Send,
        E: Send,
    {
        UciLinesHandler::new(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::UciBufReadError;
    use tokio::io::BufReader;

    #[tokio::test]
    async fn test_make_line_handler_reads_lines() {
        // Prepare in-memory input (3 lines)
        let input_data = b"line1\nline2\nline3\n";
        let cursor = std::io::Cursor::new(input_data);
        let mut reader = BufReader::new(cursor);

        // Simple line handler function: just return the line as String
        let line_fn =
            |line: &str| -> Result<String, UciBufReadError<()>> { Ok(line.trim().to_string()) };

        // Create the line handler via the trait
        let mut handler = reader.make_line_handler(line_fn);

        // Collect all lines
        let mut collected = Vec::new();
        loop {
            use std::future::poll_fn;

            let fut = poll_fn(|cx| {
                // call the provided `handle` method on the pinned handler
                let mut handler = core::pin::Pin::new(&mut handler);
                let handler = handler.as_mut();
                handler.handle(cx)
            });

            let res = fut.await;

            match res {
                Ok(Some(line)) => collected.push(line),
                Ok(None) => break,
                Err(_) => panic!("Unexpected error"),
            }
        }

        // Check that all lines were read correctly
        assert_eq!(
            collected,
            vec![
                "line1".to_string(),
                "line2".to_string(),
                "line3".to_string()
            ]
        );
    }
}
