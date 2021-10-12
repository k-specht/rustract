use std::fmt::{Display, Formatter};
use regex::Regex;
use serde_json::{Map, Value};
use serde::{Serialize,Deserialize};
use crate::error::BackendError;
use crate::types::{HasLength, HasBytes, DataType};

/// Describes a database table field's design.
/// 
/// This may be more strict than the database allows,
/// but this allows more compatibility and type safety.
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct FieldDesign {
    pub field_design_title: String,
    pub datatype: DataType,
    #[serde(skip_serializing_if="Option::is_none")]
    pub bytes: Option<isize>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub characters: Option<isize>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub decimals: Option<isize>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub regex: Option<String>,
    pub primary: bool,
    pub unique: bool,
    pub required: bool,
    #[serde(skip_serializing_if="Option::is_none")]
    pub foreign: Option<String>,
    pub increment: bool,
    pub generated: bool
}

impl Display for FieldDesign {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.field_design_title, self.datatype)
    }
}

impl FieldDesign {
    /// Constructs a new field, defaulting to varchar(255).
    pub fn new(title: &str) -> Self {
        FieldDesign {
            field_design_title: String::from(title),
            datatype: DataType::String,
            bytes: None,
            characters: None,
            decimals: None,
            regex: None,
            primary: false,
            unique: false,
            required: false,
            foreign: None,
            increment: false,
            generated: false
        }
    }

    /// Tests the provided JSON value against this field's design.
    pub fn test_json(&self, json: &Value) -> Result<(), BackendError> {
        // This match results in duplicated code, but is needed due to limitations of serde_json
        match self.datatype {
            DataType::String => {
                let json_string = String::from(self.test_type(json.as_str())?);
                self.test_length::<String>(&json_string)?;
                self.test_byte_length::<String>(&json_string)?;
                self.test_regex(&json_string)?;
            },
            DataType::ByteString => {
                let json_array = self.test_type(json.as_array())?;
                let mut byte_string = vec![];
                for value in json_array.iter() {
                    byte_string.push(self.downsize::<u8, u64>(self.test_type(value.as_u64())?)?);
                }
                if let Some(bytes) = self.bytes {
                    if byte_string.len() > bytes as usize {
                        return Err(BackendError {
                            message: format!("Bytestring {} is {} bytes long; max size is {} bytes.", self.field_design_title, byte_string.len(), bytes),
                        });
                    }
                }
            },
            DataType::Json => {
                let _json_object: &Map<String, Value> = self.test_type(json.as_object())?;
                // TODO: Decide whether custom JSON field type check will be supported
            },
            DataType::Signed64 => {
                let json_int = self.test_type(json.as_i64())?;
                self.test_length::<i64>(&json_int)?;
            },
            DataType::Unsigned64 => {
                let json_int = self.test_type(json.as_u64())?;
                self.test_length::<u64>(&json_int)?;
            },
            DataType::Signed32 => {
                let json_int = self.test_type(json.as_i64())?;
                self.test_length::<i32>(
                    &self.downsize::<i32,i64>(
                        json_int,
                    )?
                )?;
            },
            DataType::Unsigned32 => {
                let json_int = self.test_type(json.as_u64())?;
                self.test_length::<u32>(
                    &self.downsize::<u32,u64>(
                        json_int,
                    )?
                )?;
            },
            DataType::Signed16 => {
                let json_int = self.test_type(json.as_i64())?;
                self.test_length::<i16>(
                    &self.downsize::<i16,i64>(
                        json_int,
                    )?
                )?;
            },
            DataType::Unsigned16 => {
                let json_int = self.test_type(json.as_u64())?;
                self.test_length::<u16>(
                    &self.downsize::<u16,u64>(
                        json_int,
                    )?
                )?;
            },
            DataType::Float64 => {
                let json_float = self.test_type(json.as_f64())?;
                self.test_length::<f64>(&json_float)?;
            },
            DataType::Float32 => {
                // TODO: Handle possible float precision loss
                let json_float = self.test_type(json.as_f64())?;
                self.test_length::<f32>(
                    &(json_float as f32)
                )?;
            },
            DataType::Boolean => {
                self.test_type(json.as_bool())?;
            },
            DataType::Bit => {
                // TODO: Refactor bit check
                let json_bit = self.test_type(json.as_u64())?;
                let size = crate::types::digits(&json_bit);
                if size > 1 {
                    return Err(BackendError {
                        message: format!("Expected {} to be a bit, but size was {}. Number: \"{}\"", self.field_design_title, size, json_bit),
                    });
                }
            },
            DataType::Byte => {
                let json_int = self.test_type(json.as_u64())?;
                self.test_length::<u8>(
                    &self.downsize::<u8,u64>(
                        json_int,
                    )?
                )?;
            },
            // TODO: Add field property for enum options so this type check can work
            DataType::Enum => {
                let json_enum = self.test_type(json.as_u64())?;
                todo!("Add Enum type check. {}", json_enum);
            }
        };
        Ok(())
    }

