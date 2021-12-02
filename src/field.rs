use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use regex::Regex;
use serde_json::{Map, Value};
use serde::{Serialize,Deserialize};
use crate::error::RustractError;
use crate::types::{DataType, DataTypeValue, HasBytes, HasLength, capitalize};

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
    pub generated: bool,
    #[serde(skip_serializing_if="Option::is_none")]
    pub enum_set: Option<Vec<String>>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub set: Option<HashSet<String>>
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
            generated: false,
            enum_set: None,
            set: None
        }
    }

    /// Tests the provided JSON value against this field's design and returns the data if valid.
    pub fn extract(&self, json: &Value) -> Result<DataTypeValue, RustractError> {
        // This match results in duplicated code, but is needed due to limitations of serde_json
        match self.datatype {
            DataType::String => {
                let json_string = String::from(self.test_type(json.as_str())?);
                self.test_length::<String>(&json_string)?;
                self.test_byte_length::<String>(&json_string)?;
                self.test_regex(&json_string)?;
                Ok(DataTypeValue::String(json_string))
            },
            DataType::ByteString => {
                let json_array = self.test_type(json.as_array())?;
                let mut byte_string = vec![];
                for value in json_array.iter() {
                    byte_string.push(self.downsize::<u8, u64>(self.test_type(value.as_u64())?)?);
                }
                if let Some(bytes) = self.bytes {
                    if byte_string.len() > bytes as usize {
                        return Err(RustractError {
                            message: format!(
                                "Bytestring {} is {} bytes long; max size is {} bytes.",
                                self.field_design_title,
                                byte_string.len(),
                                bytes
                            ),
                        });
                    }
                }
                Ok(DataTypeValue::ByteString(byte_string))
            },
            DataType::Json => {
                let json_object: Map<String, Value> = self.test_type(json.as_object())?.clone();
                // TODO: Decide whether custom JSON field type check will be supported
                Ok(DataTypeValue::Json(json_object))
            },
            DataType::Signed64 => {
                let json_int = self.test_type(json.as_i64())?;
                self.test_length::<i64>(&json_int)?;
                Ok(DataTypeValue::Signed64(json_int))
            },
            DataType::Unsigned64 => {
                let json_int = self.test_type(json.as_u64())?;
                self.test_length::<u64>(&json_int)?;
                Ok(DataTypeValue::Unsigned64(json_int))
            },
            DataType::Signed32 => {
                let json_int = self.downsize::<i32,i64>(
                    self.test_type(json.as_i64())?
                )?;
                self.test_length::<i32>(
                    &json_int
                )?;
                Ok(DataTypeValue::Signed32(json_int))
            },
            DataType::Unsigned32 => {
                let json_int = self.downsize::<u32,u64>(
                    self.test_type(json.as_u64())?
                )?;
                self.test_length::<u32>(
                    &json_int
                )?;
                Ok(DataTypeValue::Unsigned32(json_int))
            },
            DataType::Signed16 => {
                let json_int = self.downsize::<i16,i64>(
                    self.test_type(json.as_i64())?
                )?;
                self.test_length::<i16>(
                    &json_int
                )?;
                Ok(DataTypeValue::Signed16(json_int))
            },
            DataType::Unsigned16 => {
                let json_int = self.downsize::<u16,u64>(
                    self.test_type(json.as_u64())?
                )?;
                self.test_length::<u16>(
                    &json_int
                )?;
                Ok(DataTypeValue::Unsigned16(json_int))
            },
            DataType::Float64 => {
                let json_float = self.test_type(json.as_f64())?;
                self.test_length::<f64>(&json_float)?;
                Ok(DataTypeValue::Float64(json_float))
            },
            DataType::Float32 => {
                // TODO: Handle possible float precision loss
                let json_float = self.test_type(json.as_f64())?;
                self.test_length::<f32>(
                    &(json_float as f32)
                )?;
                Ok(DataTypeValue::Float32(json_float as f32))
            },
            DataType::Boolean => {
                let json_bool = self.test_type(json.as_bool())?;
                Ok(DataTypeValue::Boolean(json_bool))
            },
            DataType::Bit => {
                // TODO: Refactor bit check
                let json_bit = self.test_type(json.as_u64())?;
                let size = crate::types::digits(&json_bit);
                if size > 1 {
                    return Err(RustractError {
                        message: format!(
                            "Expected {} to be a bit, but size was {}. Number: \"{}\"",
                            self.field_design_title,
                            size,
                            json_bit
                        ),
                    });
                }
                Ok(DataTypeValue::Bit(self.downsize::<u8, u64>(json_bit)?))
            },
            DataType::Byte => {
                let json_int = self.downsize::<u8,u64>(
                    self.test_type(json.as_u64())?
                )?;
                self.test_length::<u8>(
                    &json_int
                )?;
                Ok(DataTypeValue::Byte(json_int))
            },
            DataType::Enum => {
                let json_enum = self.downsize::<u32, u64>(self.test_type(json.as_u64())?)?;
                if let Some(list) = &self.enum_set {
                    if (json_enum as usize) < list.len() {
                        Ok(DataTypeValue::Enum(json_enum))
                    } else {
                        Err(RustractError {
                            message: format!(
                                "Expected {} to be within the enum range {}..{}.",
                                json_enum,
                                0,
                                list.len()
                            )
                        })
                    }
                } else {
                    Err(RustractError {
                        message: "Internal error: enum field has no enum attached!".to_string()
                    })
                }
            },
            DataType::Set => {
                let json_string = self.test_type(json.as_str())?.to_ascii_lowercase();
                if let Some(set) = &self.set {
                    if set.contains(&json_string) {
                        Ok(DataTypeValue::Set(json_string))
                    } else {
                        Err(RustractError {
                            message: format!(
                                "Value {} is not an element of this set.",
                                json_string
                            )
                        })
                    }
                } else {
                    Err(RustractError {
                        message: "Internal error: set field has no set attached!".to_string()
                    })
                }
            }
        }
    }

    /// Creates an export type for this field's data to match against.
    ///
    /// This will fail if this field is not a member of a type.
    /// Currently, only enums are supported.
    pub fn export_type(&self, table_name: &str) -> Result<String, RustractError> {
        let mut output: String = String::new();
        let name: String = format!(
            "export enum {} {{\n",
            enum_name(table_name, &self.field_design_title)?
        );
        output += &name;

        if let DataType::Enum = self.datatype {
            // Add each enum element to the new type
            if let Some(set) = &self.enum_set {
                for (index, element) in set.iter().enumerate() {
                    output += "  ";
                    output += element;
                    if index < set.len() - 1 {
                        output += ",";
                    }
                    output += "\n";
                }
            } else {
                return Err(RustractError {
                    message: format!("Field {} does not have an associated enum set", &self.field_design_title)
                });
            }
        } else {
            return Err(RustractError {
                message: format!("Field {} is not an enum. Other types are invalid here for now", &self.field_design_title)
            });
        }

        output += "}\n";
        Ok(output)
    }

    /// Unwraps the Option-wrapped Serde value along with a relevant error message.
    fn test_type<T>(&self, value: Option<T>) -> Result<T, RustractError> {
        match value {
            Some(val) => Ok(val),
            None => Err(RustractError {
                message: format!(
                    "Field {} is not of type {}. (JSON cast failed).",
                    self.field_design_title,
                    self.datatype
                ),
            }),
        }
    }

    /// Tests the length (digits or chars) of the given struct against this field's limit.
    fn test_length<T>(&self, value: &T) -> Result<(), RustractError>
    where T: HasLength
    {
        if let Some(max) = self.characters {
            match value.length() > max {
                true => return Err(RustractError {
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
    fn test_byte_length<T>(&self, value: &T) -> Result<(), RustractError>
    where T: HasBytes
    {
        if self.bytes.is_some() && value.byte_length() > self.bytes.unwrap() {
            return Err(RustractError {
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
    fn downsize<T, E>(&self, value: E) -> Result<T, RustractError>
    where E: Copy + std::convert::TryInto<T>
    {
        match value.try_into() {
            Ok(val) => Ok(val),
            Err(_) => Err(RustractError {
                message: format!(
                    "Field {} is over the byte limit for type {}.",
                    self.field_design_title,
                    self.datatype
                ),
            }),
        }
    }

    /// Tests the given struct against this field's regex restrictions.
    fn test_regex<T>(&self, value: &T) -> Result<(), RustractError>
    where T: AsRef<str>
    {
        if let Some(val) = &self.regex {
            // TODO: Implement Serialize/Deserialize traits for Regex to remove runtime cost.
            let regex = Regex::new(val)?;

            if !regex.is_match(value.as_ref()) {
                return Err(RustractError {
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
    pub fn export(&self, input: bool, override_name: Option<&str>) -> String {
        // Set enums or other types to be of the correct type
        let mut name: &str = &self.datatype.typescript();
        if let Some(new_name) = override_name {
            name = new_name;
        }

        let mut output = String::new();
        output += "  ";
        output += &self.field_design_title;
        output += if (input && self.generated) || !self.required { "?" } else { "" };
        output += ": ";
        output += name;
        output += ",\n";
        output
    }
}

/// Creates an enum name for the table or field structs to use.
pub(crate) fn enum_name(table_name: &str, field_name: &str) -> Result<String, RustractError> {
    Ok(format!(
        "{}{}Enum",
        &capitalize(table_name)?,
        &capitalize(field_name)?
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_int() {
        let json = json_init();
        let field = FieldDesign {
            field_design_title: "int".to_string(),
            datatype: DataType::Signed32,
            bytes: Some(32),
            characters: None,
            decimals: None,
            regex: None,
            primary: true,
            unique: true,
            required: true,
            foreign: None,
            increment: false,
            generated: true,
            enum_set: None,
            set: None
        };
        assert_eq!(field.extract(json.get("int").unwrap()).unwrap(), DataTypeValue::Signed32(-1_i32));
    }

    #[test]
    fn test_int_64() {
        let json = json_init();
        let field = FieldDesign {
            field_design_title: "int64".to_string(),
            datatype: DataType::Signed64,
            bytes: None,
            characters: None,
            decimals: None,
            regex: None,
            primary: true,
            unique: true,
            required: true,
            foreign: None,
            increment: false,
            generated: true,
            enum_set: None,
            set: None
        };
        assert_eq!(field.extract(json.get("int64").unwrap()).unwrap(), DataTypeValue::Signed64(-4294967297_i64));
    }

    #[test]
    fn test_enum() {
        let json = json_init();
        let field = FieldDesign {
            field_design_title: "enum".to_string(),
            datatype: DataType::Enum,
            bytes: Some(32),
            characters: None,
            decimals: None,
            regex: None,
            primary: true,
            unique: true,
            required: true,
            foreign: None,
            increment: false,
            generated: true,
            enum_set: Some(vec!["Zero".to_string(),"One".to_string(),"Two".to_string(),"Three".to_string(),"Four".to_string(),"Five".to_string(),"Six".to_string(),"Seven".to_string()]),
            set: None
        };
        assert_eq!(field.extract(json.get("enum").unwrap()).unwrap(), DataTypeValue::Enum(7_u32));
    }

    #[test]
    fn test_set() {
        let json = json_init();
        let field = FieldDesign {
            field_design_title: "set".to_string(),
            datatype: DataType::Set,
            bytes: Some(32),
            characters: None,
            decimals: None,
            regex: None,
            primary: true,
            unique: true,
            required: true,
            foreign: None,
            increment: false,
            generated: true,
            enum_set: None,
            set: Some(crate::db::as_set(vec!["test".to_string(),"set".to_string()]))
        };
        assert_eq!(field.extract(json.get("set").unwrap()).unwrap(), DataTypeValue::Set("test".to_string()));
    }

    #[test]
    fn test_bit() {
        let json = json_init();
        let field = FieldDesign {
            field_design_title: "bit".to_string(),
            datatype: DataType::Bit,
            bytes: Some(1),
            characters: Some(1),
            decimals: None,
            regex: None,
            primary: true,
            unique: true,
            required: true,
            foreign: None,
            increment: false,
            generated: true,
            enum_set: None,
            set: None
        };
        assert_eq!(field.extract(json.get("bit").unwrap()).unwrap(), DataTypeValue::Bit(1_u8));
    }

    #[test]
    fn test_byte() {
        let json = json_init();
        let field = FieldDesign {
            field_design_title: "byte".to_string(),
            datatype: DataType::Byte,
            bytes: Some(1),
            characters: None,
            decimals: None,
            regex: None,
            primary: true,
            unique: true,
            required: true,
            foreign: None,
            increment: false,
            generated: true,
            enum_set: None,
            set: None
        };
        assert_eq!(field.extract(json.get("byte").unwrap()).unwrap(), DataTypeValue::Byte(0_u8));
    }

    #[test]
    fn test_uint_32() {
        let json = json_init();
        let field = FieldDesign {
            field_design_title: "uint".to_string(),
            datatype: DataType::Unsigned32,
            bytes: Some(32),
            characters: None,
            decimals: None,
            regex: None,
            primary: true,
            unique: true,
            required: true,
            foreign: None,
            increment: false,
            generated: true,
            enum_set: None,
            set: None
        };
        assert_eq!(field.extract(json.get("uint").unwrap()).unwrap(), DataTypeValue::Unsigned32(1_u32));
    }

    #[test]
    fn test_uint_64() {
        let json = json_init();
        let field = FieldDesign {
            field_design_title: "uint64".to_string(),
            datatype: DataType::Unsigned64,
            bytes: None,
            characters: None,
            decimals: None,
            regex: None,
            primary: true,
            unique: true,
            required: true,
            foreign: None,
            increment: false,
            generated: true,
            enum_set: None,
            set: None
        };
        assert_eq!(field.extract(json.get("uint64").unwrap()).unwrap(), DataTypeValue::Unsigned64(4294967297_u64));
    }

    #[test]
    fn test_float() {
        let json = json_init();
        let field = FieldDesign {
            field_design_title: "float".to_string(),
            datatype: DataType::Float32,
            bytes: Some(32),
            characters: None,
            decimals: None,
            regex: None,
            primary: true,
            unique: true,
            required: true,
            foreign: None,
            increment: false,
            generated: true,
            enum_set: None,
            set: None
        };
        assert_eq!(field.extract(json.get("float").unwrap()).unwrap(), DataTypeValue::Float32(1.1_f32));
    }

    #[test]
    fn test_float_64() {
        let json = json_init();
        let field = FieldDesign {
            field_design_title: "float64".to_string(),
            datatype: DataType::Float64,
            bytes: None,
            characters: None,
            decimals: None,
            regex: None,
            primary: true,
            unique: true,
            required: true,
            foreign: None,
            increment: false,
            generated: true,
            enum_set: None,
            set: None
        };
        assert_eq!(field.extract(json.get("float64").unwrap()).unwrap(), DataTypeValue::Float64(1.1_f64));
    }

    #[test]
    fn test_string() {
        let json = json_init();
        let field = FieldDesign {
            field_design_title: "string".to_string(),
            datatype: DataType::String,
            bytes: None,
            characters: None,
            decimals: None,
            regex: None,
            primary: true,
            unique: true,
            required: true,
            foreign: None,
            increment: false,
            generated: true,
            enum_set: None,
            set: None
        };
        assert_eq!(field.extract(json.get("string").unwrap()).unwrap(), DataTypeValue::String("test".to_string()));
    }

    #[test]
    fn test_byte_string() {
        let json = json_init();
        let field = FieldDesign {
            field_design_title: "byte_string".to_string(),
            datatype: DataType::ByteString,
            bytes: None,
            characters: None,
            decimals: None,
            regex: None,
            primary: true,
            unique: true,
            required: true,
            foreign: None,
            increment: false,
            generated: true,
            enum_set: None,
            set: None
        };
        assert_eq!(field.extract(json.get("byte_string").unwrap()).unwrap(), DataTypeValue::ByteString([0_u8].to_vec()));
    }

    #[test]
    fn test_bool() {
        let json = json_init();
        let field = FieldDesign {
            field_design_title: "boolean".to_string(),
            datatype: DataType::Boolean,
            bytes: Some(1),
            characters: None,
            decimals: None,
            regex: None,
            primary: true,
            unique: true,
            required: true,
            foreign: None,
            increment: false,
            generated: true,
            enum_set: None,
            set: None
        };
        assert_eq!(field.extract(json.get("boolean").unwrap()).unwrap(), DataTypeValue::Boolean(true));
    }

    #[test]
    fn test_json() {
        let json = json_init();
        let field = FieldDesign {
            field_design_title: "json".to_string(),
            datatype: DataType::Json,
            bytes: None,
            characters: None,
            decimals: None,
            regex: None,
            primary: true,
            unique: true,
            required: true,
            foreign: None,
            increment: false,
            generated: true,
            enum_set: None,
            set: None
        };
        let mut map: Map<String, serde_json::Value> = Map::new();
        map.insert("field".to_string(), serde_json::json!("test"));
        assert_eq!(field.extract(json.get("json").unwrap()).unwrap(), DataTypeValue::Json(map));
    }

    fn json_init() -> Value {
        serde_json::json!({
            "int": -1_i32,
            "uint": 1_u32,
            "int64": -4294967297_i64,
            "uint64": 4294967297_u64,
            "float": 1.1_f32,
            "float64": 1.1_f64,
            "string": "test",
            "byte_string": [0_u8],
            "byte": 0_u8,
            "bit": 1_u8,
            "boolean": true,
            "json": { "field": "test" },
            "enum": 7_u32,
            "set": "test"
        })
    }
}
