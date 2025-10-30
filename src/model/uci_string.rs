use std::{convert::Infallible, fmt::Display};

use crate::command;

/// String representing a UCI option value.
///
/// When empty, it represents the absence of a value with `<empty>`.
#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct UciString(pub String);

impl UciString {
    pub fn parse(s: &str) -> Result<(Self, &str), command::parsing::Error<Infallible>> {
        debug_assert_eq!(s, s.trim_start());

        let value = s
            .split_whitespace()
            .next()
            .ok_or(command::parsing::Error::UnexpectedEndOfTokens)?;

        let s = s.trim_start_matches(value).trim_start();

        if value == "<empty>" {
            return Ok((UciString(String::new()), s));
        }

        Ok((UciString(value.to_string()), s))
    }
}

impl Default for UciString {
    fn default() -> Self {
        UciString(String::new())
    }
}

impl Display for UciString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "<empty>")
        } else {
            write!(f, "{}", self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uci_string() {
        let (uci_string, rest) =
            UciString::parse("hello world").expect("Failed to parse UciString");
        assert_eq!(uci_string.0, "hello");
        assert_eq!(rest, "world");

        let (uci_string, rest) =
            UciString::parse("<empty> something").expect("Failed to parse UciString");
        assert_eq!(uci_string.0, "");
        assert_eq!(rest, "something");
    }
}
