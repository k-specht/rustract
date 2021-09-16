use std::convert::TryInto;
use regex::Regex;
use crate::error::BackendError;

/// Configuration requirements.
#[derive(serde::Deserialize, Debug)]
pub struct Config {
    pub db_path: String,
    pub db_type: String,
}

/// A generic database data type.
#[derive(serde::Deserialize, Debug)]
pub struct Data<T> {
    pub contents: T,
    pub size: isize,
    pub bound: bool,
    pub regex: String,
}

/// Allows a type to have a generic length representing its digits or characters.
pub trait HasLength {
    fn length(&self) -> isize;
}

/// Adds functions for the Data struct.
impl<T> Data<T> where T: HasLength {
    /// Tests the data to ensure safety when modifying the database.
    fn test_length(&self) -> Result<String, BackendError> {
        // Tests length using trait
        if self.contents.length() >= self.size {
            return Err(BackendError{
                message: "Data contents are over the size limit for this type.".to_string(),
            });
        }

        Ok("Type validated.".to_string())
    }
}

impl<String> Data<String> where String: AsRef<str> {
    /// Tests the regex for a database String and returns the contents if it passes.
    fn test_regex(&self) -> Result<&str, BackendError> {
        if self.bound {
            match Regex::new(&self.regex) {
                Ok(regex) => {
                    let contents: &str = self.contents.as_ref();
                    if regex.is_match(contents) {
                        return Ok(self.contents.as_ref());
                    }
                    return Err(BackendError {
                        message: "Regex failed to match.".to_string(),
                    });
                },
                Err(error) => {
                    return Err(BackendError {
                        message: format!("Regex failed to compile: {}", error.to_string()),
                    });
                },
            }
        }
        Ok(self.contents.as_ref())
    }
}

/// Retrieves the number of digits of a generic number.
fn digits<T>(num: &T) -> usize
where T: std::ops::DivAssign + std::cmp::PartialOrd + From<u8> + Copy
{
    let mut len = 0;
    let mut i = *num;
    let zero = T::from(0);
    let ten = T::from(10);

    while i > zero {
        i /= ten;
        len += 1;
    }
    
    len
}

impl HasLength for String {
    fn length(&self) -> isize {
        self.len().try_into().unwrap()
    }
}

impl HasLength for i64 {
    fn length(&self) -> isize {
        digits(self).try_into().unwrap()
    }
}

impl HasLength for u64 {
    fn length(&self) -> isize {
        digits(self).try_into().unwrap()
    }
}

impl HasLength for i32 {
    fn length(&self) -> isize {
        digits(self).try_into().unwrap()
    }
}

impl HasLength for u32 {
    fn length(&self) -> isize {
        digits(self).try_into().unwrap()
    }
}

impl HasLength for i16 {
    fn length(&self) -> isize {
        digits(self).try_into().unwrap()
    }
}

impl HasLength for u16 {
    fn length(&self) -> isize {
        digits(self).try_into().unwrap()
    }
}

impl HasLength for u8 {
    fn length(&self) -> isize {
        digits(self).try_into().unwrap()
    }
}
