use tokio::io::AsyncRead;

use crate::util::{StreamingLineHandler, UciBufReadError};

pub struct UciLinesHandler<'a, R, F, O, E>
where
    R: AsyncRead + Unpin + Send,
    F: FnMut(&str) -> Result<O, UciBufReadError<E>> + Send + Unpin,
{
    reader: &'a mut tokio::io::BufReader<R>,
    f: F,
}

impl<'a, R, F, O, E> UciLinesHandler<'a, R, F, O, E>
where
    R: AsyncRead + Unpin + Send,
    F: FnMut(&str) -> Result<O, UciBufReadError<E>> + Send + Unpin,
{
    pub fn new(reader: &'a mut tokio::io::BufReader<R>, f: F) -> Self {
        UciLinesHandler { reader, f }
    }
}

fn str_prefix(bytes: &[u8]) -> &str {
    let n = match std::str::from_utf8(bytes) {
        Ok(s) => return s,
        Err(e) => e.valid_up_to(),
    };

    std::str::from_utf8(&bytes[..n]).unwrap()
}

fn reader_next_line<'a, R: AsyncRead + Unpin + Send>(
    reader: std::pin::Pin<&'a mut tokio::io::BufReader<R>>,
    cx: &mut core::task::Context<'_>,
) -> core::task::Poll<std::io::Result<&'a str>> {
    let buf = match tokio::io::AsyncBufRead::poll_fill_buf(reader, cx) {
        core::task::Poll::Pending => return core::task::Poll::Pending,
        core::task::Poll::Ready(Ok(buf)) => buf,
        core::task::Poll::Ready(Err(e)) => return core::task::Poll::Ready(Err(e)),
    };

    if buf.is_empty() {
        return core::task::Poll::Ready(Ok(""));
    }

    let buf = str_prefix(buf);

    let eol_pos = match buf.find('\n') {
        Some(pos) => pos,
        None => return core::task::Poll::Pending, // wait for more data
    };

    let line = &buf[..eol_pos + "\n".len()];

    // Do NOT consume buffer here — caller will decide after parsing
    core::task::Poll::Ready(Ok(line))
}

impl<'a, R, F, O, E> core::future::Future for UciLinesHandler<'a, R, F, O, E>
where
    R: tokio::io::AsyncRead + core::marker::Unpin + Send,
    F: FnMut(&str) -> Result<O, UciBufReadError<E>> + Send + core::marker::Unpin,
{
    type Output = Result<Option<O>, UciBufReadError<E>>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        <Self as StreamingLineHandler>::handle(self, cx)
    }
}

impl<'a, R, F, O, E> StreamingLineHandler for UciLinesHandler<'a, R, F, O, E>
where
    R: AsyncRead + Unpin + Send,
    F: FnMut(&str) -> Result<O, UciBufReadError<E>> + Send + Unpin,
{
    // Errors produced by the overall handler (wraps parsing/custom errors)
    type Error = UciBufReadError<E>;

    // Final output produced by the handler when it finishes successfully
    type Output = O;

    // The concrete reader type we operate on
    type Reader = tokio::io::BufReader<R>;

    // A reference to a UTF-8-valid prefix of the buffer
    type Line<'b>
        = &'b str
    where
        'a: 'b,
        Self: 'b;

    // Errors produced by next_line (poll-level), convertible into Self::Error
    type NextLineError = std::io::Error;

    // The function's raw output and the function type
    type FnOut = Result<O, UciBufReadError<E>>;
    type F = F;

    fn next_line<'b>(
        reader: std::pin::Pin<&'b mut Self::Reader>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<Option<Self::Line<'b>>, Self::NextLineError>>
    where
        Self: 'b,
    {
        match reader_next_line(reader, cx) {
            core::task::Poll::Pending => core::task::Poll::Pending,
            core::task::Poll::Ready(Err(e)) => core::task::Poll::Ready(Err(e)),
            core::task::Poll::Ready(Ok(line)) => {
                if line.is_empty() {
                    core::task::Poll::Ready(Ok(None))
                } else {
                    core::task::Poll::Ready(Ok(Some(line)))
                }
            }
        }
    }

    fn split_into_parts(&mut self) -> (&mut Self::Reader, &mut Self::F) {
        (&mut *self.reader, &mut self.f)
    }

    // The finalize error is the same shape as the handler error here
    type FinalizeError = UciBufReadError<E>;

    // Finalize simply turns the FnOut (Result<Option<O>, UciBufReadError<E>>)
    // into the handler Output (Option<O>) or returns the error.
    fn finalize(
        reader: &mut Self::Reader,
        line_len: usize,
        o: Self::FnOut,
    ) -> Result<Self::Output, Self::FinalizeError> {
        use tokio::io::AsyncBufReadExt as _;

        if o.is_ok() {
            reader.consume(line_len);
        }

        o
    }
}
