//! A Rust framework that presents an abstracted interface for secure back-end routing.
//!
//! Author: Käthe Specht
//! Date: 2021-09-01
pub mod error;
pub mod db;
pub mod table;
pub mod field;
pub mod types;
mod filesystem;
use error::RustractError;
use filesystem::get_config;

use crate::db::Database;

/// Initializes a local library based on the input settings.
pub fn init(json_path: &str, reload_schema: bool) -> Result<Database, RustractError> {
    // Sets up filepaths
    let config = get_config(json_path)?;
    let type_path = if config.type_path.is_some() { config.type_path.unwrap() } else { "./types/".to_string() };

    // Loads the database from the path, or from the schema if no database is found
    let db: Database = if reload_schema {
        Database::from_schema(&config.schema_path)?
    } else {
        match Database::from(&config.db_path) {
            Ok(file) => file,
            Err(_) => Database::from_schema(&config.schema_path)?,
        }
    };

    // Saves the database to the db path to skip schema reading
    db.save(&config.db_path)?;

    // Exports the database tables to a TypeScript library
    db.export(&type_path)?;

    Ok(db)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_error() {
        let error = init("", false);
        match error {
            Ok(_) => panic!("Test failed, init function did not produce errors."),
            Err(e) => assert_eq!(e.message, "Failed to find file <>: No such file or directory (os error 2)"),
        };
    }

    #[test]
    fn read_config() {
        create_config();
        let config = get_config("./example.json").unwrap();
        assert_eq!(&config.db_path, "./example_database.json");
        assert_eq!(&config.schema_path, "./tests/schema.sql");
        assert_eq!(config.type_path, None);
        delete_config();
    }

    /// Creates an example config file for testing purposes.
    fn create_config() {
        let example_config = "{\n  \"db_path\":\"./example_database.json\",\n \"schema_path\": \"./tests/schema.sql\"\n}";

        std::fs::File::create("./example_config.json")
            .expect("Failed to create config file.");
        std::fs::write("./example.json", example_config)
            .expect("Failed to write to config file.");
    }

    /// Deletes the example config after testing completes.
    fn delete_config() {
        std::fs::remove_file("./example_config.json").expect("Failed to delete file.");
    }
}
