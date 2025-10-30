# UCI

Universal Chess Interface (UCI) is a protocol used by chess engines to communicate with user interfaces. This crate provides a framework for implementing UCI-compatible engine clients (and, potentially, chess engines) in Rust.

## Known problems

* For ease of implementation, this crate does not deal with arbitrary white space in UCI commands. It assumes that commands's parameters are separated by single spaces only.
