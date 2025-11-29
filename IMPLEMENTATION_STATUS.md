# UCI-Beyond Implementation Status

This document tracks the implementation status of the `uci-beyond` crate, which provides a Rust framework for implementing UCI (Universal Chess Interface) compatible chess engine clients.

## Overview

The crate provides types and utilities for:
- GUI commands (commands sent by the GUI to the engine)
- Engine commands (commands sent by the engine to the GUI)
- GUI command responses (structured responses from the engine)
- UCI options and configurations
- Common model types (positions, moves, etc.)

## GUI Commands (GUI → Engine)

Commands that the GUI sends to the chess engine:

- [x] `uci` - Tell engine to use the UCI protocol  
  **Path**: `uci_beyond::gui_commands::UciCommand`
- [x] `isready` - Ping the engine for readiness  
  **Path**: `uci_beyond::gui_commands::IsReadyCommand`
- [x] `ucinewgame` - Tell engine a new game is starting  
  **Path**: `uci_beyond::gui_commands::UciNewGameCommand`
- [x] `position` - Set up the position on the board  
  **Path**: `uci_beyond::gui_commands::PositionCommand`
- [x] `go` - Start calculating on the current position  
  **Path**: `uci_beyond::gui_commands::GoCommand`
- [x] `stop` - Stop calculating as soon as possible  
  **Path**: `uci_beyond::gui_commands::StopCommand`
- [x] `quit` - Quit the program as soon as possible  
  **Path**: `uci_beyond::gui_commands::QuitCommand`
- [x] `setoption` - Set internal engine parameters  
  **Path**: `uci_beyond::gui_commands::SetOptionCommand`
- [ ] `ponderhit` - Tell engine the user made the expected move
- [ ] `debug` - Toggle debug mode on/off
- [ ] `register` - Registration for copy protection

### SetOption Command Coverage

- [x] `Threads` - Number of CPU threads
- [x] `Hash` - Memory size for hash tables
- [x] `MultiPV` - Number of principal variations
- [x] `NumaPolicy` - NUMA policy configuration
- [x] `Clear Hash` - Clear hash tables
- [x] `Ponder` - Enable/disable pondering
- [x] `EvalFile` - NNUE evaluation file
- [x] `EvalFileSmall` - Small NNUE evaluation file
- [x] `UCI_Chess960` - Enable Chess960 mode
- [x] `UCI_ShowWDL` - Show Win/Draw/Loss statistics
- [x] `UCI_LimitStrength` - Enable strength limiting
- [x] `UCI_Elo` - Set playing strength in Elo
- [x] `SkillLevel` - Set skill level (0-20)
- [x] `SyzygyPath` - Path to Syzygy tablebases
- [x] `SyzygyProbeDepth` - Minimum depth to probe
- [x] `Syzygy50MoveRule` - Enable 50-move rule
- [x] `SyzygyProbeLimit` - Maximum pieces for probing
- [x] `MoveOverhead` - Move overhead in milliseconds
- [x] `Nodestime` - Nodes to search per millisecond
- [x] `DebugLogFile` - Debug log file path

### Go Command Parameters

- [x] `searchmoves` - Restrict search to specific moves
- [x] `ponder` - Start in pondering mode
- [x] `wtime` - White's remaining time
- [x] `btime` - Black's remaining time
- [x] `winc` - White's increment per move
- [x] `binc` - Black's increment per move
- [x] `movestogo` - Moves until next time control
- [x] `depth` - Search to specific depth
- [x] `nodes` - Search specific number of nodes
- [x] `mate` - Search for mate in X moves
- [x] `movetime` - Search for exact time
- [x] `infinite` - Search indefinitely

## Engine Commands (Engine → GUI)

Commands that the chess engine sends to the GUI:

### ID Commands
- [x] `id name` - Engine name  
  **Path**: `uci_beyond::engine_commands::IdCommand`
- [x] `id author` - Engine author(s)  
  **Path**: `uci_beyond::engine_commands::IdCommand`
- [x] ID block parsing (complete id section)  
  **Path**: `uci_beyond::engine_commands::IdBlock`

### Option Commands
- [x] `option` - Define engine options  
  **Path**: `uci_beyond::engine_commands::OptionCommand`
- [x] Option block parsing (all options)  
  **Path**: `uci_beyond::engine_commands::UciOptionBlock`
- [x] Option types: `check`, `spin`, `combo`, `button`, `string`  
  **Path**: `uci_beyond::options::UciOptionType`
- [ ] Combo option parsing (marked as `todo!()`)

### Info Commands
- [x] `info` command structure  
  **Path**: `uci_beyond::engine_commands::info::InfoCommand`
- [ ] `info depth` - Search depth info
- [ ] `info seldepth` - Selective search depth
- [ ] `info time` - Search time in ms
- [ ] `info nodes` - Nodes searched
- [ ] `info pv` - Principal variation
- [ ] `info multipv` - Multi-PV line number
- [ ] `info score` - Position score (cp/mate)
- [ ] `info currmove` - Currently searching move
- [ ] `info currmovenumber` - Current move number
- [ ] `info hashfull` - Hash table fullness
- [ ] `info nps` - Nodes per second
- [ ] `info tbhits` - Tablebase hits
- [ ] `info sbhits` - Shredder tablebase hits
- [ ] `info cpuload` - CPU load
- [ ] `info refutation` - Refutation moves
- [ ] `info currline` - Current line being searched

### Info String Commands
- [x] `info string` - Arbitrary string output  
  **Path**: `uci_beyond::engine_commands::info::StringInfoCommand`
- [x] `info string Available processors` - Processor info (with Display/FromStr)  
  **Path**: `uci_beyond::engine_commands::AvailableProcessorsInfoCommand`
