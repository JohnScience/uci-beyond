pub trait Command {
    type ParsingError;

    const NAME: &'static str;
    fn parse_name(s: &str) -> Result<&str, parsing::Error<Self::ParsingError>> {
        if s.is_empty() {
            return Err(parsing::Error::UnexpectedEof);
        }

        let Some(without_name) = s.strip_prefix(Self::NAME) else {
            return Err(parsing::Error::UnexpectedCommand);
        };

        let Some(without_space) = without_name.strip_prefix(' ') else {
            return Err(parsing::Error::UnexpectedFormat);
        };

        Ok(without_space)
    }
}

pub mod parsing {
    #[derive(Debug)]
    pub enum Error<E> {
        UnexpectedEof,
        UnexpectedEndOfTokens,
        UnexpectedCommand,
        UnexpectedFormat,
        CustomError(E),
    }

    impl<E> Error<E> {
        pub fn map_custom<F, O>(self, f: F) -> Error<O>
        where
            F: FnOnce(E) -> O,
        {
            match self {
                Error::UnexpectedEof => Error::UnexpectedEof,
                Error::UnexpectedEndOfTokens => Error::UnexpectedEndOfTokens,
                Error::UnexpectedCommand => Error::UnexpectedCommand,
                Error::UnexpectedFormat => Error::UnexpectedFormat,
                Error::CustomError(e) => Error::CustomError(f(e)),
            }
        }
    }

    impl<E> From<E> for Error<E> {
        fn from(e: E) -> Self {
            Error::CustomError(e)
        }
    }

    // We define a new type so that we can implement custom parsing logic.
    // Both std::result::Result and std::str::FromStr are external, so we cannot implement
    // FromStr for Result<IdCommand, Error> directly.
    pub enum Result<T, E> {
        Ok(T),
        Err(Error<E>),
    }
}
