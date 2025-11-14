use std::str::FromStr as _;

use crate::options;

/// The `type <type>` clause of [`OptionCommand`](crate::engine_commands::option::OptionCommand).
pub struct TypeClause {
    pub uci_type: options::UciOptionType,
}

#[derive(Debug)]
pub enum TypeClauseParsingError {
    /// The `type` token was expected. Encountered something else.
    TypeTokenExpected(String),
    UnknownType(options::UnknownUciOptionType),
    UnexpectedEol,
}

impl From<options::UnknownUciOptionType> for TypeClauseParsingError {
    fn from(err: options::UnknownUciOptionType) -> Self {
        TypeClauseParsingError::UnknownType(err)
    }
}

impl TypeClause {
    fn consume_type_token(s: &str) -> Result<&str, TypeClauseParsingError> {
        let Some(without_type_token) = s.strip_prefix("type") else {
            return Err(TypeClauseParsingError::TypeTokenExpected(s.to_string()));
        };

        Ok(without_type_token.trim_start())
    }

    pub fn parse_clause(s: &str) -> Result<(Self, &str), TypeClauseParsingError> {
        let s = Self::consume_type_token(s)?;

        let Some(uci_type) = s.split_whitespace().next() else {
            return Err(TypeClauseParsingError::UnexpectedEol);
        };

        let rest = s.trim_start_matches(uci_type).trim_start();

        let uci_type = match options::UciOptionType::from_str(uci_type) {
            Ok(uci_type) => uci_type,
            Err(e) => {
                return Err(TypeClauseParsingError::UnknownType(e));
            }
        };

        Ok((TypeClause { uci_type }, rest))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_type_clause() {
        let (clause, rest) = TypeClause::parse_clause("type spin default 20 min 0 max 100")
            .expect("Failed to parse type clause");

        assert_eq!(
            matches!(clause.uci_type, options::UciOptionType::Spin),
            true
        );
        assert_eq!(rest, "default 20 min 0 max 100");
    }
}
