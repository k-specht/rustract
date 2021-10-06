use crate::{error::BackendError, filesystem::read_file, types::{DataType, FieldDesign, TableDesign}};

/// A database schema struct that can be used for testing JSON.
#[derive(Debug, Clone)]
pub struct Database {
    pub title: Option<String>,
    pub tables: Vec<TableDesign>,
}

impl std::fmt::Display for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let title = if self.title.is_some() {&self.title.as_ref().unwrap()} else { "Database" };
        write!(f, "{}: ({:?})", title, self.tables)
    }
}

impl Database {
    pub fn new(title: Option<String>) -> Self {
        Database {
            title,
            tables: vec![],
        }
    }

    /// Returns true if the database contains no elements.
    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
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
            add_to_db(line, db.get(&table_title).unwrap())?;
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
                match table.get(&unwrap_str(*val)?) {
                    Some(value) => value,
                    None => {
                        return Err(BackendError {
                            message: format!("Corrupt primary key formation: {} does not exist in new table.", *val)
                        });
                    }
                }.primary = true;
                return Ok(());
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
        let index = match tokens[1].next_index_of(")", 7) {
            Some(val) => val,
            None => return Err(BackendError {
                message: format!("Schema line {} has invalid characters in varchar.", line),
            })
        };
        field.bytes = tokens[1][8..index].parse()?;
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
    match str.len() > 1 && str.contains('`') {
        true => {
            // This first unwrap should be safe since it contains this character
            let pos_1 = str.index_of("`").unwrap();
            let pos_2 = str.next_index_of("`", pos_1+1);
            if pos_2.is_none() {
                return Err(BackendError {
                    message: format!("String {} does not have two instances of `.", str),
                });
            }

            // This is a string slice of a &str, the unwrap is safe due to the previous check
            Ok(str[pos_1+1..pos_2.unwrap()].to_string())
        },
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

    fn next_index_of(&self, sequence: &str, from:usize) -> Option<usize> {
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
    use super::*;

    #[test]
    fn index_test() {
        let index_this = "Find the (! (Hint: there's two!)";
        let index = index_this.index_of("(").unwrap();
        assert_eq!(index, 9);
        assert_eq!(index_this.next_index_of("(", index+1).unwrap(), 12);
    }

    #[test]
    fn unwrap_test() {
        let unwrap_me = unwrap_str("I wrapped (`this`)...").expect("Failed to unwrap str: ");
        assert_eq!(unwrap_me, String::from("this"));

        // Tests empty strings
        assert_eq!("", unwrap_str("``").unwrap());

        // Tests bounds
        assert_eq!("e", unwrap_str("`e`").unwrap());
    }

    #[test]
    fn schema_test() {
        let schema_path = "./tests/schema.sql";
        let mut db = init(schema_path).expect("Schema test failed");
        assert!(!db.is_empty());
        let db_string = db.to_string();
        let table = db.get("user").unwrap_or_else(|| panic!("Schema test failed: No user table read: {}", &db_string));
        let field = table.get("email").unwrap_or_else(|| panic!("Schema test failed: No email read: {}", &db_string));
        assert!(field.required);
    }
}