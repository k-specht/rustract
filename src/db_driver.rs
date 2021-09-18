use crate::{error::BackendError, types::Config};

pub fn init(config: &Config) -> Result<(), BackendError> {
    print!("Stub: {}", config.db_type);
    
    Ok(())
}

// Example of how to allow calling code to use pools simply.
// pub async fn get_pool()-> Result<(), BackendError>{
//     let pool = sqlx::MySqlPool::connect("mysql://user:pass@host/database").await?;
//     Ok(())
// }