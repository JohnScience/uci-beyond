# Development approach

`uci-beyond` Rust workspace is developed for the purpose of supporting the development of <https://github.com/JohnScience/main-line>.

Hence, the completeness of features and correctness of implementations are driven by the needs of `main-line`.

Initially, it was intended to support UCI fully, because the needs of `main-line` in the end-game required nearly all UCI features and implementing UCI in one go would be more efficient. However, as development progressed, it became clear that implementing UCI fully would take a significant amount of time and effort, delaying the availability of most important features for `main-line`.

## The feature sets: MVP, end-game, and evolution

The `main-line` Stockfish client (GUI in UCI terms) is required to support at least two use cases:

* Evaluating a position using the provided parameters (e.g. depth)
* Comparing the chosen move against the engine's best move (also using provided parameters)

Ideally, the parameters should be reused if possible, to avoid redundant setups.

### Evaluating a position using the provided parameters (e.g. depth)

One way to do this would be to...

- Initiate UCI the mode with `uci` command

```
< Stockfish 17.1 by the Stockfish developers (see AUTHORS file)
> uci
...
```

- Store options received from the engine via `option` commands

> uci
< id name Stockfish 17.1
< id author the Stockfish developers (see AUTHORS file)
< 
< option name Debug Log File type string default <empty>
< option name NumaPolicy type string default auto
< option name Threads type spin default 1 min 1 max 1024
< option name Hash type spin default 16 min 1 max 33554432
< option name Clear Hash type button
< option name Ponder type check default false
< option name MultiPV type spin default 1 min 1 max 256
< option name Skill Level type spin default 20 min 0 max 20
< option name Move Overhead type spin default 10 min 0 max 5000
< option name nodestime type spin default 0 min 0 max 10000
< option name UCI_Chess960 type check default false
< option name UCI_LimitStrength type check default false
< option name UCI_Elo type spin default 1320 min 1320 max 3190
< option name UCI_ShowWDL type check default false
< option name SyzygyPath type string default <empty>
< option name SyzygyProbeDepth type spin default 1 min 1 max 100
< option name Syzygy50MoveRule type check default true
< option name SyzygyProbeLimit type spin default 7 min 0 max 7
< option name EvalFile type string default nn-1c0000000000.nnue
< option name EvalFileSmall type string default nn-37f18f62d772.nnue
< uciok
```

- Set the options via `setoption` commands (accounting for the options previously received)

```
> setoption name Threads value 4
< info string Using 4 threads
```

- Set up the position via the `position` command and use `go depth X` to start the search

```
> position startpos moves e2e4 e7e5 g1f3 b8c6
> go depth 20
< info string Available processors: 0-7
< info string Using 4 threads
< info string NNUE evaluation using nn-1c0000000000.nnue (133MiB, (22528, 3072, 15, 32, 1))
< info string NNUE evaluation using nn-37f18f62d772.nnue (6MiB, (22528, 128, 15, 32, 1))
< info depth 1 seldepth 4 multipv 1 score cp 12 nodes 161 nps 3425 hashfull 0 tbhits 0 time 47 pv f1c4
< info depth 2 seldepth 3 multipv 1 score cp 26 nodes 287 nps 5979 hashfull 0 tbhits 0 time 48 pv f1c4
< info depth 3 seldepth 4 multipv 1 score cp 33 nodes 436 nps 9083 hashfull 0 tbhits 0 time 48 pv f1c4
< info depth 4 seldepth 5 multipv 1 score cp 44 nodes 628 nps 12560 hashfull 0 tbhits 0 time 50 pv f1c4 a7a6
< info depth 5 seldepth 10 multipv 1 score cp 50 nodes 3727 nps 44903 hashfull 0 tbhits 0 time 83 pv b1c3 g8f6 f1b5
< info depth 6 seldepth 8 multipv 1 score cp 48 nodes 5549 nps 57802 hashfull 0 tbhits 0 time 96 pv b1c3 g8f6 d2d4 e5d4
< info depth 7 seldepth 8 multipv 1 score cp 50 nodes 6618 nps 65524 hashfull 0 tbhits 0 time 101 pv d2d4 e5d4 f3d4 f8c5
< info depth 8 seldepth 10 multipv 1 score cp 65 nodes 7173 nps 68971 hashfull 1 tbhits 0 time 104 pv d2d4 e5d4 f3d4
< info depth 9 seldepth 14 multipv 1 score cp 50 nodes 24809 nps 149451 hashfull 5 tbhits 0 time 166 pv d2d4 e5d4 f3d4 g8f6 d4c6 b7c6 e4e5
< info depth 10 seldepth 12 multipv 1 score cp 48 nodes 32649 nps 178409 hashfull 6 tbhits 0 time 183 pv d2d4 e5d4 f3d4 f8b4 c2c3 b4c5 c1e3 c5b6 f2f3 c6d4 c3d4
< info depth 11 seldepth 23 multipv 1 score cp 49 nodes 87864 nps 335358 hashfull 30 tbhits 0 time 262 pv f1b5 a7a6 b5a4 b7b5
< info depth 12 seldepth 19 multipv 1 score cp 45 nodes 136610 nps 419049 hashfull 49 tbhits 0 time 326 pv f1b5 a7a6 b5a4 g8f6 e1g1 f8e7 f1e1 b7b5 a4b3 d7d6 c2c3 c8g4
< info depth 13 seldepth 25 multipv 1 score cp 40 nodes 213526 nps 508395 hashfull 72 tbhits 0 time 420 pv f1b5 a7a6 b5a4 g8f6 e1g1 b7b5 a4b3 f6e4 f1e1 e4c5 d2d4 c5b3 a2b3 d7d6 d4e5 d6e5 f3e5 c6e5
< info depth 14 seldepth 26 multipv 1 score cp 39 nodes 442703 nps 678992 hashfull 137 tbhits 0 time 652 pv f1b5 a7a6 b5a4 g8f6 e1g1 f6e4 d2d4 b7b5 a4b3 d7d5 d4e5 c8e6 c2c3 f8e7
< info depth 15 seldepth 29 multipv 1 score cp 41 nodes 492183 nps 709197 hashfull 148 tbhits 0 time 694 pv f1b5 a7a6 b5a4 g8f6 e1g1 b7b5 a4b3 f6e4 d2d4 d7d5 d4e5 c8e6 c2c3 f8e7
< info depth 16 seldepth 28 multipv 1 score cp 35 nodes 593032 nps 756418 hashfull 177 tbhits 0 time 784 pv f1b5 a7a6 b5a4 g8f6 e1g1 f6e4 d2d4 b7b5 a4b3 d7d5 d4e5 c8e6 b1d2 e4c5 c2c3 c5b3 d2b3 f8e7 a2a4 e8g8 c1f4 d8d7 a4b5 a6b5
< info depth 17 seldepth 30 multipv 1 score cp 33 nodes 672287 nps 782639 hashfull 198 tbhits 0 time 859 pv f1b5 a7a6 b5a4 g8f6 e1g1 f6e4 d2d4 b7b5 a4b3 d7d5 d4e5 c8e6 c2c3 f8c5 b1d2 e8g8 b3c2 e6f5 a2a4 c5b6 a4b5 a6b5 a1a8 d8a8 d2b3 b5b4
< info depth 18 seldepth 30 multipv 1 score cp 30 nodes 1141559 nps 832646 hashfull 352 tbhits 0 time 1371 pv f1c4 f8c5 d2d4 c5d4 f3d4 c6d4 c1e3 d7d6 e3d4 e5d4 e1g1 d8f6 c2c3 d4c3 b1c3 g8e7
< info depth 19 seldepth 26 multipv 1 score cp 30 nodes 1295869 nps 854794 hashfull 400 tbhits 0 time 1516 pv f1b5 g8f6 e1g1 f6e4 f1e1 e4d6 f3e5 f8e7 b5f1 c6e5 e1e5 e8g8 d2d4 e7f6 e5e1 f8e8
< info depth 20 seldepth 31 multipv 1 score cp 31 nodes 1394550 nps 870505 hashfull 422 tbhits 0 time 1602 pv f1b5 g8f6 e1g1 f6e4 f1e1 e4d6 f3e5 c6e5 b5f1 f8e7 e1e5 e8g8 d2d4 e7f6 e5e1 d6f5 c2c3 f8e8 e1e8 d8e8 b1d2 d7d6 d2f3 a7a5 h2h3 a5a4
< bestmove f1b5 ponder g8f6
```

- Extract and display the best move from the `bestmove` response, the principal variation from the `pv` fields in the `info` responses, and the evaluation score from the `score` fields in the `info` responses.

Example output:

```
Best move: f1b5
Winning sequence: f1b5 a7a6 b5a4 g8f6 e1g1 f6e4 d2d4 b7b5 a4b3 d7d5 d4e5 c8e6 c2c3 f8e7
Evaluation score: +0.31cp for White at depth 20
```

### Comparing the chosen move against the engine's best move

One way to do this would be to...

- Evaluate the position after the chosen move using the engine
- Evaluate the position before the chosen move using the engine
- Compare the evaluation scores and the principal variations to assess the quality of the chosen move

At first, the different principal variation can be provided without detailed analysis. Later on, the difference could be explained using the positional understanding and tactical motifs.
