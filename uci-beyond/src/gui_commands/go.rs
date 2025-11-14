use std::fmt::Display;

use crate::{gui_commands::UciCommandTrait, model};

/// Start calculating on the current position set up with the position command.
/// There are a number of parameters that can follow this command and all will be sent in the same string.
///
/// See in Stockfish UCI documentation: <https://official-stockfish.github.io/docs/stockfish-wiki/UCI-&-Commands.html#go>.
///
/// # UCI listing examples
///
/// <details>
/// <summary>Example: go infinite </summary>
///
/// ```uci
/// > position startpos
/// > go infinite
/// info string NNUE evaluation using nn-ad9b42354671.nnue enabled
/// info depth 1 seldepth 1 multipv 1 score cp 18 nodes 20 nps 4000 hashfull 0 tbhits 0 time 5 pv e2e4
/// info depth 2 seldepth 2 multipv 1 score cp 46 nodes 66 nps 11000 hashfull 0 tbhits 0 time 6 pv d2d4
/// info depth 3 seldepth 2 multipv 1 score cp 51 nodes 120 nps 20000 hashfull 0 tbhits 0 time 6 pv e2e4
/// info depth 4 seldepth 2 multipv 1 score cp 58 nodes 144 nps 18000 hashfull 0 tbhits 0 time 8 pv d2d4
/// info depth 5 seldepth 2 multipv 1 score cp 58 nodes 174 nps 15818 hashfull 0 tbhits 0 time 11 pv d2d4 a7a6
/// info depth 6 seldepth 7 multipv 1 score cp 34 nodes 1303 nps 81437 hashfull 0 tbhits 0 time 16 pv e2e4 c7c5 g1f3 b8c6 c2c3
/// info depth 7 seldepth 6 multipv 1 score cp 29 nodes 3126 nps 120230 hashfull 1 tbhits 0 time 26 pv d2d4 g8f6 e2e3 d7d5 c2c4 d5c4
/// info depth 8 seldepth 7 multipv 1 score cp 26 nodes 5791 nps 152394 hashfull 4 tbhits 0 time 38 pv g1f3 g8f6 d2d4 d7d5 e2e3
/// info depth 9 seldepth 9 multipv 1 score cp 31 nodes 8541 nps 174306 hashfull 5 tbhits 0 time 49 pv g1f3 c7c5 e2e4 e7e6 d2d4 c5d4 f3d4
/// info depth 10 seldepth 13 multipv 1 score cp 25 nodes 20978 nps 209780 hashfull 10 tbhits 0 time 100 pv e2e4 c7c5 g1f3 b8c6 f1c4 e7e6 e1g1 g8f6
/// info depth 11 seldepth 13 multipv 1 score cp 32 nodes 29040 nps 220000 hashfull 14 tbhits 0 time 132 pv e2e4 c7c5 c2c3 g8f6 e4e5 f6d5 d2d4
/// info depth 12 seldepth 14 multipv 1 score cp 38 nodes 41207 nps 242394 hashfull 18 tbhits 0 time 170 pv e2e4 e7e6 d2d4 d7d5 b1c3 d5e4 c3e4
/// > stop
/// info depth 13 seldepth 14 multipv 1 score cp 38 nodes 45531 nps 247451 hashfull 21 tbhits 0 time 184 pv e2e4 e7e6 d2d4 d7d5 b1c3 d5e4 c3e4
/// bestmove e2e4 ponder e7e6
/// ```
///
/// </details>
pub struct GoCommand {
    /// Restrict search to these moves only.
    /// Example: After `position startpos` and `go infinite searchmoves e2e4 d2d4` the engine will only search the two moves e2e4 and d2d4 in the initial position.
    pub searchmoves: Vec<model::MoveString>,
    /// Start searching in pondering mode. It won't exit the search in ponder mode, even if it's mate!
    /// This means that the last move sent in in the position string is the ponder move.
    /// The engine can do what it wants to do, but after a `ponderhit` command it will execute the suggested move to ponder on.
    /// This means that the ponder move sent by the GUI can be interpreted as a recommendation about which move to ponder.
    /// However, if the engine decides to ponder on a different move, it won't display any mainlines as they are likely to be misinterpreted by the GUI because the GUI expects the engine to ponder on the suggested move.
    pub ponder: bool,
    /// Tell the engine that White has x ms left on the clock.
    pub wtime: Option<u32>,
    /// Tell the engine that Black has x ms left on the clock.
    pub btime: Option<u32>,
    /// Tell the engine that White's increment per move in ms if x > 0.
    pub winc: Option<u32>,
    /// Tell the engine that Black's increment per move in ms if x > 0.
    pub binc: Option<u32>,
    /// Tell the engine that there are x moves to the next time control
    /// Note: this will only be sent if x > 0, if you don't get this and get the wtime and btime it's sudden death.
    pub movestogo: Option<u32>,
    /// Stop the search when depth x has been reached.
    pub depth: Option<u32>,
    /// Stop the search when approximately x number of nodes have been reached.
    pub nodes: Option<u32>,
    /// Stop the search when/if a mate in x or less moves is found.
    /// It will stop if the side to move is mating and since Stockfish 17 when getting mated too.
    pub mate: Option<u32>,
    /// Stop the search when approximately x ms have passed.
    pub movetime: Option<u32>,
    /// Search until the `stop` command is given. Stockfish won't exit the search without being told so in this mode!
    pub indefinite: bool,
    /// A debugging function to walk the move generation tree of strictly legal moves to count all the leaf nodes of a certain depth.
    pub perft: Option<u32>,
}

