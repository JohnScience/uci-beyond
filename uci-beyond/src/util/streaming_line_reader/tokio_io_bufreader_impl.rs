use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::util::StreamingLineReader;

impl<R> StreamingLineReader for tokio::io::BufReader<R>
where
    R: tokio::io::AsyncRead + Unpin + Send,
{
    const AUTO_CONSUMING: bool = false;

    type Error = std::io::Error;

    type Line<'a>
        = &'a str
    where
        Self: 'a;

    fn next_line<'a>(
        self: &'a mut Self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<Self::Line<'a>>, Self::Error>> {
        let pinned = Pin::new(self);
        match reader_next_line(pinned, cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Ready(Ok(line)) => {
                if line.is_empty() {
                    // EOF
                    Poll::Ready(Ok(None))
                } else {
                    Poll::Ready(Ok(Some(line)))
                }
            }
        }
    }

    fn consume_line_manually(&mut self, line_len: usize) {
        <Self as tokio::io::AsyncBufReadExt>::consume(self, line_len);
    }
}

fn str_prefix(bytes: &[u8]) -> &str {
    let n = match std::str::from_utf8(bytes) {
        Ok(s) => return s,
        Err(e) => e.valid_up_to(),
    };

    std::str::from_utf8(&bytes[..n]).unwrap()
}

fn reader_next_line<'a, R: tokio::io::AsyncRead + Unpin + Send>(
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

#[cfg(test)]
mod tests {
    use super::*;
    use core::task::Poll;
    use std::io::Cursor;
    use tokio::io::BufReader;

    use std::future::poll_fn;

    #[tokio::test]
    async fn reads_multiple_lines_correctly() {
        let input = b"line1\nline2\nline3\n";
        let cursor = Cursor::new(input);
        let mut reader = BufReader::new(cursor);

        let mut lines = Vec::new();

        loop {
            let res = poll_fn(|cx| match StreamingLineReader::next_line(&mut reader, cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                Poll::Ready(Ok(Some(line))) => Poll::Ready(Ok(Some(line.to_string()))),
                Poll::Ready(Ok(None)) => Poll::Ready(Ok(None)),
            })
            .await;

            match res {
                Ok(Some(line)) => {
                    let line: &str = line.as_ref();
                    // Simulate consumer consuming the buffer
                    let len = line.len();
                    reader.consume_line_manually(len);
                    lines.push(line.trim_end().to_string());
                }
                Ok(None) => break,
                Err(e) => panic!("Unexpected I/O error: {e}"),
            }
        }

        assert_eq!(lines, vec!["line1", "line2", "line3"]);
    }
}
