use crate::{error::BackendError, types::Config, filesystem::{read_file,read_type}};
use std::fs::{DirEntry, File, create_dir, read_dir};
use sqlx::mysql::MySqlPoolOptions;

pub fn init(config: &Config) -> Result<(), BackendError> {
    print!("Stub: {}", config.db_type);
    read_type(config)?;
    Ok(())
}
