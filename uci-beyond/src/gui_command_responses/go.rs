use crate::engine_commands::{
    AvailableProcessorsInfoCommand, NnueEvaluationInfoCommand, UsingThreadsInfoCommand,
};

// Ideally, GoCommandResponse should be an enum to support different implementations.
// However, implementing multiple go command response types is not a priority right now.
// Therefore, we use a type alias for the basic implementation.
pub type GoCommandResponse = BasicGoCommandResponse;

/// The temporary basic implementation of a go command response.
///
/// It is deliberately incomplete to cover only the parts of the response
/// that are currently needed.
///
/// Eventually, this should be replaced with a more complete implementation
/// that can handle all aspects of a go command response.
pub struct BasicGoCommandResponse {
    info_string_block: InfoStringBlock,
}

/// "Info string" block of a go command response.
///
/// ```text
/// info string Available processors: 0-7
// info string Using 1 thread
// info string NNUE evaluation using nn-1c0000000000.nnue (133MiB, (22528, 3072, 15, 32, 1))
// info string NNUE evaluation using nn-37f18f62d772.nnue (6MiB, (22528, 128, 15, 32, 1))
// info depth 1 seldepth 2 multipv 1 score cp 17 nodes 20 nps 6666 hashfull 0 tbhits 0 time 3 pv e2e4
/// ```
///
/// in
///
/// ```text
/// go depth 5
/// info string Available processors: 0-7
/// info string Using 1 thread
/// info string NNUE evaluation using nn-1c0000000000.nnue (133MiB, (22528, 3072, 15, 32, 1))
/// info string NNUE evaluation using nn-37f18f62d772.nnue (6MiB, (22528, 128, 15, 32, 1))
/// info depth 1 seldepth 2 multipv 1 score cp 17 nodes 20 nps 6666 hashfull 0 tbhits 0 time 3 pv e2e4
/// info depth 2 seldepth 3 multipv 1 score cp 34 nodes 45 nps 11250 hashfull 0 tbhits 0 time 4 pv e2e4
/// info depth 3 seldepth 4 multipv 1 score cp 42 nodes 72 nps 14400 hashfull 0 tbhits 0 time 5 pv e2e4
/// info depth 4 seldepth 7 multipv 1 score cp 39 nodes 512 nps 85333 hashfull 0 tbhits 0 time 6 pv g1f3 d7d5 d2d4
/// info depth 5 seldepth 7 multipv 1 score cp 58 nodes 609 nps 87000 hashfull 0 tbhits 0 time 7 pv e2e4
/// bestmove e2e4 ponder d7d6
/// ```
pub struct InfoStringBlock {
    available_processors: AvailableProcessorsInfoCommand,
    used_threads: UsingThreadsInfoCommand,
    nnue_evaluations: Vec<NnueEvaluationInfoCommand>,
}
