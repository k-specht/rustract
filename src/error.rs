use std::{fmt::{Display, Formatter, Result}, num::ParseIntError};

#[derive(Debug)]
pub enum RustractError {
    DB(GenericError),
    Table(GenericError),
    Field(GenericError),
    Filesystem(GenericError),
    Generic(GenericError),
    ParseInt(ParseIntError),
    IO(std::io::Error),
    JSON(serde_json::Error),
    Regex(regex::Error)
}

impl RustractError {
    pub fn message(&self) -> String {
        match self {
            RustractError::DB(e) => e.message.clone(),
            RustractError::Table(e) => e.message.clone(),
            RustractError::Field(e) => e.message.clone(),
            RustractError::Filesystem(e) => e.message.clone(),
            RustractError::Generic(e) => e.message.clone(),
            RustractError::ParseInt(e) => e.to_string(),
            RustractError::IO(e) => e.to_string(),
            RustractError::JSON(e) => e.to_string(),
            RustractError::Regex(e) => e.to_string()
        }
    }
}

// Adds the Error trait to the RustractError enum.
impl std::error::Error for RustractError {}

impl Display for RustractError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.message())
    }
}

/// Allows parse int errors to be converted into RustractError's.
impl From<std::num::ParseIntError> for RustractError {
    fn from(e: std::num::ParseIntError) -> Self {
        RustractError::ParseInt(e)
    }
}

/// Allows IO errors to be converted into RustractError's.
impl From<std::io::Error> for RustractError {
    fn from(e: std::io::Error) -> Self {
        RustractError::IO(e)
    }
}

/// Allows Serde JSON errors to be converted into RustractError's.
impl From<serde_json::Error> for RustractError {
    fn from(e: serde_json::Error) -> Self {
        RustractError::JSON(e)
    }
}

/// Allows Regex errors to be converted into RustractError's.
impl From<regex::Error> for RustractError {
    fn from(e: regex::Error) -> Self {
        RustractError::Regex(e)
    }
}

/// Allows GenericError's to be converted into RustractError's.
impl From<GenericError> for RustractError {
    fn from(e: GenericError) -> Self {
        RustractError::Generic(e)
    }
}

/// A generic error for removing context from other errors.
/// 
/// Context removal is useful for certain lifetime restrictions.
#[derive(Debug, Clone)]
pub struct GenericError {
    pub message: String,
}

/// Allows parse int errors to be converted into GenericError's.
impl From<std::num::ParseIntError> for GenericError {
    fn from(e: std::num::ParseIntError) -> Self {
        GenericError {
            message: e.to_string(),
        }
    }
}

/// Allows IO errors to be converted into GenericError's.
impl From<std::io::Error> for GenericError {
    fn from(e: std::io::Error) -> Self {
        GenericError {
            message: e.to_string(),
        }
    }
}

/// Allows Serde JSON errors to be converted into GenericError's.
impl From<serde_json::Error> for GenericError {
    fn from(e: serde_json::Error) -> Self {
        GenericError {
            message: e.to_string(),
        }
    }
}

/// Allows Regex errors to be converted into GenericError's.
impl From<regex::Error> for GenericError {
    fn from(e: regex::Error) -> Self {
        GenericError {
            message: e.to_string(),
        }
    }
}

/// Adds the Error trait to GenericError.
impl std::error::Error for GenericError {}

impl Display for GenericError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.message)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_send() {
        fn assert_send<T: Send>() {}
        assert_send::<RustractError>();
    }

    #[test]
    fn test_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<RustractError>();
    }
}
