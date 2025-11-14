use futures::stream::Stream;

use crate::util::StreamingLineReader;

pub struct StringStreamReader<E, S>
where
    S: Stream<Item = Result<String, E>> + Unpin + Send,
    E: Send,
{
    stream: S,
}

impl<E, S> StringStreamReader<E, S>
where
    S: Stream<Item = Result<String, E>> + Unpin + Send,
    E: Send,
{
    pub fn new(stream: S) -> Self {
        Self { stream }
    }
}

impl<E, S> StreamingLineReader for StringStreamReader<E, S>
where
    S: Stream<Item = Result<String, E>> + Unpin + Send,
    E: Send,
{
    type Error = E;

    const AUTO_CONSUMING: bool = true;

    type Line<'a>
        = String
    where
        Self: 'a;

    fn next_line<'a>(
        self: &'a mut Self,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<Option<Self::Line<'a>>, Self::Error>> {
        use futures::TryStreamExt as _;

        match futures::ready!(self.stream.try_poll_next_unpin(cx)) {
            Some(Ok(line)) => core::task::Poll::Ready(Ok(Some(line))),
            Some(Err(e)) => core::task::Poll::Ready(Err(e)),
            None => core::task::Poll::Ready(Ok(None)),
        }
    }

    fn consume_line_manually(&mut self, _line_len: usize) {
        // No-op since we are auto-consuming
    }
}
