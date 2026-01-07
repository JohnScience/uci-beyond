use std::{fmt::Display, ops::RangeInclusive, str::FromStr};

use crate::command;

pub enum InfoCommand {
    String(String),
    Depth(DepthInfoCommand),
}

#[derive(thiserror::Error, Debug)]
pub enum InfoCommandParsingError {
    #[error("Unexpected token. Expected `{expected}`, found `{found}`.")]
    UnexpectedToken {
        expected: &'static str,
        found: String,
    },
}

impl command::Command for InfoCommand {
    type ParsingError = InfoCommandParsingError;

    const NAME: &'static str = "info";
}

pub enum StringInfoCommand {
    /// ```text
    /// info string Available processors: 0-7
    /// ```
    AvailableProcessors(AvailableProcessorsInfoCommand),
    /// ```text
    /// info string Using 1 thread
    /// ```
    UsingThreads(UsingThreadsInfoCommand),
    NnueEvaluation(NnueEvaluationInfoCommand),
}

/// Metadata about an NNUE network used in evaluation. Appears in
///
/// ```text
/// info string NNUE evaluation using nn-1c0000000000.nnue (133MiB, (22528, 3072, 15, 32, 1))
/// ```
pub struct NnueEvaluationInfoCommand {
    pub name: String,
    pub size_mib: u32,
    pub architecture: NnueNetworkArchitecture,
}

/// The structure representing the architecture of an NNUE network,
/// e.g. `(22528, 3072, 15, 32, 1)` in
///
/// ```text
/// info string NNUE evaluation using nn-1c0000000000.nnue (133MiB, (22528, 3072, 15, 32, 1))
/// ```
pub struct NnueNetworkArchitecture {
    /// number of input features (material + piece–square features)
    pub inputs_features: u32,
    /// number of neurons in the first fully connected hidden layer (the big one)
    pub hidden_neurons: u32,
    /// the layers after the main hidden layer, i.e., the output head of the network.
    pub additional_layers_dimensions: NnueNetowrkHeadDimensions,
}

/// Dimensions of the output head of an NNUE network,
/// i.e. the last three numbers in [`NnueNetworkArchitecture`].
pub struct NnueNetowrkHeadDimensions(pub [u32; 3]);

/// The
///
/// ```text
/// info string Using 1 thread
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
pub struct UsingThreadsInfoCommand {
    pub threads: u32,
}

impl Display for UsingThreadsInfoCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "info string Using {} thread{}",
            self.threads,
            if self.threads == 1 { "" } else { "s" }
        )
    }
}

/// ```text
/// info string Available processors: 0-7
/// ```
pub struct AvailableProcessorsInfoCommand {
    pub processors: RangeInclusive<u32>,
}

impl Display for AvailableProcessorsInfoCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "info string Available processors: {}-{}",
            self.processors.start(),
            self.processors.end()
        )
    }
}

impl FromStr for AvailableProcessorsInfoCommand {
    type Err = command::parsing::Error<InfoCommandParsingError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::command::Command as _;

        let s = InfoCommand::parse_cmd_name(s)?;
        let string_token = s
            .split_whitespace()
            .next()
            .ok_or(command::parsing::Error::UnexpectedEndOfTokens)?;

        if string_token != "string" {
            return Err(command::parsing::Error::CustomError(
                InfoCommandParsingError::UnexpectedToken {
                    expected: "string",
                    found: string_token.to_string(),
                },
            ));
        }

        let s = s[string_token.len()..].trim_start();

        let available_token = s
            .split_whitespace()
            .next()
            .ok_or(command::parsing::Error::UnexpectedEndOfTokens)?;

        if available_token != "Available" {
            return Err(command::parsing::Error::CustomError(
                InfoCommandParsingError::UnexpectedToken {
                    expected: "Available",
                    found: available_token.to_string(),
                },
            ));
        }

        let s = s[available_token.len()..].trim_start();

        let processors_token = s
            .split_whitespace()
            .next()
            .ok_or(command::parsing::Error::UnexpectedEndOfTokens)?;

        if !processors_token.starts_with("processors:") {
            return Err(command::parsing::Error::CustomError(
                InfoCommandParsingError::UnexpectedToken {
                    expected: "processors:",
                    found: processors_token.to_string(),
                },
            ));
        }

        let s = s[processors_token.len()..].trim_start();

        let (beg, end) = s
            .split_once('-')
            .ok_or(command::parsing::Error::UnexpectedFormat)?;

