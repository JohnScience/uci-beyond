use std::{fmt::Display, str::FromStr};

use crate::model;

#[derive(Debug)]
pub enum NumaPolicyParsingError {
    EmptyNumaPolicyString,
    CustomNumaPolicyStringParsingError(CustomNumaPolicyStringParsingError),
}

#[derive(Debug)]
pub struct CustomNumaPolicyStringParsingError;

/// Precisely specify the available CPUs per [NUMA] domain. ':' separates numa nodes; ',' separates cpu indices; supports "first-last" range syntax for cpu indices, for example `0-15,32-47:16-31,48-63`.
///
/// [NUMA]: https://www.chessprogramming.org/NUMA
#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct CustomNumaPolicyString(pub String);

/// The type for `NumaPolicy` option (see [`options::UciOption::NumaPolicy`](crate::options::UciOption::NumaPolicy)), which binds threads to a specific [NUMA] node to enhance performance on multi-CPU or multi-[NUMA] domain systems.
///
/// [NUMA]: https://www.chessprogramming.org/NUMA
#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub enum NumaPolicy {
    /// assumes a single [NUMA] node, no thread binding
    ///
    /// [NUMA]: https://www.chessprogramming.org/NUMA
    None,
    /// uses [NUMA] information available from the system and binds the threads accordingly
    ///
    /// [NUMA]: https://www.chessprogramming.org/NUMA
    System,
    /// default; automatically selects system or none based on the system
    Auto,
    /// uses [NUMA] information from the underlying hardware and binds the threads accordingly, overrides any previous affinities.
    /// Should be used if Stockfish doesn't utilize all threads, e.g. Windows 10 or certain GUI's like ChessBase.
    ///
    /// [NUMA]: https://www.chessprogramming.org/NUMA
    Hardware,
    /// allows for custom [NUMA] policies to be specified
    ///
    /// [NUMA]: https://www.chessprogramming.org/NUMA
    Custom(CustomNumaPolicyString),
}

impl Display for CustomNumaPolicyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for NumaPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumaPolicy::None => write!(f, "none"),
            NumaPolicy::System => write!(f, "system"),
            NumaPolicy::Auto => write!(f, "auto"),
            NumaPolicy::Hardware => write!(f, "hardware"),
            NumaPolicy::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl TryFrom<model::UciString> for NumaPolicy {
    type Error = NumaPolicyParsingError;

    fn try_from(value: model::UciString) -> Result<Self, Self::Error> {
        let custom = match value.0.as_str() {
            "" => return Err(NumaPolicyParsingError::EmptyNumaPolicyString),
            "none" => return Ok(NumaPolicy::None),
            "system" => return Ok(NumaPolicy::System),
            "auto" => return Ok(NumaPolicy::Auto),
            "hardware" => return Ok(NumaPolicy::Hardware),
            custom => custom,
        };

        let custom: CustomNumaPolicyString = custom
            .parse()
            .map_err(NumaPolicyParsingError::CustomNumaPolicyStringParsingError)?;
        Ok(NumaPolicy::Custom(custom))
    }
}

impl FromStr for CustomNumaPolicyString {
    type Err = CustomNumaPolicyStringParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: implement validation of the custom NUMA policy string format
        Ok(CustomNumaPolicyString(s.to_string()))
    }
}
