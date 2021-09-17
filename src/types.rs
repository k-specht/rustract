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
    contents: T,
    pub size: isize,
    pub bound: bool,
    pub regex: String
}

/// Allows a type to have a generic length representing its digits or characters.
pub trait HasLength {
    fn length(&self) -> isize;
}

impl<T> Data<T> {
    /// Constructs a new Data struct based on the parameters.
    pub fn new(contents: T, size: isize, bound: bool, regex: String) -> Data<T> {
        Data { contents, size, bound, regex }
    }
}

/// Adds functions for the Data struct.
impl<T> Data<T> where T: HasLength {
    /// Tests the length for a database entry and returns the contents if it passes.
    fn test_length(&self) -> Result<&T, BackendError> {
        // Tests length using trait
        if self.contents.length() > self.size {
            return Err(BackendError{
                message: "Data contents are over the size limit for this type.".to_string(),
            });
        }

        Ok(&self.contents)
    }

    /// Retrieves the data after performing a test to ensure its integrity.
    pub fn get(&self) -> Result<&T, BackendError> {
        self.test_length()
    }
}

impl<String> Data<String> where String: AsRef<str> {
    /// Tests the regex for a database String and returns the contents if it passes.
    pub fn test_regex(&self) -> Result<&str, BackendError> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn db_data_test() {
        let dbdata: Data<u32> = Data::new(89, 2, false, String::new());
        match dbdata.test_length() {
            Ok(contents) => {
                print!("Contents: {}.", contents);
            }
            Err(error) => {
                panic!("{}", error);
            },
        };
    }

    #[test]
    fn db_regex_test() {
        // Tests the Regex library itself
        let good_email = "test_person89@test.com";
        let bad_email = "bad_email@test";
        let regex_str = "(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|\"(?:[\\x01-\\x08\\x0b\\x0c\\x0e-\\x1f\\x21\\x23-\\x5b\\x5d-\\x7f]|\\\\[\\x01-\\x09\\x0b\\x0c\\x0e-\\x7f])*\")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\\x01-\\x08\\x0b\\x0c\\x0e-\\x1f\\x21-\\x5a\\x53-\\x7f]|\\\\[\\x01-\\x09\\x0b\\x0c\\x0e-\\x7f])+)\\])";
        let regex = Regex::new(regex_str).unwrap();
        assert!(regex.is_match(good_email));
        assert!(!regex.is_match(bad_email));

        // Tests Data's regex testing function
        let good_data = Data::new(
            String::from(good_email), 
            255, 
            true, 
            String::from(regex_str)
        );
        let bad_data = Data::new(
            String::from(bad_email), 
            255, 
            true, 
            String::from(regex_str)
        );
        assert_eq!(good_email, good_data.test_regex().unwrap());
        match bad_data.test_regex() {
            Ok(data) => panic!("Test failed; bad data passed regex test: {}", data),
            Err(error) => assert_eq!(
                BackendError {
                    message: "Regex failed to match.".to_string()
                },
                error
            ),
        };
    }

    #[test]
    fn db_size_test() {
        let dbdata: Data<u32> = Data::new(
            100, 
            2, 
            false, 
            String::new()
        );
        let dbdatastring = Data::new(
            String::from("Kat"), 
            2, 
            false, 
            String::new()
        );

        // Test passes if entering a number over the digit limit fails
        match dbdata.get() {
            Ok(contents) => {
                panic!("Size check failed: size: {}, contents: {}.", dbdata.size, contents);
            }
            Err(error) => {
                print!("Size check passed!: {}", error);
            },
        };
        // Test passes if entering a String over the character limit fails
        match dbdatastring.get() {
            Ok(contents) => {
                panic!("Size check failed: size: {}, contents: {}.", dbdatastring.size, contents);
            }
            Err(error) => {
                print!("Size check passed!: {}", error);
            },
        };
    }
}