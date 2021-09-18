use crate::{error::BackendError, types::Config, filesystem::{read_type}};

pub fn init(config: &Config) -> Result<(), BackendError> {
    print!("Stub: {}", config.db_type);
    read_type(config)?;
    Ok(())
}
