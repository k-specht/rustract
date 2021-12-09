use std::fmt::{Display, Formatter, Result};

/// An initialization error in the Rusty Backend library.
#[derive(Debug, Clone)]
pub struct RustractError {
    pub message: String,
}

/// Allows errors to be compared for testing.
impl std::cmp::PartialEq for RustractError {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message
    }
}

/// Allows parse int errors to be converted into BackendError's.
impl From<std::num::ParseIntError> for RustractError {
    fn from(e: std::num::ParseIntError) -> Self {
        RustractError {
            message: e.to_string(),
        }
    }
}

/// Allows IO errors to be converted into BackendError's.
impl From<std::io::Error> for RustractError {
    fn from(e: std::io::Error) -> Self {
        RustractError {
            message: e.to_string(),
        }
    }
}

/// Allows Serde JSON errors to be converted into BackendError's.
impl From<serde_json::Error> for RustractError {
    fn from(e: serde_json::Error) -> Self {
        RustractError {
            message: e.to_string(),
        }
    }
}

/// Allows Regex errors to be converted into BackendError's.
impl From<regex::Error> for RustractError {
    fn from(error: regex::Error) -> Self {
        RustractError {
            message: error.to_string(),
        }
    }
}

// This just adds the Error trait to BackendError's
impl std::error::Error for RustractError {}

// Generation of an error is completely separate from how it is displayed
impl Display for RustractError {
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
