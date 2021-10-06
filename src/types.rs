use std::convert::TryInto;
use std::fmt::{Display, Formatter};
use regex::Regex;
use serde_json::Value;
use serde::{Serialize,Deserialize};
use crate::error::BackendError;

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
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct FieldDesign {
    pub title: String,
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
}

impl Display for FieldDesign {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.title, self.datatype)
    }
}

impl FieldDesign {
    /// Constructs a new field, defaulting to varchar(255).
    pub fn new(title: &str) -> Self {
        FieldDesign {
            title: String::from(title),
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
            DataType::ByteString => todo!(),
            DataType::JSON => todo!(),
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
            DataType::Float64 => todo!(),
            DataType::Float32 => todo!(),
            DataType::Boolean => {
                self.test_type(json.as_bool())?;
            },
            DataType::Bit => {
                self.test_type(json.as_bool())?;
            },
            DataType::Byte => {
                let json_int = self.test_type(json.as_u64())?;
                self.test_length::<u8>(
                    &self.downsize::<u8,u64>(
                        json_int,
                    )?
                )?;
            },
            DataType::Enum => todo!()
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
                    self.title,
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
                        self.title,
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
                    self.title,
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
                    self.title,
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
                        self.title,
                        regex.to_string()
                    ),
                });
            }
        }

        Ok(())
    }
}

/// Describes a database table's design.
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct TableDesign {
    pub title: String,
    pub fields: Vec<FieldDesign>,
}

impl Display for TableDesign {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ({:?})", self.title, self.fields)
    }
}

impl TableDesign {
    pub fn new(title: &str) -> Self {
        TableDesign {
            title: String::from(title),
            fields: vec![]
        }
    }
    /// Tests the provided JSON values against this table's design.
    /// 
    /// Ignores the required check for any fields marked as primary or generated if input is true.
    pub fn test(&self, fields: &[Value], input: bool) -> Result<(), BackendError> {
        // Iterates over the fields in this design and attempts to match each to the JSON
        for field_design in &self.fields {
            let mut matched = false;

            // Finds a match for this field design
            for field in fields {
                if let Some(val) = field.get(&field_design.title) {
                    matched = true;
                    field_design.test_json(val)?;
                    break;
                }
            }

            // If a required field is missing in the request JSON, decline it
            if (!matched && field_design.required) && ((input && !field_design.primary) || !input) {
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

    /// Saves the configuration info to a JSON file for quick loading.
    pub fn save(&self, filepath: &str) -> Result<(), BackendError> {
        std::fs::write(
            filepath,
            serde_json::to_string_pretty(self)?
        )?;
        Ok(())
    }

    /// Creates an instance of this struct from the JSON file at the specified path.
    pub fn from(filepath: &str) -> Result<Self, BackendError> {
        Ok(serde_json::from_str(&std::fs::read_to_string(filepath)?)?)
    }

    /// Adds the provided field to this table.
    pub fn add(&mut self, field: FieldDesign) {
        self.fields.push(field);
    }

    /// Gets the specified field by its title.
    /// If there's a duplicate, the first is returned.
    pub fn get(&mut self, title: &str) -> Option<&mut FieldDesign> {
        for field in &mut self.fields {
            if field.title == title {
                return Some(field);
            }
        }
        None
    }

    /// Gets a reference to the specified field by its title.
    /// If there's a duplicate, the first is returned.
    pub fn get_ref(&self, title: &str) -> Option<&FieldDesign> {
        for field in &self.fields {
            if field.title == title {
                return Some(field);
            }
        }
        None
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

/// Adds indexing functions to the implementing type.
pub trait IndexOf {
    /// Retrieves the first index of the specified sequence.
    fn index_of(&self, sequence: &str) -> Option<usize>;

    /// Retrieves the next index of the first sequence matched.
    /// 
    /// Note that 
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

#[cfg(test)]
mod test {
    use crate::filesystem::{delete_file, read_file};

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

    #[test]
    fn table_test() {
        let filepath = String::from("./tests/test_type.json");
        let table_design = default_table();
        table_design.save(&filepath).unwrap();
        let string_form = read_file(&filepath).unwrap();
        delete_file(&filepath).unwrap();
        let new_table: TableDesign = serde_json::from_str(&string_form).unwrap();
        assert_eq!(table_design, new_table);
    }

    #[test]
    fn table_data_test() {
        let table_design = default_table();
        let json = serde_json::json!({
            "code": 200,
            "success": true,
            "payload": {
                "fields": [
                    {
                        "title": "User",
                        "email": "test@test.com"
                    },
                ]
            }
        });
        let fields = match json["payload"]["fields"].as_array() {
            Some(val) => val,
            None => panic!("Test failed, could not read JSON data as an array."),
        };
        table_design.test(fields, false).unwrap();
    }

    /// Creates a default TableDesign struct for use in testing.
    fn default_table() -> TableDesign {
        let fields: Vec<FieldDesign> = vec![
            FieldDesign {
                title: String::from("email"),
                datatype: DataType::String,
                bytes: Some(800),
                characters: Some(110),
                decimals: None,
                regex: Some(String::from("(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|\"(?:[\\x01-\\x08\\x0b\\x0c\\x0e-\\x1f\\x21\\x23-\\x5b\\x5d-\\x7f]|\\\\[\\x01-\\x09\\x0b\\x0c\\x0e-\\x7f])*\")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\\x01-\\x08\\x0b\\x0c\\x0e-\\x1f\\x21-\\x5a\\x53-\\x7f]|\\\\[\\x01-\\x09\\x0b\\x0c\\x0e-\\x7f])+)\\])")),
                primary: false,
                unique: false,
                required: true,
                foreign: None,
                increment: false
            },
            FieldDesign {
                title: String::from("name"),
                datatype: DataType::String,
                bytes: Some(800),
                characters: Some(100),
                decimals: None,
                regex: None,
                primary: false,
                unique: false,
                required: false,
                foreign: None,
                increment: false,
            },
        ];
        TableDesign {
            title: String::from("User"),
            fields,
        }
    }
}