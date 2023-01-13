use std::collections::BTreeMap;

use crate::{error::RustractError, field::FieldDesign, filesystem::read_file, table::TableDesign, types::{DataType, IndexOf, IntoHashSet}};

/// A database schema struct that can be used for testing JSON.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Database {
    pub title: String,
    pub tables: BTreeMap<String, TableDesign>
}

impl std::fmt::Display for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ({:?})", self.title, self.tables)
    }
}

impl Database {
    /// Constructs a new instance of the Database struct.
    pub fn new() -> Self {
        Database {
            title: "Database".to_string(),
            tables: BTreeMap::new()
        }
    }

    /// Constructs a new instance of the Database struct using the given title.
    pub fn from_string(title: String) -> Self {
        Database {
            title,
            tables: BTreeMap::new()
        }
    }

    /// Returns true if the database contains no elements.
    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }

    /// Adds the table to this database.
    pub fn add(&mut self, table: TableDesign) {
        self.tables.insert(table.table_design_title.clone(), table);
    }

    /// Gets a reference to a table in this database by its title.
    ///
    /// If there are duplicates, it retrieves the first.
    pub fn table(&self, title: &str) -> Option<&TableDesign> {
        self.tables.get(title)
    }

    /// Gets a table in this database by its title.
    ///
    /// If there are duplicates, it retrieves the first.
    pub fn table_mut(&mut self, title: &str) -> Option<&mut TableDesign> {
        self.tables.get_mut(title)
    }

    /// Reads a Database schema from the specified filepath.
    pub fn from_schema(schema_path: &str) -> Result<Self, RustractError> {
        let schema = read_file(schema_path)?;
        let mut reading = false;
        let mut db = Database::new();
        let mut table_title = String::new();

        // Loop until all tables are found
        for line_src in schema.lines() {
            let line = line_src.trim();
            // Only read sections that declare new tables
            if line.contains("CREATE TABLE") {
                reading = true;
                table_title = read_name(line)?;
                db.add(TableDesign::new(&table_title));
                continue;
            }

            // Abort reading if the end of the table is reached
            if line.starts_with(')') && line.contains(';') {
                reading = false;
                continue;
            }

            // Add each line to the database
            if reading {
                add_to_db(line, db.table_mut(&table_title).unwrap())?;
            }
        }
        
        Ok(db)
    }

    /// Creates an instance of this struct from the JSON file at the specified path.
    pub fn from(filepath: &str) -> Result<Self, RustractError> {
        Ok(serde_json::from_str(&std::fs::read_to_string(filepath)?)?)
    }

    /// Saves the configuration info to a JSON file for quick loading.
    pub fn save(&self, filepath: &str) -> Result<(), RustractError> {
        std::fs::write(
            filepath,
            serde_json::to_string_pretty(self)?
        )?;
        Ok(())
    }

    /// Exports this database design to a TypeScript library of types.
    /// 
    /// These types can be used in the front-end to standardize routes.
    /// Note that depending on usage, scripts using these may reveal internal Database structure.
    /// This function will return the last encountered error only after processing each table.
    pub fn export(&self, folder: &str) -> Result<(), RustractError> {
        crate::filesystem::check_path(folder)?;

        // Allows each table to complete saving before error is returned
        let mut err_message: Result<(), RustractError> = Ok(());
        for table in self.tables.values() {
            let result = table.export(folder);
            if result.is_err() {
                err_message = result;
            }
        }

        err_message
    }
}

/// Attempts to read the table name from the provided schema line.
fn read_name(line: &str) -> Result<String, RustractError> {
    let tokens: Vec<&str> = line.split(' ').collect();
    for token in tokens {
        if token.starts_with('`') {
            return unwrap_str(token);
        }
    }

    Err(RustractError {
        message: format!("no table name found in schema line: {}", line),
    })
}

/// Attempts to add the schema line's field data to the provided table.
fn add_to_db(source: &str, table: &mut TableDesign) -> Result<(), RustractError> {
    // Gathers the tokens in lower case, separated by a single space each
    let line = source.trim().to_ascii_lowercase();
    if line.is_empty() {
        return Ok(())
    }
    let tokens: Vec<&str> = line.split(' ').filter(|&substr| !substr.is_empty()).collect();

    // Creates a blank field from the line's field name
    if tokens.is_empty() {
        return Err(RustractError {
            message: format!("line {} did not contain any field data.", line),
        });
    }
    if tokens[0].len() < 3 {
        return Err(RustractError {
            message: format!("table field {} cannot have empty name, line: {}", tokens[0], line),
        });
    }
    let mut field = FieldDesign::new("temp");

    // Handles primary key line (returns)
    if tokens[0] == "primary" {
        // Skips over the word "KEY"
        match tokens.get(2) {
            Some(val) => {
                // Sets the requested field to primary
                match table.field_mut(&unwrap_str(*val)?) {
                    Some(value) => value,
                    None => {
                        return Err(RustractError {
                            message: format!("corrupt primary key formation: {} does not exist in new table", *val)
                        });
                    }
                }.primary = true;
                return Ok(());
            },
            None => {
                return Err(RustractError {
                    message: String::from("primary key statement found, but end of line reached"),
                });
            }
        }
    }

    // Handles column lines
    if tokens[0].contains('`') {
        field.field_design_title = unwrap_str(tokens[0])?;
        let descriptor = tokens[1].trim().to_ascii_lowercase();

        // Sets the data type and related fields
        if descriptor.as_str() == "int" {
            field.datatype = if line.contains("unsigned") { DataType::Unsigned64 } else { DataType::Signed64 };
            field.increment = line.contains("auto_increment");
            if field.increment {
                field.generated = true;
            }
            field.bytes = Some(64);
        } else if descriptor.starts_with("varchar(") {
            // Pulls the size out of the varchar wrap and converts it to an integer
            field.datatype = DataType::String;
            let index = match tokens[1].next_index_of(")", 7) {
                Some(val) => val,
                None => return Err(RustractError {
                    message: format!("schema line {} has invalid characters in varchar", line),
                })
            };
            field.characters = Some(tokens[1][8..index].parse()?);
        } else if descriptor.starts_with("enum(") {
            field.datatype = DataType::Enum;
            // Counts all of the elements in the comma-separated enum
            field.enum_set = Some(extract_csl(&descriptor)?);
        } else if descriptor.starts_with("set(") {
            field.datatype = DataType::Set;
            field.set = Some(extract_csl(&descriptor)?.into_set());
        } else if descriptor.contains("tinyint") {
            field.datatype = DataType::Byte;
        } else if descriptor.contains("json") {
            field.datatype = DataType::Json;
        } else {
            return Err(RustractError {
                message: format!("failed to read schema, {} is not a valid token", descriptor),
            });
        }

        // Sets whether the field is null 
        field.required = line.contains("not null");
        table.add(field);
    }

    // Unsupported lines like index declarations are ignored for compatibility
    Ok(())
}

