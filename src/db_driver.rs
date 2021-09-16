use crate::types::Config;
use sqlx::mysql::MySqlPoolOptions;

pub fn init(config: Config) {
    print!("Stub: {}", config.db_type);

}
