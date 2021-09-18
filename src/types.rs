use std::convert::TryInto;
use std::fmt::{Display, Formatter};
use regex::Regex;
use serde_json::Value;
use serde::{Serialize,Deserialize};
use crate::error::BackendError;

/// Holds configuration info for the library.
#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub db_type: String,
    pub db_path: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub type_path: Option<String>,
}
pub trait Testable {
    fn test(&self) -> Result<(), BackendError>;
}

/// Defines a possible type of Database Data.
#[derive(Deserialize, Serialize, Debug)]
pub enum DataType {
    // String
    String,
    ByteString,
    JSON,
    
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
    Enum
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            DataType::String => "String",
            DataType::ByteString => "Byte String",
            DataType::JSON => "JSON",
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
        })
    }
}

/// Describes a database table field's design.
/// 
/// This may be more strict than the database allows,
/// but this allows more compatibility and type safety.
#[derive(Deserialize, Serialize, Debug)]
pub struct FieldDesign {
    pub title: String,
    pub datatype: DataType,
    pub bytes: isize,
    pub characters: isize,
    pub decimals: isize,
    pub regex_bound: bool,
    pub regex: String,
    pub primary: bool,
    pub unique: bool,
    pub required: bool,
    #[serde(skip_serializing_if="Option::is_none")]
    pub foreign: Option<String>,
}

impl FieldDesign {
    /// Tests the provided JSON value against this field's design.
    fn test_json(&self, json: &Value) -> Result<(), BackendError> {
        // This match results in duplicated code, but is needed due to limitations of serde_json
        match self.datatype {
            DataType::String => {
                let json_string = String::from(test_type(self, json.as_str())?);
                test_length::<String>(self, &json_string)?;
                test_byte_length::<String>(self, &json_string)?;
                test_regex(self, &json_string)?;
                
            },
            DataType::ByteString => todo!(),
            DataType::JSON => todo!(),
            DataType::Signed64 => {
                let json_int = test_type(self, json.as_i64())?;
                test_length::<i64>(self, &json_int)?;
            },
            DataType::Unsigned64 => {
                let json_int = test_type(self, json.as_u64())?;
                test_length::<u64>(self, &json_int)?;
            },
            DataType::Signed32 => {
                let json_int = test_type(self, json.as_i64())?;
                test_length::<i32>(
                    self, 
                    &downsize::<i32,i64>(
                        self,
                        json_int,
                    )?
                )?;
            },
            DataType::Unsigned32 => {
                let json_int = test_type(self, json.as_u64())?;
                test_length::<u32>(
                    self, 
                    &downsize::<u32,u64>(
                        self,
                        json_int,
                    )?
                )?;
            },
            DataType::Signed16 => {
                let json_int = test_type(self, json.as_i64())?;
                test_length::<i16>(
                    self, 
                    &downsize::<i16,i64>(
                        self,
                        json_int,
                    )?
                )?;
            },
            DataType::Unsigned16 => {
                let json_int = test_type(self, json.as_u64())?;
                test_length::<u16>(
                    self, 
                    &downsize::<u16,u64>(
                        self,
                        json_int,
                    )?
                )?;
            },
            DataType::Float64 => todo!(),
            DataType::Float32 => todo!(),
            DataType::Boolean => {
                test_type(self, json.as_bool())?;
            },
            DataType::Bit => {
                test_type(self, json.as_bool())?;
            },
            DataType::Byte => {
                let json_int = test_type(self, json.as_u64())?;
                test_length::<u8>(
                    self, 
                    &downsize::<u8,u64>(
                        self,
                        json_int,
                    )?
                )?;
            },
            DataType::Enum => todo!(),
            _ => {
                // This is only reached if there's a development issue
                panic!("Unsupported DataType used: {:?}", self.datatype);
            }
        };
        Ok(())
    }
}

/// Attempts to downsize the given number into the specified size.
fn downsize<T, E>(field_design: &FieldDesign, value: E) -> Result<T, BackendError>
where E: Copy + std::convert::TryInto<T>
{
    match value.try_into() {
        Ok(val) => Ok(val),
        Err(error) => Err(BackendError {
            message: format!(
                "Field {} is over the byte limit for type {}.",
                field_design.title,
                field_design.datatype
            )
        }),
    }
}

/// Tests the given JSON value against the listed type using Serde.
fn test_type<T>(field_design: &FieldDesign, value: Option<T>) -> Result<T, BackendError> {
    match value {
        Some(val) => Ok(val),
        None => Err(BackendError {
            message: format!(
                "Field {} is not of type {}. (JSON cast failed).",
                field_design.title,
                field_design.datatype
            ),
        }),
    }
}

/// Tests the length (digits or chars) of the given struct against the field's limit.
fn test_length<T>(field_design: &FieldDesign, value: &T) -> Result<(), BackendError>
where T: HasLength
{
    match value.length() > field_design.characters {
        true => Err(BackendError {
            message: format!(
                "Field {} is over the size limit of {}.",
                field_design.title,
                field_design.characters
            ),
        }),
        false => Ok(())
    }
}

/// Tests the byte length of the given struct against the field's limit.
fn test_byte_length<T>(field_design: &FieldDesign, value: &T) -> Result<(), BackendError>
where T: HasBytes
{
    match value.byte_length() > field_design.bytes {
        true => Err(BackendError {
            message: format!(
                "Field {} is over the byte limit of {}.",
                field_design.title,
                field_design.bytes
            ),
        }),
        false => Ok(()),
    }
}

/// Tests the given struct against the field's regex restrictions.
fn test_regex<T>(field_design: &FieldDesign, value: &T) -> Result<(), BackendError>
where T: AsRef<str>
{
    if field_design.regex_bound {
        // TODO: Implement Serialize/Deserialize traits for Regex to remove runtime cost.
        let regex = Regex::new(&field_design.regex)?;

        if !regex.is_match(value.as_ref()) {
            return Err(BackendError {
                message: format!(
                    "Field {} failed to match the regex restriction of {}.",
                    field_design.title,
                    regex.to_string()
                ),
            });
        }
    }

    Ok(())
}

/// Describes a database table's design.
#[derive(Deserialize, Serialize, Debug)]
pub struct TableDesign {
    pub title: String,
    pub fields: Vec<FieldDesign>,
}

impl TableDesign {
    /// Tests the provided JSON values against this table's design.
    fn test(&self, fields: &Vec<Value>) -> Result<(), BackendError> {
        // Iterates over the fields in this design and attempts to match each to the JSON
        for field_design in &self.fields {
            let mut matched = false;

            // Finds a match for this field design
            for field in fields {
                match field.get(&field_design.title) {
                    // Once matched, run the field's relevant tests
                    Some(val) => {
                        matched = true;
                        field_design.test_json(val)?;
                        break;
                    },
                    None => {
                        // Ignore non-matching or unrelated fields
                    }
                }
            }

            // If a required field is missing in the request JSON, decline it
            if !matched && field_design.required {
                return Err(BackendError {
                    message: format!(
                        "The {} field is required in {}, but was not included in the request.",
                        field_design.title,
                        self.title
                    ),
                });
            }
        }
        Ok(())
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

trait HasLength {
    fn length(&self) -> isize;
}

trait HasBytes {
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

#[cfg(test)]
mod test {
    use super::*;

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