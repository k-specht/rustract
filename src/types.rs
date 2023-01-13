use std::collections::HashSet;
use std::convert::TryInto;
use std::fmt::{Display, Formatter};
use serde::{Serialize,Deserialize};
use crate::error::{RustractError, GenericError};

/// Holds configuration info for the library.
#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub db_path: String,
    pub schema_path: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub type_path: Option<String>,
}

/// Defines a possible type of Database Data.
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum DataType {
    // String
    String,
    ByteString,
    Json,
    
    // Integer
    Signed64,
    Unsigned64,
    Signed32,
    Unsigned32,
    Signed16,
    Unsigned16,

    // Decimal
    Float64,
    Float32,

    // Other
    Boolean,
    Bit,
    Byte,
    Enum,
    Set
}

/// A Datatype that contains a wrapped version of its enum.
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum DataTypeValue {
    // String
    String(String),
    ByteString(Vec<u8>),
    Json(serde_json::Map<String, serde_json::Value>),
    
    // Integer
    Signed64(i64),
    Unsigned64(u64),
    Signed32(i32),
    Unsigned32(u32),
    Signed16(i16),
    Unsigned16(u16),

    // Decimal
    Float64(f64),
    Float32(f32),

    // Other
    Boolean(bool),
    Bit(u8),
    Byte(u8),
    Enum(u32),
    Set(String)
}

impl DataType {
    pub fn typescript(&self) -> String {
        match self {
            DataType::String => "string",
            DataType::ByteString => "[]",
            DataType::Json => "any",
            DataType::Signed64 => "number",
            DataType::Unsigned64 => "number",
            DataType::Signed32 => "number",
            DataType::Unsigned32 => "number",
            DataType::Signed16 => "number",
            DataType::Unsigned16 => "number",
            DataType::Float64 => "number",
            DataType::Float32 => "number",
            DataType::Boolean => "bool",
            DataType::Bit => "number",
            DataType::Byte => "number",
            DataType::Enum => "Enum",
            DataType::Set => "string"
        }.to_string()
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            DataType::String => "String",
            DataType::ByteString => "Byte String",
            DataType::Json => "JSON",
            DataType::Signed64 => "Signed 64-bit Integer",
            DataType::Unsigned64 => "Unsigned 64-bit Integer",
            DataType::Signed32 => "Signed 32-bit Integer",
            DataType::Unsigned32 => "Unsigned 32-bit Integer",
            DataType::Signed16 => "Signed 16-bit Integer",
            DataType::Unsigned16 => "Unsigned 16-bit Integer",
            DataType::Float64 => "64-bit Float",
            DataType::Float32 => "32-bit Float",
            DataType::Boolean => "Boolean",
            DataType::Bit => "Bit",
            DataType::Byte => "Byte",
            DataType::Enum => "Enum",
            DataType::Set => "Set"
        })
    }
}

/// Retrieves the number of digits of a generic number.
pub(crate) fn digits<T>(num: &T) -> usize
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

pub(crate) trait HasLength {
    fn length(&self) -> isize;
}

pub(crate) trait HasBytes {
    fn byte_length(&self) -> isize;
}

impl HasLength for String {
    fn length(&self) -> isize {
        self.len().try_into().unwrap()
    }
}

