use core::task::{Context, Poll};

mod string_stream_reader;
mod tokio_io_bufreader_impl;

pub use string_stream_reader::StringStreamReader;

pub trait StreamingLineReader: Unpin + Send {
    /// The type of error that can occur while reading a line.
    type Error: Send;

    const AUTO_CONSUMING: bool;

    /// The type representing a borrowed line slice.
    type Line<'a>: AsRef<str>
    where
        Self: 'a;

    /// Try to read the next line.
    ///
    /// - Returns `Poll::Pending` if no new line is ready.
    /// - Returns `Poll::Ready(Ok(Some(line)))` when a line is available.
    /// - Returns `Poll::Ready(Ok(None))` on EOF.
    /// - Returns `Poll::Ready(Err(err))` on I/O error.
    fn next_line<'a>(
        self: &'a mut Self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<Self::Line<'a>>, Self::Error>>;

    fn consume_line_manually(&mut self, line_len: usize);
}

pub async fn handle_next_line<R, F, O, E>(
    reader: &mut R,
    mut f: F,
) -> Result<Option<Result<O, E>>, R::Error>
where
    R: StreamingLineReader,
    F: FnMut(&str) -> Result<O, E> + Send + Unpin,
    O: Send,
{
    use core::future::poll_fn;

    poll_fn(|cx| {
        let (poll, len) = match reader.next_line(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Ready(Ok(Some(line))) => {
                let line_str: &str = line.as_ref();
                let len = line_str.len();

                let o = f(line_str);

                (Poll::Ready(Ok(Some(o))), len)
            }
            Poll::Ready(Ok(None)) => return Poll::Ready(Ok(None)),
        };

        // If not auto-consuming, consume manually
        if !R::AUTO_CONSUMING {
            reader.consume_line_manually(len);
        }

        poll
    })
    .await
}
