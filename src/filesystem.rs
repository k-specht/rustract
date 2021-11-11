use std::fs::File;
use std::io::Read;

use crate::error::RustractError;
use crate::types::Config;

/// Gets the config settings from the specified configuration file.
pub fn get_config(json_path: &str) -> Result<Config, RustractError> {
    let s = read_file(json_path)?;
    let json: Config = serde_json::from_str(&s)?;
    Ok(json)
}

/// Reads the file at the specified path.
pub(crate) fn read_file(path: &str) -> Result<String, RustractError> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) => return Err(RustractError {
            message: format!("Failed to find file <{}>: {}", path, err.to_string())
        })
    };
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    Ok(s)
}

/// Deletes the specified file (usually used after testing).
pub(crate) fn _delete_file(filepath: &str) -> Result<(), RustractError> {
    std::fs::remove_file(filepath)?;
    Ok(())
}

/// Reads the provided directory's contents or creates it and reads the new one.
pub(crate) fn _check_path(path: &Option<String>, default: &str) -> Result<std::fs::ReadDir, RustractError> {
    Ok(match path {
        Some(path) => {
            if !std::path::Path::new(path).is_dir() {
                std::fs::create_dir(path)?;
            }
            std::fs::read_dir(path)?
        },
        None => {
            std::fs::create_dir(default)?;
            std::fs::read_dir(default)?
        },
    })
}