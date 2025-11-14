use async_trait::async_trait;

use crate::util::StreamingLineReader;

#[async_trait(?Send)]
pub trait AsyncReadable: Sized + Send {
    type Err: Send + std::fmt::Debug;

    async fn read_from<R>(reader: &mut R) -> Result<Option<Result<Self, Self::Err>>, R::Error>
    where
        R: StreamingLineReader;
}

#[async_trait(?Send)]
impl AsyncReadable for () {
    type Err = std::convert::Infallible;

    async fn read_from<R>(_: &mut R) -> Result<Option<Result<Self, Self::Err>>, R::Error>
    where
        R: StreamingLineReader,
    {
        Ok(Some(Ok(())))
    }
}
