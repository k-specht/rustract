use std::{convert::Infallible, fmt::{Display, Formatter, Result}};

/// An initialization error in the Rusty Backend library.
#[derive(Debug, Clone)]
pub struct BackendError {
    pub message: String,
}

/// Allows errors to be compared for testing.
impl std::cmp::PartialEq for BackendError {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message
    }
}

/// Allows IO errors to be converted into BackendError's.
impl From<std::io::Error> for BackendError {
    fn from(e: std::io::Error) -> Self {
        BackendError {
            message: e.to_string(),
        }
    }
}

/// Allows Serde JSON errors to be converted into BackendError's.
impl From<serde_json::Error> for BackendError {
    fn from(e: serde_json::Error) -> Self {
        BackendError {
            message: e.to_string(),
        }
    }
}

/// Allows Regex errors to be converted into BackendError's.
impl From<regex::Error> for BackendError {
    fn from(error: regex::Error) -> Self {
        BackendError {
            message: error.to_string(),
        }
    }
}

// This just adds the Error trait to BackendError's
impl std::error::Error for BackendError {}

// Generation of an error is completely separate from how it is displayed
impl Display for BackendError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.message)
    }
}
