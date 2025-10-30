use std::{fmt::Display, str::FromStr};

use kinded::Kinded;
use optional_struct::optional_struct;
use strum::{EnumCount, EnumIter};
use variants_data_struct::VariantsDataStruct;

use crate::command;

#[derive(Debug)]
pub struct UnknownSpinFieldKind(pub String);

#[derive(Debug)]
pub enum SpinFieldParsingError {
    UnknownSpinFieldKind(UnknownSpinFieldKind),
    InvalidValue {
        found: String,
        err: core::num::ParseIntError,
    },
}

#[derive(Debug)]
pub enum SpinParsingError {
    SpinFieldParsingError(SpinFieldParsingError),
    MissingFields(SpinBuilder),
}

#[derive(VariantsDataStruct, Kinded, PartialEq, Eq, Clone, Copy, Debug)]
#[variants_data_struct(
    name=Spin,
    attrs(
        #[optional_struct(SpinBuilder)]
        #[derive(Debug, PartialEq, Eq, Clone)]
    ),
    variants_tys_attrs(
        #[derive(Debug, PartialEq, Eq, Clone)]
    )
)]
#[kinded(kind = SpinFieldKind, opt_outs=[from_str_impl], derive(EnumCount, EnumIter))]
pub enum SpinField {
    #[variants_data_struct_field(field_ty_override = u32)]
    Default(u32),
    #[variants_data_struct_field(field_ty_override = u32)]
    Min(u32),
    #[variants_data_struct_field(field_ty_override = u32)]
    Max(u32),
}

impl Display for Spin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { default, min, max } = self;
        write!(f, "default {default} min {min} max {max}")
    }
}

impl SpinFieldKind {
    pub fn name(&self) -> &'static str {
        match self {
            SpinFieldKind::Default => "default",
            SpinFieldKind::Min => "min",
            SpinFieldKind::Max => "max",
        }
    }
}

impl FromStr for SpinFieldKind {
    type Err = UnknownSpinFieldKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use strum::IntoEnumIterator as _;

        for kind in SpinFieldKind::iter() {
            if kind.name() == s {
                return Ok(kind);
            }
        }
        Err(UnknownSpinFieldKind(s.to_string()))
    }
}

impl SpinField {
    fn from_parts(kind: SpinFieldKind, value: u32) -> Self {
        match kind {
            SpinFieldKind::Default => SpinField::Default(value),
            SpinFieldKind::Min => SpinField::Min(value),
            SpinFieldKind::Max => SpinField::Max(value),
        }
    }

    fn parse_spin_field_kind(
        s: &str,
    ) -> Result<(SpinFieldKind, &str), command::parsing::Error<SpinFieldParsingError>> {
        let mut parts = s.split_whitespace();

        let kind_str = parts
            .next()
            .ok_or(command::parsing::Error::UnexpectedEndOfTokens)?;

        let s = s.trim_start_matches(kind_str).trim_start();

        let kind = SpinFieldKind::from_str(kind_str)?;

        Ok((kind, s))
    }

    fn parse(s: &str) -> Result<(Self, &str), command::parsing::Error<SpinFieldParsingError>> {
        let (kind, s) = SpinField::parse_spin_field_kind(s)?;

        let mut parts = s.split_whitespace();

        let value_str = parts
            .next()
            .ok_or(command::parsing::Error::UnexpectedEndOfTokens)?;

        let s = s.trim_start_matches(value_str).trim_start();

        let value: u32 = match value_str.parse() {
            Ok(v) => v,
            Err(err) => {
                return Err(SpinFieldParsingError::InvalidValue {
                    found: value_str.to_string(),
                    err,
                }
                .into());
            }
        };

        let field = SpinField::from_parts(kind, value);

        Ok((field, s))
    }
}

impl Spin {
    pub fn parse(mut s: &str) -> Result<(Self, &str), command::parsing::Error<SpinParsingError>> {
        let mut builder = SpinBuilder::default();

        for _ in 0..SpinFieldKind::COUNT {
            let (field, rest) = SpinField::parse(s)?;

            s = rest;

            match field {
                SpinField::Default(v) => builder.default = Some(v),
                SpinField::Min(v) => builder.min = Some(v),
                SpinField::Max(v) => builder.max = Some(v),
            }
        }

        let spin: Spin = match builder.try_into() {
            Ok(spin) => spin,
            Err(e) => {
                return Err(command::parsing::Error::CustomError(
                    SpinParsingError::MissingFields(e),
                ));
            }
        };

        Ok((spin, s))
    }
}

impl From<UnknownSpinFieldKind> for command::parsing::Error<SpinFieldParsingError> {
    fn from(err: UnknownSpinFieldKind) -> Self {
        command::parsing::Error::CustomError(SpinFieldParsingError::UnknownSpinFieldKind(err))
    }
}

impl From<command::parsing::Error<SpinFieldParsingError>>
    for command::parsing::Error<SpinParsingError>
{
    fn from(err: command::parsing::Error<SpinFieldParsingError>) -> Self {
        command::parsing::Error::map_custom(err, |e| SpinParsingError::SpinFieldParsingError(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spin_field_kind_from_str_impl() {
        assert_eq!(
            SpinFieldKind::from_str("default").unwrap(),
            SpinFieldKind::Default
        );
        assert_eq!(SpinFieldKind::from_str("min").unwrap(), SpinFieldKind::Min);
        assert_eq!(SpinFieldKind::from_str("max").unwrap(), SpinFieldKind::Max);
        assert!(SpinFieldKind::from_str("unknown").is_err());
    }

    #[test]
    fn test_parse_spin_field_kind() {
        let (kind, rest) = SpinField::parse_spin_field_kind("default 20 min 0 max 100")
            .expect("Failed to parse spin field kind");

        assert_eq!(kind, SpinFieldKind::Default);
        assert_eq!(rest, "20 min 0 max 100");
    }

    #[test]
    fn test_parse_spin_field() {
        let (field, rest) =
            SpinField::parse("min 0 default 20 max 100").expect("Failed to parse spin field");

        assert_eq!(field, SpinField::Min(0));
        assert_eq!(rest, "default 20 max 100");
    }

    #[test]
    fn test_parse_spin() {
        let (spin, rest) =
            Spin::parse("default 20 min 0 max 100 extra").expect("Failed to parse spin");

        assert_eq!(
            spin,
            Spin {
                default: 20,
                min: 0,
                max: 100
            }
        );
        assert_eq!(rest, "extra");
    }
}