- [x] `info string Using X thread(s)` - Thread usage info (struct defined)  
  **Path**: `uci_beyond::engine_commands::UsingThreadsInfoCommand`
- [x] `info string NNUE evaluation` - NNUE network info (with Display/FromStr)  
  **Path**: `uci_beyond::engine_commands::NnueEvaluationInfoCommand`
  - [x] Network name parsing
  - [x] Network size parsing
  - [x] Network architecture parsing  
    **Path**: `uci_beyond::engine_commands::info::NnueNetworkArchitecture`
  - [x] Full NNUE evaluation command tests

### Other Engine Commands
- [x] `uciok` - UCI initialization complete  
  **Path**: `uci_beyond::engine_commands::UciOkCommand`
- [ ] `readyok` - Response to isready
- [ ] `bestmove` - Best move found
- [ ] `copyprotection` - Copy protection check
- [ ] `registration` - Registration status

## GUI Command Responses

Structured responses to GUI commands:

- [x] UCI command response  
  **Path**: `uci_beyond::gui_command_responses::UciCommandResponse`
  - [x] ID block  
    **Path**: `uci_beyond::engine_commands::IdBlock`
  - [x] Option block  
    **Path**: `uci_beyond::engine_commands::UciOptionBlock`
  - [x] uciok command  
    **Path**: `uci_beyond::engine_commands::UciOkCommand`
  - [x] Async parsing support
- [ ] Go command response (partial implementation)  
  **Path**: `uci_beyond::gui_command_responses::GoCommandResponse`
  - [x] Basic structure defined  
    **Path**: `uci_beyond::gui_command_responses::go::BasicGoCommandResponse`
  - [x] Info string block structure  
    **Path**: `uci_beyond::gui_command_responses::go::InfoStringBlock`
  - [ ] Info depth blocks
  - [ ] Best move parsing
  - [ ] Ponder move parsing
- [ ] IsReady response (readyok)
- [ ] Stop response (bestmove)

## Model Types

Common types used across the crate:

- [x] `Position` - Board position (startpos or FEN)  
  **Path**: `uci_beyond::model::Position`
- [x] `FenString` - FEN notation string  
  **Path**: `uci_beyond::model::FenString`
- [x] `MoveString` - UCI long algebraic notation  
  **Path**: `uci_beyond::model::MoveString`
- [x] `Check` - Boolean option type  
  **Path**: `uci_beyond::model::Check`
- [x] `NumaPolicy` - NUMA policy configuration  
  **Path**: `uci_beyond::model::NumaPolicy`
  - [x] Auto/System/None variants
  - [x] Custom policy parsing (structure)
  - [ ] Custom policy validation (marked as TODO)
- [x] `UciString` - String option type  
  **Path**: `uci_beyond::model::UciString`

## Options System

UCI option definitions and parsing:

- [x] `UciOption` enum  
  **Path**: `uci_beyond::options::UciOption`
- [x] `UciOptionKind` - Known option types  
  **Path**: `uci_beyond::options::UciOptionKind`
- [x] `UciOptionType` - Type tags (check, spin, combo, button, string)  
  **Path**: `uci_beyond::options::UciOptionType`
- [x] `Spin` - Numeric range option  
  **Path**: `uci_beyond::options::Spin`
- [x] Typed option data structures  
  **Path**: `uci_beyond::options::typed_uci_option_data`
- [x] Option parsing from engine output
- [ ] Complete combo option support

## Utilities

- [x] `AsyncReadable` trait - Async reading of commands  
  **Path**: `uci_beyond::util::AsyncReadable`
- [x] `StreamingLineReader` - Line-by-line reading  
  **Path**: `uci_beyond::util::StreamingLineReader`
- [x] Command parsing framework  
  **Path**: `uci_beyond::command`
- [x] Error types and handling  
  **Path**: `uci_beyond::command::parsing::Error`
- [x] Display implementations for command serialization

## Known Limitations & TODs

1. **Whitespace Handling**: The crate assumes single spaces between command parameters and doesn't handle arbitrary whitespace
2. **Info Command Parsing**: Depth info commands are not yet implemented (marked as TODO)
3. **ID Block Parsing**: Needs reimplementation using better abstractions (marked as TODO)
4. **Non-standard Commands**: Support for engine-specific extensions (e.g., Stockfish-specific commands) not yet added
5. **Combo Options**: Parsing for combo-type options is marked as `todo!()`
6. **NUMA Policy Validation**: Custom NUMA policy string format validation not implemented
7. **Response Types**: Several commands use `()` or need proper response type definitions
8. **Go Command Response**: Only basic structure defined, needs complete implementation

## Testing Status

- [x] Available processors info command (Display + FromStr + tests)
- [x] NNUE evaluation info command (Display + FromStr + tests)
- [x] NNUE network architecture (Display + FromStr + tests)
- [ ] Depth info command parsing
- [ ] Complete go command response parsing
- [ ] Full integration tests

## Architecture Notes

- Commands implement `Display` trait for serialization (without trailing newlines)
- GUI commands implement `UciCommandTrait` with associated response types
- Engine commands use `Command` trait with associated parsing errors
- Async support through `AsyncReadable` trait for streaming responses
- Error handling uses typed parsing errors with custom variants

## Priority Implementation Areas

High priority items for completion:

1. Info depth commands (commonly used in engine output)
2. `bestmove` command parsing (essential for game play)
3. `readyok` command (completes basic engine interaction)
4. Go command response completion (needed for practical use)
5. Combo option support (for complete option handling)

Medium priority:

1. Non-standard/engine-specific command support
2. Better ID block parsing implementation
3. NUMA policy validation
4. More comprehensive testing

Low priority:

1. Copy protection commands (rarely used)
2. Registration commands (rarely used)
3. Debug command (development feature)
