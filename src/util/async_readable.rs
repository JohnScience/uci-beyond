use async_trait::async_trait;

use crate::util::streaming_line_handler_ext::StreamingLineHandlerExt;

#[async_trait(?Send)]
pub trait AsyncReadable: Sized + Send {
    type Err: Send;

    async fn read_from<R>(reader: &mut R) -> Result<Self, R::HandlerError<Self::Err>>
    where
        R: StreamingLineHandlerExt<
                // Require a line handler whose output is exactly `Self`
                HandlerOutput<Self> = Self,
                // And whose error type matches our own error type
                HandlerError<Self::Err>: From<Self::Err>,
            >;
}
