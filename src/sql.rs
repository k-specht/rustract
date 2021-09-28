use crate::{error::BackendError, filesystem::read_file, types::{DataType, FieldDesign, TableDesign}};

/// A database schema struct that can be used for testing JSON.
#[derive(Debug, Clone)]
pub struct Database {
    pub title: Option<String>,
    pub tables: Vec<TableDesign>,
}

impl Database {
    pub fn new(title: Option<String>) -> Self {
        Database {
            title,
            tables: vec![],
        }
    }

    /// Adds the table to this database.
    pub fn add(&mut self, table: TableDesign) {
        self.tables.push(table);
    }

    /// Gets a table in this database by its title. If there are duplicates, it retrieves the first.
    pub fn get(&mut self, title: &str) -> Option<&mut TableDesign> {
        for table in &mut self.tables {
            if table.title == title {
                return Some(table);
            }
        }
        None
    }
}

pub fn init(schema_path: &str) -> Result<Database, BackendError> {
    let schema = read_file(schema_path)?;
    let mut reading = false;
    let mut db = Database::new(None);
    let mut new_table = TableDesign::new("Temp", vec![]);

    // Loop until all tables are found
    for line_src in schema.lines() {
        let line = line_src.trim();
        // Only read sections that declare new tables
        if line.contains("CREATE TABLE") {
            reading = true;
            new_table = TableDesign::new(&read_name(line)?, vec![]);
            db.add(new_table.clone());
            continue;
        }

        // Abort reading if the end of the table is reached
        if line.starts_with(')') && line.contains(';') {
            reading = false;
            continue;
        }

        // Add each line to the database
        if reading {
            add_to_db(line, &mut db.get(&new_table.title).unwrap())?;
        }
    }
    
    Ok(db)
}

/// Attempts to read the table name from the provided schema line.
fn read_name(line: &str) -> Result<String, BackendError> {
    let tokens: Vec<&str> = line.split(' ').collect();
    for token in tokens {
        if token.starts_with('`') {
            return unwrap_str(token);
        }
    }

    Err(BackendError {
        message: format!("No table name found in schema line: {}.", line),
    })
}

/// Attempts to add the schema line's field data to the provided table.
///
/// TODO: Add support for composite keys; Add support for each Datatype.
fn add_to_db(line: &str, table: &mut TableDesign) -> Result<(), BackendError> {
    let tokens: Vec<&str> = line.split(' ').collect();

    // Creates a blank field from the line's field name
    if tokens.is_empty() {
        return Err(BackendError {
            message: format!("Line {} did not contain any field data.", line),
        });
    }
    if tokens[0].len() < 3 {
        return Err(BackendError {
            message: format!("Table field {} cannot have empty name. Line: {}", tokens[0], line),
        });
    }
    let mut field = FieldDesign::new("TEMP");
    if tokens[0] == "PRIMARY" {
        // Skips over the word "KEY"
        match tokens.get(2) {
            Some(val) => {
                // Sets the requested field to primary
                match table.get(*val) {
                    Some(value) => value,
                    None => {
                        return Err(BackendError {
                            message: format!("Corrupt primary key formation: {} does not exist in new table.", *val)
                        });
                    }
                }.primary = true;
            },
            None => {
                return Err(BackendError {
                    message: String::from("Primary key statement found, but end of line reached."),
                });
            }
        }
    }
    field.title = unwrap_str(tokens[0])?;

    // Sets the data type and related fields
    if tokens[1] == "int" {
        field.datatype = if line.contains("unsigned") { DataType::Unsigned64 } else { DataType::Signed64 };
        field.increment = line.contains("AUTO_INCREMENT");
        field.bytes = 64;
    } else if tokens[1].starts_with("varchar(") {
        // Pulls the size out of the varchar wrap and converts it to an integer
        field.datatype = DataType::String;
        let index = match tokens[1].next_index_of(")", 6) {
            Some(val) => val,
            None => return Err(BackendError {
                message: format!("Schema line {} has invalid characters in varchar.", line),
            })
        };
        field.bytes = tokens[1][6..index].parse()?;
    } else {
        return Err(BackendError {
            message: format!("Failed to read schema, {} is not a valid token.", tokens[1]),
        });
    }

    field.required = line.contains("NOT NULL");
    table.add(field);
    Ok(())
}

/// Pulls a value out of a sql string-wrapped slice.
fn unwrap_str(str: &str) -> Result<String, BackendError> {
    match str.len() > 2 && str.contains('`') {
        true => Ok(String::from(&(str[1..str.len()-1]))),
        false => Err(BackendError {
            message: format!("String slice does not match the format `val`: {}", str),
        })
    }
}

/// Adds indexing functions to the implementing type.
trait IndexOf {
    /// Retrieves the first index of the specified sequence.
    fn index_of(&self, sequence: &str) -> Option<usize>;

    /// Retrieves the next index of the first sequence matched.
    fn next_index_of(&self, sequence: &str, from: usize) -> Option<usize>;
}

impl IndexOf for String {
    fn index_of(&self, sequence: &str) -> Option<usize> {
        self.next_index_of(sequence, 0)
    }

    fn next_index_of(&self, sequence: &str, from:usize) -> Option<usize> {
        let char_sequence: Vec<char> = sequence.chars().collect();
        let mut index = 0;
        let mut matching = false;
        for character in self.chars().skip(from) {
            if character == char_sequence[index] {
                matching = true;
                index += 1;
            }
            if matching && index == char_sequence.len() {
                return Some(index);
            }
        }
        None
    }
}

impl IndexOf for &str {
    fn index_of(&self, sequence: &str) -> Option<usize> {
        self.next_index_of(sequence, 0)
    }

    fn next_index_of(&self, sequence: &str, from:usize) -> Option<usize> {
        let char_sequence: Vec<char> = sequence.chars().collect();
        let mut index = 0;
        let mut matching = false;
        for character in self.chars().skip(from) {
            if character == char_sequence[index] {
                matching = true;
                index += 1;
            }
            if matching && index == char_sequence.len() {
                return Some(index);
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn schema_test() {
        let schema_path = "./tests/schema.sql";
        let mut db = init(schema_path).expect("Schema test failed");
        let table = db.get("user").expect("Schema test failed: No user table read.");
        let field = table.get("email").expect("Schema test failed: No email field.");
        assert!(field.required);
    }
}