use crate::{error::BackendError, types::{Config, TableDesign}, filesystem::{load_types}};

pub fn init(config: &Config) -> Result<(), BackendError> {
    print!("Stub: {}", config.db_type);
    Ok(())
}