impl HasBytes for String {
    fn byte_length(&self) -> isize {
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

impl HasLength for f64 {
    fn length(&self) -> isize {
        digits(self).try_into().unwrap()
    }
}

impl HasLength for f32 {
    fn length(&self) -> isize {
        digits(self).try_into().unwrap()
    }
}

/// Adds indexing functions to the implementing type.
pub(crate) trait IndexOf {
    /// Retrieves the first index of the specified sequence.
    fn index_of(&self, sequence: &str) -> Option<usize>;

    /// Retrieves the next index of the first sequence matched.
    fn next_index_of(&self, sequence: &str, from: usize) -> Option<usize>;
}

impl IndexOf for String {
    fn index_of(&self, sequence: &str) -> Option<usize> {
        self.next_index_of(sequence, 0)
    }

    fn next_index_of(&self, sequence: &str, from: usize) -> Option<usize> {
        let char_sequence: Vec<char> = sequence.chars().collect();
        let mut index = 0;
        let mut matching: bool;
        for (pos, character) in self.chars().skip(from).enumerate() {
            // Prevent out of bounds when doesn't exist
            if index == char_sequence.len() {
                break;
            }

            // Proceed through each character in the sequence and reset
            if character == char_sequence[index] {
                matching = true;
                index += 1;
            } else {
                matching = false;
                index = 0;
            }

            // If all characters matched, return the sequence
            if matching && index == char_sequence.len() {
                return Some(pos+from);
            }
        }
        None
    }
}

impl IndexOf for &str {
    fn index_of(&self, sequence: &str) -> Option<usize> {
        self.next_index_of(sequence, 0)
    }

    fn next_index_of(&self, sequence: &str, from: usize) -> Option<usize> {
        let char_sequence: Vec<char> = sequence.chars().collect();
        let mut index = 0;
        let mut matching: bool;
        for (pos, character) in self.chars().skip(from).enumerate() {
            // Prevent out of bounds when doesn't exist
            if index == char_sequence.len() {
                break;
            }

            // Proceed through each character in the sequence and reset
            if character == char_sequence[index] {
                matching = true;
                index += 1;
            } else {
                matching = false;
                index = 0;
            }

            // If all characters matched, return the sequence
            if matching && index == char_sequence.len() {
                return Some(pos+from);
            }
        }
        None
    }
}

/// A trait that allows converting Vectors into HashSets.
pub trait IntoHashSet {
    /// Converts this vector into a HashSet.
    fn into_set(self) -> HashSet<String>;
}

impl IntoHashSet for Vec<String> {
    fn into_set(self) -> HashSet<String> {
        let mut set: HashSet<String> = HashSet::new();
        for value in self {
            set.insert(value);
        }
        set
    }
}

pub(crate) fn capitalize(string: &str) -> Result<String, RustractError> {
    return if string.is_empty() {
        Err(RustractError::Generic(GenericError {
            message: "cannot capitalize an empty string".to_string(),
        }))
    } else {
        Ok(format!("{}{}", &string[0..1].to_uppercase(), &string[1..string.len()]))
    }
}

#[cfg(test)]
mod test {

    use regex::Regex;

    use super::*;

    #[test]
    fn index_test() {
        // Tests &str indexing
        let index_this = "Find the (! (Hint: there's two!)";
        let index = index_this.index_of("(").unwrap();

        assert_eq!(index, 9);
        assert_eq!(index_this.next_index_of("(", index + 1).unwrap(), 12);

        // Tests String indexing
        let index_string = String::from(index_this);
        let index_2 = index_string.index_of("(").unwrap();

        assert_eq!(index_2, 9);
        assert_eq!(index_string.next_index_of("(", index_2 + 1).unwrap(), 12);
    }

    #[test]
    fn regex_test() {
        // Tests the Regex library itself
        let good_email = "test_person89@test.com";
        let bad_email = "bad_email@test";
        let regex_str = "(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|\"(?:[\\x01-\\x08\\x0b\\x0c\\x0e-\\x1f\\x21\\x23-\\x5b\\x5d-\\x7f]|\\\\[\\x01-\\x09\\x0b\\x0c\\x0e-\\x7f])*\")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\\x01-\\x08\\x0b\\x0c\\x0e-\\x1f\\x21-\\x5a\\x53-\\x7f]|\\\\[\\x01-\\x09\\x0b\\x0c\\x0e-\\x7f])+)\\])";
        let regex = Regex::new(regex_str).unwrap();

        assert!(regex.is_match(good_email));
        assert!(!regex.is_match(bad_email));
    }
}