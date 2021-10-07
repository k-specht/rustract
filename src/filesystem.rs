use std::fs::File;
use std::io::Read;

use crate::error::BackendError;
use crate::types::Config;

/// Gets the config settings from the specified configuration file.
pub fn get_config(json_path: &str) -> Result<Config, BackendError> {
    let s = read_file(json_path)?;
    let json: Config = serde_json::from_str(&s)?;
    Ok(json)
}

/// Reads the file at the specified path.
pub fn read_file(path: &str) -> Result<String, BackendError> {
    let mut file = File::open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok(s)
}

/// Deletes the specified file (usually used after testing).
pub fn _delete_file(filepath: &str) -> Result<(), BackendError> {
    std::fs::remove_file(filepath)?;
    Ok(())
}