    /// Unwraps the Option-wrapped Serde value along with a relevant error message.
    fn test_type<T>(&self, value: Option<T>) -> Result<T, BackendError> {
        match value {
            Some(val) => Ok(val),
            None => Err(BackendError {
                message: format!(
                    "Field {} is not of type {}. (JSON cast failed).",
                    self.field_design_title,
                    self.datatype
                ),
            }),
        }
    }

    /// Tests the length (digits or chars) of the given struct against this field's limit.
    fn test_length<T>(&self, value: &T) -> Result<(), BackendError>
    where T: HasLength
    {
        if let Some(max) = self.characters {
            match value.length() > max {
                true => return Err(BackendError {
                    message: format!(
                        "Field {} is over the size limit of {}.\n(Size: {}).",
                        self.field_design_title,
                        max,
                        value.length()
                    ),
                }),
                false => return Ok(())
            }
        }
        Ok(())
    }

    /// Tests the byte length of the given struct against this field's limit.
    fn test_byte_length<T>(&self, value: &T) -> Result<(), BackendError>
    where T: HasBytes
    {
        if self.bytes.is_some() && value.byte_length() > self.bytes.unwrap() {
            return Err(BackendError {
                message: format!(
                    "Field {} is over the byte limit of {}.\n(Bytes: {}).",
                    self.field_design_title,
                    self.bytes.unwrap(),
                    value.byte_length()
                ),
            })
        }
        Ok(())
    }

    /// Attempts to downsize the given number into the specified size.
    fn downsize<T, E>(&self, value: E) -> Result<T, BackendError>
    where E: Copy + std::convert::TryInto<T>
    {
        match value.try_into() {
            Ok(val) => Ok(val),
            Err(_) => Err(BackendError {
                message: format!(
                    "Field {} is over the byte limit for type {}.",
                    self.field_design_title,
                    self.datatype
                ),
            }),
        }
    }

    /// Tests the given struct against this field's regex restrictions.
    fn test_regex<T>(&self, value: &T) -> Result<(), BackendError>
    where T: AsRef<str>
    {
        if let Some(val) = &self.regex {
            // TODO: Implement Serialize/Deserialize traits for Regex to remove runtime cost.
            let regex = Regex::new(val)?;

            if !regex.is_match(value.as_ref()) {
                return Err(BackendError {
                    message: format!(
                        "Field {} failed to match the regex restriction of {}.",
                        self.field_design_title,
                        regex.to_string()
                    ),
                });
            }
        }

        Ok(())
    }

    /// Exports this field to a String containing TypeScript.
    /// 
    /// TODO: Decide whether to make this exclude generated fields altogether with input.
    pub fn export(&self, input: bool) -> String {
        let mut output = String::new();
        output += "  ";
        output += &self.field_design_title;
        output += if (input && self.generated) || !self.required { "?" } else { "" };
        output += ": ";
        output += &self.datatype.typescript();
        output += ",\n";
        output
    }
}
