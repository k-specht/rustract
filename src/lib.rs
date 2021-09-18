//! A Rust framework that presents an abstracted interface for secure back-end routing.
//!
//! Author: Käthe Specht
//! Date: 2021-09-01
pub mod error;
pub mod db_driver;
pub mod types;
pub mod filesystem;
use types::Config;
use error::BackendError;
use filesystem::{get_config, load_types};

/// Initializes a local library based on the input settings.
pub fn init(json_path: &str) -> Result<(), BackendError> {
    let config = get_config(json_path)?;
    let tables = load_types(&config)?;

    // Initialize database if there are no types saved
    if tables.is_empty() {
        // Attempts to read the database type in order to parse it properly
        match config.db_type.as_str() {
            "SQL" => init_sql(&config)?,
            _ => return Err(BackendError {
                message: format!("{} is not a valid database type.", config.db_type.as_str()),
            }),
        };
    }

    Ok(())
}

/// Initializes the SQL database interface.
fn init_sql(config: &Config) -> Result<(), BackendError> {
    db_driver::init(config)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_error() {
        let error = init("");
        match error {
            Ok(_) => panic!("Test failed, init function did not produce errors."),
            Err(e) => assert_eq!(e.message, "No such file or directory (os error 2)"),
        };
    }

    #[test]
    fn read_config() {
        create_config();
        let config = get_config("./example.json").unwrap();
        assert_eq!(config.db_path, "./example_database");
        assert_eq!(config.db_type, "SQL");
        assert_eq!(config.type_path, None);
        delete_config();
    }

    /// Creates an example config file for testing purposes.
    fn create_config() {
        let example_config = "{\n  \"db_path\":\"./example_database\",\n  \"db_type\":\"SQL\"\n}";

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
