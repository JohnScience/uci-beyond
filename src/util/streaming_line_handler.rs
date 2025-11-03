pub trait StreamingLineHandler: Unpin {
    type Error;
    type Output;

    type Reader: Unpin;
    type Line<'a>: AsRef<str>
    where
        Self: 'a;
    type NextLineError: Into<Self::Error>;

    type FnOut;
    type F: FnMut(&str) -> Self::FnOut;

    fn next_line<'a>(
        reader: std::pin::Pin<&'a mut Self::Reader>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<Option<Self::Line<'a>>, Self::NextLineError>>
    where
        Self: 'a;

    fn split_into_parts(&mut self) -> (&mut Self::Reader, &mut Self::F);

    type FinalizeError: Into<Self::Error>;

    fn finalize(
        reader: &mut Self::Reader,
        line_len: usize,
        o: Self::FnOut,
    ) -> Result<Self::Output, Self::FinalizeError>;

    fn handle(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<Option<Self::Output>, Self::Error>> {
        let this = self.get_mut();
        let (reader, f) = this.split_into_parts();

        let reader_pinned = core::pin::Pin::new(&mut *reader);

        let (o, line_len): (Self::FnOut, usize) = {
            let line: Self::Line<'_> = match Self::next_line(reader_pinned, cx) {
                core::task::Poll::Pending => return core::task::Poll::Pending,
                core::task::Poll::Ready(Err(e)) => return core::task::Poll::Ready(Err(e.into())),
                core::task::Poll::Ready(Ok(Some(line))) => line,
                core::task::Poll::Ready(Ok(None)) => return core::task::Poll::Ready(Ok(None)),
            };

            let line_str: &str = line.as_ref();

            let line_len: usize = line_str.len();

            let o: Self::FnOut = (f)(line_str);

            (o, line_len)
        };

        match Self::finalize(reader, line_len, o) {
            Err(e) => core::task::Poll::Ready(Err(e.into())),
            Ok(o) => core::task::Poll::Ready(Ok(Some(o))),
        }
    }
}
