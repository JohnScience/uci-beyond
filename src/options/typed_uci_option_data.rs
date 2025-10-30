use std::{convert::Infallible, str::FromStr};

use kinded::Kinded;
use strum::EnumIter;

use crate::{
    command,
    model::{self, CheckParsingError},
    options::{Spin, spin::SpinParsingError},
};

#[derive(Debug)]
pub struct UnknownUciOptionType(pub String);

#[derive(Debug)]
pub enum KnownUciOptionDataParsingError {
    SpinParsingError(SpinParsingError),
    StringParsingError,
    CheckParsingError(CheckParsingError),
}

/// The data for the respective [`UciOptionType`] <https://backscattering.de/chess/uci/#engine-option-type>
#[derive(Kinded)]
#[kinded(
    kind = UciOptionType,
    display="snake_case",
    opt_outs=[from_str_impl],
    derive(EnumIter),
)]
pub enum TypedUciOptionData {
    /// a spin wheel that can be an integer in a certain range.
    Spin(Spin),
    /// a text field that has a string as a value. An empty string is represented as `<empty>`.
    String(model::UciString),
    /// a button that can be pressed to send a command to the engine
    Button,
    /// a checkbox that can either be true or false
    Check(model::Check),
    /// a combo box that can have different predefined strings as a value
    Combo(Vec<model::UciString>),
}

impl FromStr for UciOptionType {
    type Err = UnknownUciOptionType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use strum::IntoEnumIterator as _;

        for kind in UciOptionType::iter() {
            if kind.to_string() == s {
                return Ok(kind);
            }
        }

        Err(UnknownUciOptionType(s.to_string()))
    }
}

impl TypedUciOptionData {
    pub fn r#type(&self) -> UciOptionType {
        self.kind()
    }

    fn parse_default_token(
        s: &str,
    ) -> Result<&str, command::parsing::Error<KnownUciOptionDataParsingError>> {
        debug_assert!(s == s.trim_start());

        let Some(without_default_token) = s.strip_prefix("default") else {
            return Err(command::parsing::Error::CustomError(
                KnownUciOptionDataParsingError::StringParsingError,
            ));
        };

        Ok(without_default_token.trim_start())
    }

    pub fn parse_for_type(
        uci_option_type: UciOptionType,
        s: &str,
    ) -> Result<(Self, &str), command::parsing::Error<KnownUciOptionDataParsingError>> {
        debug_assert!(s == s.trim_start());

        match uci_option_type {
            UciOptionType::Button => Ok((TypedUciOptionData::Button, s)),
            UciOptionType::Spin => {
                let (spin, rest) = Spin::parse(s)?;
                Ok((TypedUciOptionData::Spin(spin), rest))
            }
            UciOptionType::String => {
                let s = Self::parse_default_token(s)?;
                let (uci_string, rest) = model::UciString::parse(s)?;
                Ok((TypedUciOptionData::String(uci_string), rest))
            }
            UciOptionType::Check => {
                let s = Self::parse_default_token(s)?;
                let (check, rest) = model::Check::parse(s)?;
                Ok((TypedUciOptionData::Check(check), rest))
            }
            UciOptionType::Combo => todo!(),
        }
    }
}

impl From<command::parsing::Error<SpinParsingError>>
    for command::parsing::Error<KnownUciOptionDataParsingError>
{
    fn from(err: command::parsing::Error<SpinParsingError>) -> Self {
        err.map_custom(KnownUciOptionDataParsingError::SpinParsingError)
    }
}

impl From<command::parsing::Error<Infallible>>
    for command::parsing::Error<KnownUciOptionDataParsingError>
{
    fn from(err: command::parsing::Error<Infallible>) -> Self {
        err.map_custom(|_infallible| KnownUciOptionDataParsingError::StringParsingError)
    }
}

impl From<command::parsing::Error<CheckParsingError>>
    for command::parsing::Error<KnownUciOptionDataParsingError>
{
    fn from(err: command::parsing::Error<CheckParsingError>) -> Self {
        err.map_custom(KnownUciOptionDataParsingError::CheckParsingError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_typed_uci_option_data_check() {
        let (data, rest) = TypedUciOptionData::parse_for_type(
            UciOptionType::Check,
            "default true some other tokens",
        )
        .expect("Failed to parse TypedUciOptionData::Check");

        match data {
            TypedUciOptionData::Check(check) => {
                assert_eq!(check.0, true);
            }
            _ => panic!("Expected TypedUciOptionData::Check"),
        }

        assert_eq!(rest, "some other tokens");
    }
}
