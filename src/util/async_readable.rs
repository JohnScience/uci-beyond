use async_trait::async_trait;
use tokio::io::AsyncRead;

#[async_trait]
pub trait AsyncReadable: Sized {
    type Err;

    async fn read_from<R: AsyncRead + Unpin + Send>(
        reader: &mut tokio::io::BufReader<R>,
    ) -> Result<Self, Self::Err>;
}
