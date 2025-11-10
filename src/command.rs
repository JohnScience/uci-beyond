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
    #[derive(thiserror::Error, Debug)]
    pub enum Error<E> {
        #[error("Unexpected end of file")]
        UnexpectedEof,
        #[error("Unexpected end of tokens")]
        UnexpectedEndOfTokens,
        #[error("Unexpected command")]
        UnexpectedCommand,
        #[error("Unexpected format")]
        UnexpectedFormat,
        #[error("Custom parsing error: {0}")]
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

        pub fn wrap<O, RR>(self) -> Result<Option<Result<O, Self>>, RR> {
            Ok(Some(Err(self)))
        }
    }

    impl<E> From<E> for Error<E> {
        fn from(e: E) -> Self {
            Error::CustomError(e)
        }
    }
}
