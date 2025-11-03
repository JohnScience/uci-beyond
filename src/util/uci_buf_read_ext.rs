use async_trait::async_trait;
use tokio::io::AsyncRead;

use crate::util::{UciBufReadError, UciLinesHandler};

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
        F: FnMut(&str) -> Result<O, UciBufReadError<E>> + Send + Unpin,
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
