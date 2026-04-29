use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IoError,
};

/// Errore locale dei menu testuali della CLI.
#[derive(Debug)]
pub enum MenuError {
    Canceled,
    InvalidConfiguration(&'static str),
    Io(IoError),
}

impl Display for MenuError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Canceled => formatter.write_str("Operazione annullata"),
            Self::InvalidConfiguration(message) => formatter.write_str(message),
            Self::Io(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for MenuError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Canceled | Self::InvalidConfiguration(_) => None,
        }
    }
}

impl From<IoError> for MenuError {
    fn from(error: IoError) -> Self {
        Self::Io(error)
    }
}

pub type MenuResult<T> = Result<T, MenuError>;