/// Pulls a value out of a sql string-wrapped slice.
fn unwrap_str(str: &str) -> Result<String, RustractError> {
    match str.len() > 1 && str.contains('`') {
        true => {
            // This first unwrap should be safe since it contains this character
            let pos_1 = str.index_of("`").unwrap();
            let pos_2 = str.next_index_of("`", pos_1+1);
            if pos_2.is_none() {
                return Err(RustractError {
                    message: format!("string {} does not have two instances of `'s", str),
                });
            }

            // This is a string slice of a &str, the unwrap is safe due to the previous check
            Ok(str[pos_1+1..pos_2.unwrap()].to_string())
        },
        false => Err(RustractError {
            message: format!("string slice does not match the format `val`: {}", str),
        })
    }
}

/// Unwraps parenthesis to get the contents
fn unwrap_parenthesis(line: &str) -> Result<String, RustractError> {
    // Get the start and end positions of the parenthesis
    let start = match line.index_of("(") {
        Some(index) => index + 1,
        None => return Err(RustractError {
            message: format!(
                "could not unwrap parenthesis, line {} had no start",
                line
            )
        })
    };
    let end = match line.index_of(")") {
        Some(index) => index,
        None => return Err(RustractError {
            message: format!(
                "could not unwrap parenthesis, line {} had no end",
                line
            )
        })
    };

    // Catch )( errors
    if start > end || start >= line.len() {
        return Err(RustractError {
            message: format!(
                "could not unwrap parenthesis, line {} has invalid parenthesis format",
                line
            )
        });
    }

    Ok(line.to_ascii_lowercase()[start..end].to_string())
}

/// Extracts a comma separated list as a vector.
fn extract_csl(line_src: &str) -> Result<Vec<String>, RustractError> {
    // Get line segment containing the CSL
    let mut csl = vec![];
    let line = unwrap_parenthesis(line_src)?;

    for token in line.split(',') {
        csl.push(token.trim().to_string());
    }

    Ok(csl)
}

#[cfg(test)]
mod test {
    use super::*;

    /// Tests pulling values out of SQL string-wrapped slices.
    #[test]
    fn unwrap_test() {
        let unwrap_me = unwrap_str("I wrapped (`this`)...").expect("failed to unwrap str");
        assert_eq!(unwrap_me, "this".to_string());

        // Tests empty strings
        assert_eq!("", unwrap_str("``").unwrap());

        // Tests bounds
        assert_eq!("e", unwrap_str("`e`").unwrap());
    }

    /// Tests the Database extraction code to ensure it obtains the data from the dump.
    #[test]
    fn schema_test() {
        let schema_path = "./tests/schema.sql";
        let db = Database::from_schema(schema_path).expect("schema test failed");
        assert!(!db.is_empty());
        let db_string = db.to_string();
        let table = db.table("user").unwrap_or_else(|| panic!("schema test failed, no user table read: {}", &db_string));
        let field = table.field("email").unwrap_or_else(|| panic!("schema test failed, no email read: {}", &db_string));
        assert!(field.required);
    }

    /// Tests the Rust field bounds extracted from a Database dump.
    #[test]
    fn reading_test() {
        // Gets the date field from the test schema
        let db = Database::from_schema("./tests/schema.sql").unwrap();
        let table_ref: &TableDesign = db.table("user").unwrap();
        let field_ref: &FieldDesign = table_ref.field("registered").unwrap();

        // The good date is below the character limit of 10 (for ISO Strings)
        let good = serde_json::json!({"registered": "2021-01-01"});
        let bad = serde_json::json!({"registered": "2021-01-001"});

        field_ref.extract(&good["registered"]).unwrap();
        assert!(field_ref.extract(&bad["registered"]).is_err());
    }

    /// Creates a test export of the types extracted from the Database dump.
    /// 
    /// These TypeScript types should be compiled manually to complete the test.
    #[test]
    fn typescript_test() {
        let db = Database::from_schema("./tests/schema.sql").unwrap();
        crate::filesystem::check_path("./types/").unwrap();
        db.export("./types/").unwrap();
    }
}
