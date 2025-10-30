use async_trait::async_trait;
use tokio::io::AsyncRead;

use crate::command;

#[derive(Debug)]
pub enum UciBufReadError<E> {
    IoError(std::io::Error),
    CustomError(E),
}

impl<E> From<E> for UciBufReadError<E> {
    fn from(e: E) -> Self {
        UciBufReadError::CustomError(e)
    }
}

impl<E> UciBufReadError<E> {
    pub fn map_custom<F, O>(self, f: F) -> UciBufReadError<O>
    where
        F: FnOnce(E) -> O,
    {
        match self {
            UciBufReadError::IoError(e) => UciBufReadError::IoError(e),
            UciBufReadError::CustomError(e) => UciBufReadError::CustomError(f(e)),
        }
    }
}

impl<E> UciBufReadError<command::parsing::Error<E>> {
    pub fn map_parsing_custom<F, O>(self, f: F) -> UciBufReadError<command::parsing::Error<O>>
    where
        F: FnOnce(E) -> O,
    {
        self.map_custom(|e| e.map_custom(f))
    }

    pub fn from_parsing_custom(err: E) -> Self {
        UciBufReadError::CustomError(command::parsing::Error::CustomError(err))
    }
}

#[async_trait]
pub trait AsyncReadable: Sized {
    type Err;

    async fn read_from<R: AsyncRead + Unpin + Send>(
        reader: &mut tokio::io::BufReader<R>,
    ) -> Result<Self, Self::Err>;
}

pub trait UciBufReadExt {
    type R: AsyncRead + Unpin + Send;

    fn buf_reader_mut(&mut self) -> &mut tokio::io::BufReader<Self::R>;
}

impl<R: AsyncRead + Unpin + Send> UciBufReadExt for tokio::io::BufReader<R> {
    type R = R;

    fn buf_reader_mut(&mut self) -> &mut tokio::io::BufReader<Self::R> {
        self
    }
}

#[async_trait]
pub trait UciBufReadExtAsync: UciBufReadExt {
    async fn with_next_line<F, O, E>(&mut self, f: F) -> Result<Option<O>, UciBufReadError<E>>
    where
        F: FnMut(&str) -> Result<Option<O>, UciBufReadError<E>> + Send + Unpin,
        Self: Sized,
    {
        let reader = self.buf_reader_mut();
        let handler = UciLinesHandler::new(reader, f);
        handler.await
    }
}

/// Blanket impl for any type implementing UciBufReadExt
#[async_trait]
impl<T: UciBufReadExt + Send> UciBufReadExtAsync for T {}

struct UciLinesHandler<'a, R, F, O, E>
where
    R: AsyncRead + Unpin + Send,
    F: FnMut(&str) -> Result<Option<O>, UciBufReadError<E>> + Send + Unpin,
{
    reader: &'a mut tokio::io::BufReader<R>,
    f: F,
}

impl<'a, R, F, O, E> UciLinesHandler<'a, R, F, O, E>
where
    R: AsyncRead + Unpin + Send,
    F: FnMut(&str) -> Result<Option<O>, UciBufReadError<E>> + Send + Unpin,
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

    let line = &buf[..eol_pos];

    // Do NOT consume buffer here — caller will decide after parsing
    core::task::Poll::Ready(Ok(line))
}

impl<'a, R, F, O, E> core::future::Future for UciLinesHandler<'a, R, F, O, E>
where
    R: tokio::io::AsyncRead + core::marker::Unpin + Send,
    F: FnMut(&str) -> Result<Option<O>, UciBufReadError<E>> + Send + core::marker::Unpin,
{
    type Output = Result<Option<O>, UciBufReadError<E>>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        use tokio::io::AsyncBufReadExt as _;

        let this = self.get_mut();

        // Scope the mutable borrow of the reader
        let line = {
            let reader_pin = core::pin::Pin::new(&mut *this.reader);
            match reader_next_line(reader_pin, cx) {
                core::task::Poll::Pending => return core::task::Poll::Pending,
                core::task::Poll::Ready(Err(e)) => {
                    return core::task::Poll::Ready(Err(UciBufReadError::IoError(e)));
                }
                core::task::Poll::Ready(Ok(l)) => l,
            }
        }; // <- reader_pin dropped here, borrow ends

        let res = (this.f)(line)?;
        let len_to_consume = line.len() + "\n".len();

        // Apply the parsing function
        match res {
            Some(o) => {
                // Parsing succeeded: advance the buffer by the slice + newline
                this.reader.consume(len_to_consume);
                core::task::Poll::Ready(Ok(Some(o)))
            }
            None => {
                // Parsing failed or incomplete: do not consume, allow retry
                core::task::Poll::Ready(Ok(None))
            }
        }
    }
}
