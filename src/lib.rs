//! A Rust framework that presents an abstracted interface for secure back-end routing.
//!
//! Author: Käthe Specht
//! Date: 2021-09-01
use std::fs;
use std::io::{Read,Error};

/// Configuration requirements.
#[derive(serde::Deserialize, Debug)]
pub struct Config {
    db_path: String,
    db_type: String,
}

/// Initializes a local library based on the input settings.
pub fn init(json_path: &str) {
    let config = get_config(json_path)
        .unwrap_or_else(|_| {
            panic!("The file at \"{}\" could not be read as a config file.", json_path);
        });

    // Attempts to read the database type in order to parse it properly
    match config.db_type.as_str() {
        "SQL" => init_sql(&config.db_path),
        _ => panic!("{} is not a supported database type.", config.db_type),
    }
}

/// Initializes the SQL database interface.
fn init_sql(db_path: &str) {
    print!("Stub, {}", db_path);
}

/// Gets the config settings from the specified configuration file.
fn get_config(json_path: &str) -> Result<Config, Error> {
    let s = read_file(json_path)?;
    let json: Config = serde_json::from_str(&s)?;
    Ok(json)
}

/// Reads the file at the specified path.
fn read_file(path: &str) -> Result<String, Error> {
    let mut file = fs::File::open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok(s)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_config() {
        create_config();
        let config = get_config("./example.json").unwrap();
        assert_eq!(config.db_path, "./example_database");
        assert_eq!(config.db_type, "mysql");
    }

    /// Creates an example config file for testing purposes.
    fn create_config() {
        let example_config = "{\n  \"db_path\":\"./example_database\",\n  \"db_type\":\"mysql\"\n}";

        std::fs::File::create("./example_config.json")
            .expect("Failed to create config file.");
        std::fs::write("./example.json", example_config)
            .expect("Failed to write to config file.");
    }
}
