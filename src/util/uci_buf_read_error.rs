use crate::command;

#[derive(thiserror::Error, Debug)]
pub enum UciBufReadError<E> {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Custom error: {0}")]
    CustomError(E),
}

impl<E> UciBufReadError<E> {
    pub fn map_custom<F, O>(self, f: F) -> UciBufReadError<O>
    where
        F: FnOnce(E) -> O,
    {
        match self {
            UciBufReadError::IoError(e) => UciBufReadError::IoError(e),
            UciBufReadError::CustomError(e) => UciBufReadError::CustomError(f(e)),
        }
    }
}

impl<E> UciBufReadError<command::parsing::Error<E>> {
    pub fn map_parsing_custom<F, O>(self, f: F) -> UciBufReadError<command::parsing::Error<O>>
    where
        F: FnOnce(E) -> O,
    {
        self.map_custom(|e| e.map_custom(f))
    }

    pub fn from_parsing_custom(err: E) -> Self {
        UciBufReadError::CustomError(command::parsing::Error::CustomError(err))
    }
}
