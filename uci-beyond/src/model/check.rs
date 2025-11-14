use std::fmt::Display;

use crate::command;

#[derive(Debug)]
pub enum CheckParsingError {
    InvalidCheckValue(String),
}

#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct Check(pub bool);

impl Display for Check {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Check(value) = self;
        write!(f, "{value}")
    }
}

impl Check {
    pub fn parse(s: &str) -> Result<(Self, &str), command::parsing::Error<CheckParsingError>> {
        debug_assert_eq!(s, s.trim_start());

        let value_str = s
            .split_whitespace()
            .next()
            .ok_or(command::parsing::Error::UnexpectedEndOfTokens)?;

        let s = s.trim_start_matches(value_str).trim_start();

        let value = match value_str {
            "true" => true,
            "false" => false,
            _ => {
                return Err(command::parsing::Error::CustomError(
                    CheckParsingError::InvalidCheckValue(value_str.to_string()),
                ));
            }
        };

        Ok((Check(value), s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_check() {
        let (check, rest) = Check::parse("true something").expect("Failed to parse Check");
        assert_eq!(check.0, true);
        assert_eq!(rest, "something");

        let (check, rest) = Check::parse("false another").expect("Failed to parse Check");
        assert_eq!(check.0, false);
        assert_eq!(rest, "another");
    }
}