impl Default for GoCommand {
    fn default() -> Self {
        Self {
            searchmoves: vec![],
            ponder: false,
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
            movestogo: None,
            depth: None,
            nodes: None,
            mate: None,
            movetime: None,
            indefinite: false,
            perft: None,
        }
    }
}

impl Display for GoCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "go")?;
        if !self.searchmoves.is_empty() {
            write!(f, " searchmoves")?;
            for m in &self.searchmoves {
                write!(f, " {m}")?;
            }
        }
        if self.ponder {
            write!(f, " ponder")?;
        }
        if let Some(wtime) = self.wtime {
            write!(f, " wtime {wtime}")?;
        }
        if let Some(btime) = self.btime {
            write!(f, " btime {btime}")?;
        }
        if let Some(winc) = self.winc {
            write!(f, " winc {winc}")?;
        }
        if let Some(binc) = self.binc {
            write!(f, " binc {binc}")?;
        }
        if let Some(movestogo) = self.movestogo {
            write!(f, " movestogo {movestogo}")?;
        }
        if let Some(depth) = self.depth {
            write!(f, " depth {depth}")?;
        }
        if let Some(nodes) = self.nodes {
            write!(f, " nodes {nodes}")?;
        }
        if let Some(mate) = self.mate {
            write!(f, " mate {mate}")?;
        }
        if let Some(movetime) = self.movetime {
            write!(f, " movetime {movetime}")?;
        }
        if self.indefinite {
            write!(f, " infinite")?;
        }
        if let Some(perft) = self.perft {
            write!(f, " perft {perft}")?;
        }
        Ok(())
    }
}

impl UciCommandTrait for GoCommand {
    type Response = ();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_idefinite() {
        let cmd = GoCommand {
            indefinite: true,
            ..Default::default()
        };
        assert_eq!(cmd.to_string(), "go infinite");
    }

    #[test]
    fn test_go_depth() {
        let cmd = GoCommand {
            depth: Some(25),
            ..Default::default()
        };
        assert_eq!(cmd.to_string(), "go depth 25");
    }

    #[test]
    fn test_go_nodes() {
        let cmd = GoCommand {
            nodes: Some(100_000),
            ..Default::default()
        };
        assert_eq!(cmd.to_string(), "go nodes 100000");
    }

    #[test]
    fn test_go_mate() {
        let cmd = GoCommand {
            mate: Some(1),
            ..Default::default()
        };
        assert_eq!(cmd.to_string(), "go mate 1");
    }

    #[test]
    fn test_go_movetime() {
        let cmd = GoCommand {
            movetime: Some(1000),
            ..Default::default()
        };
        assert_eq!(cmd.to_string(), "go movetime 1000");
    }

    #[test]
    fn test_go_ponder_movetime() {
        let cmd = GoCommand {
            ponder: true,
            movetime: Some(1000),
            ..Default::default()
        };
        assert_eq!(cmd.to_string(), "go ponder movetime 1000");
    }
}
