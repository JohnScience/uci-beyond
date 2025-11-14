use async_trait::async_trait;

use crate::{gui_commands::UciCommandTrait, util::AsyncReadable};

#[async_trait(?Send)]
pub trait Connection {
    type Err: std::fmt::Debug + Send;

    async fn send<C>(
        &mut self,
        cmd: C,
    ) -> Result<Result<C::Response, <C::Response as AsyncReadable>::Err>, Self::Err>
    where
        C: UciCommandTrait;
}
