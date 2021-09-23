use crate::{error::BackendError, filesystem::read_file, types::TableDesign};

/// A database schema struct that can be used for testing JSON.
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
}

pub fn init(schema_path: &str) -> Result<(), BackendError> {
    let schema = read_file(schema_path)?;
    let mut reading = false;
    let db = Database::new(None);

    // Loop until all tables are found
    let index = schema.index_of("CREATE TABLE").unwrap();
    
    Ok(())
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
        let mut index = from;
        let mut matching = false;
        for character in self.chars() {
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