        let beg = beg
            .trim_end()
            .parse::<u32>()
            .map_err(|_| command::parsing::Error::UnexpectedFormat)?;

        let end = end
            .trim()
            .parse::<u32>()
            .map_err(|_| command::parsing::Error::UnexpectedFormat)?;

        Ok(AvailableProcessorsInfoCommand {
            processors: beg..=end,
        })
    }
}

impl Display for NnueEvaluationInfoCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "info string NNUE evaluation using {} ({}MiB, {})",
            self.name, self.size_mib, self.architecture
        )
    }
}

impl FromStr for NnueEvaluationInfoCommand {
    type Err = command::parsing::Error<InfoCommandParsingError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::command::Command as _;

        let s = InfoCommand::parse_cmd_name(s)?;
        let string_token = s
            .split_whitespace()
            .next()
            .ok_or(command::parsing::Error::UnexpectedEndOfTokens)?;

        if string_token != "string" {
            return Err(command::parsing::Error::CustomError(
                InfoCommandParsingError::UnexpectedToken {
                    expected: "string",
                    found: string_token.to_string(),
                },
            ));
        }

        let s = s[string_token.len()..].trim_start();

        let nnue_token = s
            .split_whitespace()
            .next()
            .ok_or(command::parsing::Error::UnexpectedEndOfTokens)?;

        if nnue_token != "NNUE" {
            return Err(command::parsing::Error::CustomError(
                InfoCommandParsingError::UnexpectedToken {
                    expected: "NNUE",
                    found: nnue_token.to_string(),
                },
            ));
        }

        let s = s[nnue_token.len()..].trim_start();

        let evaluation_token = s
            .split_whitespace()
            .next()
            .ok_or(command::parsing::Error::UnexpectedEndOfTokens)?;

        if evaluation_token != "evaluation" {
            return Err(command::parsing::Error::CustomError(
                InfoCommandParsingError::UnexpectedToken {
                    expected: "evaluation",
                    found: evaluation_token.to_string(),
                },
            ));
        }

        let s = s[evaluation_token.len()..].trim_start();

        let using_token = s
            .split_whitespace()
            .next()
            .ok_or(command::parsing::Error::UnexpectedEndOfTokens)?;

        if using_token != "using" {
            return Err(command::parsing::Error::CustomError(
                InfoCommandParsingError::UnexpectedToken {
                    expected: "using",
                    found: using_token.to_string(),
                },
            ));
        }

        let s = s[using_token.len()..].trim_start();

        // Parse the filename (e.g., "nn-1c0000000000.nnue")
        let name_end = s
            .find(' ')
            .ok_or(command::parsing::Error::UnexpectedFormat)?;
        let name = s[..name_end].to_string();

        let s = s[name_end..].trim_start();

        // Parse "(133MiB, (22528, 3072, 15, 32, 1))"
        if !s.starts_with('(') {
            return Err(command::parsing::Error::UnexpectedFormat);
        }

        let s = &s[1..];

        // Parse size "133MiB"
        let size_end = s
            .find("MiB")
            .ok_or(command::parsing::Error::UnexpectedFormat)?;
        let size_mib = s[..size_end]
            .trim()
            .parse::<u32>()
            .map_err(|_| command::parsing::Error::UnexpectedFormat)?;

        let s = &s[size_end + 3..].trim_start();

        // Expect comma
        if !s.starts_with(',') {
            return Err(command::parsing::Error::UnexpectedFormat);
        }

        let s = s[1..].trim_start();

        // Parse architecture "(22528, 3072, 15, 32, 1))"
        // Find the extent of the architecture substring (including outer closing paren)
        if !s.starts_with('(') {
            return Err(command::parsing::Error::UnexpectedFormat);
        }

        // Find the first closing parenthesis (end of the architecture tuple)
        let close_paren = s
            .find(')')
            .ok_or(command::parsing::Error::UnexpectedFormat)?;

        // Extract substring for architecture parsing: "(22528, 3072, 15, 32, 1))"
        // Include both closing parens (one for tuple, one for outer context)
        let arch_slice = &s[..=close_paren];

        let architecture = arch_slice.parse::<NnueNetworkArchitecture>()?;

        Ok(NnueEvaluationInfoCommand {
            name,
            size_mib,
            architecture,
        })
    }
}

impl Display for NnueNetworkArchitecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}, {})",
            self.inputs_features, self.hidden_neurons, self.additional_layers_dimensions
        )
    }
}

