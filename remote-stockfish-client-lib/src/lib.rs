use anyhow::Context;
use async_trait::async_trait;
use futures_util::stream::StreamExt as _;
use futures_util::stream::{SplitSink, SplitStream};
use tokio_tungstenite::connect_async;
use tungstenite::Utf8Bytes;
use tungstenite::protocol::Message;
use uci_beyond::gui_commands::UciCommandTrait;
use uci_beyond::util::{AsyncReadable, StringStreamReader};

pub struct RemoteChessEngine<R>
where
    R: tungstenite::client::IntoClientRequest + Unpin,
{
    request: R,
}

pub struct RemoteChessEngineConnection {
    write: SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        tokio_tungstenite::tungstenite::protocol::Message,
    >,
    read: SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
}

#[async_trait(?Send)]
impl uci_beyond::util::Connection for RemoteChessEngineConnection {
    type Err = anyhow::Error;

    async fn send<C>(
        &mut self,
        cmd: C,
    ) -> Result<Result<C::Response, <C::Response as AsyncReadable>::Err>, Self::Err>
    where
        C: UciCommandTrait,
    {
        use futures_util::SinkExt as _;

        let cmd: String = cmd.to_string();
        let cmd: Utf8Bytes = Utf8Bytes::try_from(cmd)?;
        self.write.send(Message::Text(cmd)).await?;

        let Self { read, write: _ } = self;

        let read = read.flat_map(|msg| {
            let opt = match msg {
                Ok(tungstenite::Message::Text(text)) => Some(Ok(text.to_string())),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            };
            futures::stream::iter(opt)
        });

        let mut reader = StringStreamReader::new(read);
        let response = C::Response::read_from(&mut reader)
            .await?
            .context("A command expected")?;

        match response {
            Ok(resp) => return Ok(Ok(resp)),
            Err(e) => return Ok(Err(e)),
        }
    }
}

impl<R> RemoteChessEngine<R>
where
    R: tungstenite::client::IntoClientRequest + Unpin,
{
    pub fn new(request: R) -> Self {
        Self { request }
    }

    /// Note: when connecting to stockfish, it greets you with
    /// `Stockfish 17.1 by the Stockfish developers (see AUTHORS file)\n`
    pub async fn connect(self) -> anyhow::Result<RemoteChessEngineConnection> {
        let (ws_stream, _) = connect_async(self.request).await?;
        let (write, read) = ws_stream.split();
        Ok(RemoteChessEngineConnection { write, read })
    }
}

impl RemoteChessEngineConnection {
    pub async fn next_message(&mut self) -> anyhow::Result<String> {
        let res = self.read.next().await;
        if let Some(Ok(msg)) = res {
            if let tungstenite::Message::Text(text) = msg {
                return Ok(text.to_string());
            }
        }
        Err(anyhow::anyhow!("No message received"))
    }

    pub async fn skip_message(&mut self) -> anyhow::Result<()> {
        if let Some(Ok(_msg)) = self.read.next().await {
            // println!("Skipped message: {:?}", msg);
        }
        Ok(())
    }

    // Ideally, this should be an async drop but Rust does not support that yet.
    pub async fn close_gracefully(&mut self) -> anyhow::Result<()> {
        use futures_util::SinkExt as _;
        use tungstenite::protocol::CloseFrame;
        use tungstenite::protocol::frame::coding::CloseCode;

        let reason = Utf8Bytes::try_from("Normal closure")?;

        self.write
            .send(Message::Close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason,
            })))
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures_util::SinkExt as _;
    use tokio::io::AsyncBufReadExt as _;

    use tokio::io::BufReader;
    use tokio_tungstenite::connect_async;
    use tungstenite::Message;

    // To run test, start stockfish websocket server:
    // websocat --text ws-l:127.0.0.1:8080 cmd:"C:\Program Files\stockfish\stockfish-windows-x86-64-avx2.exe"

    #[tokio::test]
    async fn tokio_tungstenite_cli_test() -> anyhow::Result<()> {
        // Connect to the Stockfish WebSocket server
        let (ws_stream, _) = connect_async("ws://127.0.0.1:8080").await?;
        println!("Connected to Stockfish server.");
        println!("EOL binary is {:?}", "\n".as_bytes());

        let (mut write, mut read) = ws_stream.split();

        // Task to forward stdin -> websocket
        let stdin_task = tokio::spawn(async move {
            let stdin = BufReader::new(tokio::io::stdin());
            let mut lines = stdin.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if write.send(Message::Text(line.into())).await.is_err() {
                    break;
                }
            }
        });

        // Task to forward websocket -> stdout
        let stdout_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = read.next().await {
                if let Message::Text(text) = msg {
                    print!("{text} = {:?}\n", text.as_bytes());
                }
            }
        });

        tokio::try_join!(stdin_task, stdout_task)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_uci_command() -> anyhow::Result<()> {
        use uci_beyond::gui_commands::UciCommand;
        use uci_beyond::util::Connection as _;

        let engine = RemoteChessEngine::new("ws://127.0.0.1:8080");
        let mut connection = engine.connect().await?;

        connection.skip_message().await?;

        let res = connection.send(UciCommand).await??;

        println!("Response to UCI command: {res:?}");

        connection.close_gracefully().await?;

        Ok(())
    }
}
