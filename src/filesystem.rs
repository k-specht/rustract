use std::fs::{File, ReadDir, create_dir, read_dir};
use std::path::Path;
use std::io::Read;

use crate::error::BackendError;
use crate::types::{Config, DataType};

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

/// Reads types from the path specified in a config file.
pub fn read_type(config: &Config) -> Result<(), BackendError> {
    // Gets the contents of the types directory
    let path = check_path(&config.type_path, "./types/")?;

    // Converts any files in that directory into types
    for dir_entry in path {
        // Gets the data from the Directory Entry
        let entry = dir_entry?;
        let metadata = entry.metadata()?;

        if metadata.is_file() {
            // Gets the path (it might not be a Unicode String)
            // let path = &(entry.path().to_string_lossy());

            // If the file has an extension
            // match get_extension(path) {
            //     Some(ext) => match DataType::from(ext) {
            //         _ => print!("Error"),
            //     },
            //     None => continue,
            // }
            // let contents = read_file(path)?;
            // let json = serde_json::to_value(serde_json::from_str(&contents)?)?;
        }
    }
    Ok(())
}

// fn get_extension(filename: &str) -> Option<&str> {
//     match Path::new(filename).extension() {
//         Some(extension) => Some(&extension.to_string_lossy()),
//         None => None,
//     }
// }

/// Reads the provided directory's contents or creates it and reads the new one.
fn check_path(path: &Option<String>, default: &str) -> Result<ReadDir, BackendError> {
    Ok(match path {
        Some(path) => {
            read_dir(path)?
        },
        None => {
            create_dir(default)?;
            read_dir(default)?
        },
    })
}

/// Deletes the specified file (usually used after testing).
pub fn delete_file(filepath: &str) -> Result<(), BackendError> {
    std::fs::remove_file(filepath)?;
    Ok(())
}