impl FromStr for NnueNetworkArchitecture {
    type Err = command::parsing::Error<InfoCommandParsingError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Parse "(22528, 3072, 15, 32, 1))" - note the double closing paren
        // The format is the architecture tuple followed by closing paren for outer context
        if !s.starts_with('(') {
            return Err(command::parsing::Error::UnexpectedFormat);
        }

        // Find the first closing parenthesis (end of the architecture tuple)
        let close_paren = s
            .find(')')
            .ok_or(command::parsing::Error::UnexpectedFormat)?;

        let inner = &s[1..close_paren];

        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();

        if parts.len() != 5 {
            return Err(command::parsing::Error::UnexpectedFormat);
        }

        let inputs_features = parts[0]
            .parse::<u32>()
            .map_err(|_| command::parsing::Error::UnexpectedFormat)?;

        let hidden_neurons = parts[1]
            .parse::<u32>()
            .map_err(|_| command::parsing::Error::UnexpectedFormat)?;

        let dim1 = parts[2]
            .parse::<u32>()
            .map_err(|_| command::parsing::Error::UnexpectedFormat)?;

        let dim2 = parts[3]
            .parse::<u32>()
            .map_err(|_| command::parsing::Error::UnexpectedFormat)?;

        let dim3 = parts[4]
            .parse::<u32>()
            .map_err(|_| command::parsing::Error::UnexpectedFormat)?;

        Ok(NnueNetworkArchitecture {
            inputs_features,
            hidden_neurons,
            additional_layers_dimensions: NnueNetowrkHeadDimensions([dim1, dim2, dim3]),
        })
    }
}

impl Display for NnueNetowrkHeadDimensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}, {}", self.0[0], self.0[1], self.0[2])
    }
}

/// TODO: Implement parsing for info command responses.
pub struct DepthInfoCommand;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_available_processors_info_command() {
        let s = "info string Available processors: 0-7";
        let cmd = s
            .parse::<AvailableProcessorsInfoCommand>()
            .expect("Failed to parse AvailableProcessorsInfoCommand");

        assert_eq!(*cmd.processors.start(), 0);
        assert_eq!(*cmd.processors.end(), 7);
    }

    #[test]
    fn test_display_available_processors_info_command() {
        let cmd = AvailableProcessorsInfoCommand { processors: 0..=7 };

        let s = format!("{cmd}");
        assert_eq!(s, "info string Available processors: 0-7");
    }

    #[test]
    fn test_parse_nnue_evaluation_info_command() {
        let s = "info string NNUE evaluation using nn-1c0000000000.nnue (133MiB, (22528, 3072, 15, 32, 1))";
        let cmd = s
            .parse::<NnueEvaluationInfoCommand>()
            .expect("Failed to parse NnueEvaluationInfoCommand");

        assert_eq!(cmd.name, "nn-1c0000000000.nnue");
        assert_eq!(cmd.size_mib, 133);
        assert_eq!(cmd.architecture.inputs_features, 22528);
        assert_eq!(cmd.architecture.hidden_neurons, 3072);
        assert_eq!(cmd.architecture.additional_layers_dimensions.0, [15, 32, 1]);
    }

    #[test]
    fn test_display_nnue_evaluation_info_command() {
        let cmd = NnueEvaluationInfoCommand {
            name: "nn-1c0000000000.nnue".to_string(),
            size_mib: 133,
            architecture: NnueNetworkArchitecture {
                inputs_features: 22528,
                hidden_neurons: 3072,
                additional_layers_dimensions: NnueNetowrkHeadDimensions([15, 32, 1]),
            },
        };

        let s = format!("{cmd}");
        assert_eq!(
            s,
            "info string NNUE evaluation using nn-1c0000000000.nnue (133MiB, (22528, 3072, 15, 32, 1))"
        );
    }

    #[test]
    fn test_parse_nnue_network_architecture() {
        let s = "(22528, 3072, 15, 32, 1)";
        let arch = s
            .parse::<NnueNetworkArchitecture>()
            .expect("Failed to parse NnueNetworkArchitecture");

        assert_eq!(arch.inputs_features, 22528);
        assert_eq!(arch.hidden_neurons, 3072);
        assert_eq!(arch.additional_layers_dimensions.0, [15, 32, 1]);
    }

    #[test]
    fn test_display_nnue_network_architecture() {
        let arch = NnueNetworkArchitecture {
            inputs_features: 22528,
            hidden_neurons: 3072,
            additional_layers_dimensions: NnueNetowrkHeadDimensions([15, 32, 1]),
        };

        let s = format!("{arch}");
        assert_eq!(s, "(22528, 3072, 15, 32, 1)");
    }
}
