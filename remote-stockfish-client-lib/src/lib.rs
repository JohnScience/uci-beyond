use anyhow::Context;
use async_trait::async_trait;
use futures_util::stream::StreamExt as _;
use futures_util::stream::{SplitSink, SplitStream};
use tokio_tungstenite::connect_async;
use tungstenite::Utf8Bytes;
use tungstenite::protocol::Message;
use uci_beyond::gui_commands::UciCommandTrait;
use uci_beyond::model::MoveString;
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

pub enum PositionEvaluation {
    Undecided {
        // best move is the first move in principal_variation
        principal_variation: Vec<MoveString>,
        score_cp: i32,
    },
    Mate {
        // best move is the first move in principal_variation
        principal_variation: Vec<MoveString>,
        mate_in_moves: u32,
    },
}

pub enum MoveEvaluation {
    Best,
    Subpar {
        /// The loss in position evaluation caused by this move, measured in centipawns.
        /// A positive value indicates the move worsens the position (from the current player's perspective).
        /// Calculated as: (score before move) - (score after move).
        /// For example, if score_delta_cp = 132, the move loses 132 centipawns (1.32 pawns).
        score_delta_cp: i32,
        principal_variation: Vec<MoveString>,
    },
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
        C::Response: AsyncReadable,
    {
        use futures_util::SinkExt as _;

        let cmd: String = cmd.to_string();
        let cmd: Utf8Bytes = Utf8Bytes::try_from(cmd)?;
        self.write.send(Message::Text(cmd)).await?;

        let Self { read, write: _ } = self;

        let read = read.flat_map(|msg| {
            match msg {
                Ok(tungstenite::Message::Text(text)) => {
                    eprintln!("=== WebSocket Text Message Received ===");
                    eprintln!("Length: {} bytes", text.len());
                    eprintln!("Content: {:?}", text);
                    eprintln!("======================================");

                    // Split text by newlines to handle multiple UCI lines in one WebSocket message
                    // This handles both \n (Linux) and \r\n (Windows) line endings
                    // Also trim trailing whitespace to handle trailing spaces in UCI option lines
                    let mut lines: Vec<String> = text
                        .split('\n')
                        .map(|line| line.trim_end().to_string())
                        .collect();

                    // Remove the last empty element if text ends with \n
                    // (split always creates a trailing empty string after a final separator)
                    if text.ends_with('\n') && lines.last().map_or(false, |s| s.is_empty()) {
                        lines.pop();
                    }

                    // Filter out command echoes from pseudo-TTY (script command echoes input)
                    // UCI commands we send: uci, isready, position, go, stop, quit, setoption, ucinewgame
                    lines.retain(|line| {
                        let trimmed = line.trim();
                        eprintln!("DEBUG: Checking line: {:?} (trimmed: {:?})", line, trimmed);
                        // Skip empty lines that would be command echoes or actual UCI commands
                        // Valid UCI responses start with: id, option, uciok, readyok, bestmove, info
                        if trimmed.is_empty() {
                            eprintln!("DEBUG: Keeping empty line");
                            true // Keep empty lines (they're structural in UCI protocol)
                        } else if trimmed == "uci"
                            || trimmed == "isready"
                            || trimmed == "quit"
                            || trimmed == "stop"
                            || trimmed == "ucinewgame"
                            || trimmed.starts_with("position ")
                            || trimmed.starts_with("go ")
                            || trimmed.starts_with("setoption ")
                        {
                            eprintln!("Filtering out command echo: {:?}", trimmed);
                            false // Filter out command echoes
                        } else {
                            eprintln!("DEBUG: Keeping response line");
                            true // Keep actual UCI responses
                        }
                    });

                    eprintln!("Split into {} lines: {:?}", lines.len(), lines);

                    let results: Vec<Result<String, tungstenite::Error>> =
                        lines.into_iter().map(Ok).collect();
                    futures::stream::iter(results)
                }
                Ok(other_msg) => {
                    eprintln!("=== WebSocket Non-Text Message: {:?} ===", other_msg);
                    futures::stream::iter(vec![])
                }
                Err(e) => {
                    eprintln!("=== WebSocket Error: {:?} ===", e);
                    futures::stream::iter(vec![Err(e)])
                }
            }
        });

        eprintln!("=== Starting to parse response ===");
        let mut reader = StringStreamReader::new(read);
        let response = C::Response::read_from(&mut reader)
            .await?
            .context("A command expected")?;
        eprintln!("=== Finished parsing response ===");

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

    async fn evaluate_position_inner(
        &mut self,
        fen: uci_beyond::model::FenString,
        moves: Vec<MoveString>,
    ) -> anyhow::Result<PositionEvaluation> {
        use uci_beyond::gui_commands::{GoCommand, PositionCommand, UciCommand};
        use uci_beyond::util::Connection as _;

        // Send UCI command
        let _ = self.send(UciCommand).await??;

        // Set position
        let mut position_cmd = PositionCommand::from_fen(fen);
        position_cmd.moves = moves;
        let _ = self.send(position_cmd).await??;

        // Start search with depth 20
        let go_cmd = GoCommand {
            depth: Some(20),
            ..Default::default()
        };
        let _ = self.send(go_cmd).await??;

        // Parse evaluation from messages
        let mut last_evaluation: Option<PositionEvaluation> = None;

        loop {
            let msg = self.next_message().await?;
            if msg.starts_with("info") && msg.contains("score") {
                // Example: info depth 20 seldepth 32 multipv 1 score cp 34 nodes 123456 nps 123456 tbhits 0 time 123 pv e2e4 e7e5 g1f3
                let parts: Vec<&str> = msg.split_whitespace().collect();
                if let Some(score_index) = parts.iter().position(|&s| s == "score") {
                    if score_index + 2 < parts.len() {
                        let score_type = parts[score_index + 1];
                        if score_type == "cp" {
                            if let Ok(score_cp) = parts[score_index + 2].parse::<i32>() {
                                if let Some(pv_index) = parts.iter().position(|&s| s == "pv") {
                                    let principal_variation: Vec<MoveString> = parts
                                        [pv_index + 1..]
                                        .iter()
                                        .map(|&s| MoveString(s.to_string()))
                                        .collect();
                                    last_evaluation = Some(PositionEvaluation::Undecided {
                                        principal_variation,
                                        score_cp,
                                    });
                                }
                            }
                        } else if score_type == "mate" {
                            if let Ok(mate_in_moves) = parts[score_index + 2].parse::<u32>() {
                                if let Some(pv_index) = parts.iter().position(|&s| s == "pv") {
                                    let principal_variation: Vec<MoveString> = parts
                                        [pv_index + 1..]
                                        .iter()
                                        .map(|&s| MoveString(s.to_string()))
                                        .collect();
                                    last_evaluation = Some(PositionEvaluation::Mate {
                                        principal_variation,
                                        mate_in_moves,
                                    });
                                }
                            }
                        }
                    }
                }
            } else if msg.starts_with("bestmove") {
                if let Some(eval) = last_evaluation {
                    return Ok(eval);
                } else {
                    return Err(anyhow::anyhow!(
                        "Did not receive evaluation info before bestmove"
                    ));
                }
            }
        }
    }

    pub async fn evaluate_position(
        &mut self,
        fen: uci_beyond::model::FenString,
    ) -> anyhow::Result<PositionEvaluation> {
        self.evaluate_position_inner(fen, Vec::new()).await
    }

    pub async fn evaluate_move(
        &mut self,
        fen: uci_beyond::model::FenString,
        mv: MoveString,
    ) -> anyhow::Result<MoveEvaluation> {
        let moves = vec![mv];
        // First, evaluate the position without the move to get the best move
        let eval_before = self.evaluate_position(fen.clone()).await?;
        let eval_after = self.evaluate_position_inner(fen, moves).await?;

        match (eval_before, eval_after) {
            (
                PositionEvaluation::Undecided {
                    principal_variation: _,
                    score_cp: score_before,
                },
                PositionEvaluation::Undecided {
                    principal_variation,
                    score_cp: score_after,
                },
            ) => {
                if score_after == score_before {
                    Ok(MoveEvaluation::Best)
                } else {
                    Ok(MoveEvaluation::Subpar {
                        score_delta_cp: score_before - score_after,
                        principal_variation,
                    })
                }
            }
            (
                PositionEvaluation::Mate {
                    principal_variation: _,
                    mate_in_moves: _,
                },
                PositionEvaluation::Mate {
                    principal_variation: _,
                    mate_in_moves: _,
                },
            ) => Ok(MoveEvaluation::Best),
            _ => Err(anyhow::anyhow!(
                "Incompatible evaluation types for move evaluation"
            )),
        }
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
        let (ws_stream, _) = connect_async("ws://127.0.0.1:9002").await?;
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

        let engine = RemoteChessEngine::new("ws://127.0.0.1:9002");
        let mut connection = engine.connect().await?;

        connection.skip_message().await?;

        let res = connection.send(UciCommand).await??;

        println!("Response to UCI command: {res:?}");

        connection.close_gracefully().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_best_by_test() -> anyhow::Result<()> {
        use uci_beyond::gui_commands::{GoCommand, UciCommand};
        use uci_beyond::util::Connection as _;

        let engine = RemoteChessEngine::new("ws://127.0.0.1:8080");
        let mut connection = engine.connect().await?;

        connection.skip_message().await?;

        let res = connection.send(UciCommand).await??;

        println!("Response to UCI command: {res:?}");

        let go_cmd = GoCommand {
            depth: Some(5),
            ..Default::default()
        };

        let _res = connection.send(go_cmd).await??;

        let msg = connection.next_message().await?;

        println!("Msg: {msg}");

        connection.close_gracefully().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_evaluate_position() -> anyhow::Result<()> {
        let engine = RemoteChessEngine::new("ws://127.0.0.1:8080");
        let mut connection = engine.connect().await?;
        connection.skip_message().await?;
        let fen = uci_beyond::model::FenString("6k1/5ppp/8/8/8/6Q1/5PPP/6K1 w - - 0 1".to_string());
        let eval = connection.evaluate_position(fen).await?;

        match eval {
            PositionEvaluation::Undecided {
                principal_variation,
                score_cp,
            } => {
                println!("Position is undecided with score {score_cp} centipawns.");
                println!("Principal Variation: [");
                for mv in principal_variation {
                    println!("  {},", mv.0);
                }
                println!("]");
            }
            PositionEvaluation::Mate {
                principal_variation,
                mate_in_moves,
            } => {
                println!("Position is a mate in {mate_in_moves} moves.");
                println!("Principal Variation: [");
                for mv in principal_variation {
                    println!("  {},", mv.0);
                }
                println!("]");
            }
        }

        connection.close_gracefully().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_evaluate_move() -> anyhow::Result<()> {
        let engine = RemoteChessEngine::new("ws://127.0.0.1:9002");
        let mut connection = engine.connect().await?;
        connection.skip_message().await?;
        let fen = uci_beyond::model::FenString(
            "r1bqkbnr/pppppppp/n7/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
        );
        let mv = MoveString("e2e4".to_string());
        let eval = connection.evaluate_move(fen, mv).await?;
        match eval {
            MoveEvaluation::Best => {
                println!("The move is the best move.");
            }
            MoveEvaluation::Subpar {
                score_delta_cp,
                principal_variation,
            } => {
                println!(
                    "The move is subpar by {} centipawns. New PV:",
                    score_delta_cp
                );
                for mv in principal_variation {
                    println!("  {},", mv.0);
                }
            }
        }

        connection.close_gracefully().await?;
        Ok(())
    }
}
